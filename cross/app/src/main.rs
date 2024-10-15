#![no_std]
#![no_main]

use core::error;
use core::str::FromStr;

use app::hotend::{controller::Hotend, heater::Heater, thermistor::Thermistor, thermistor};
use app::planner;
use app::planner::planner::Planner;
use app::sdcard::SdmmcDevice;
use app::utils::stopwatch::Clock;
use defmt::{info, error};
use embassy_executor::Spawner;
use embassy_stm32::adc::AdcChannel;
use embassy_stm32::peripherals::{
    ADC2, DMA1_CH2, DMA1_CH3, PA0, PA1, PA5, PA6, PB0, PB1, PB2, PB4, PC10, PC11, PC12, PC7, PC8, PC9, PD2, SDMMC1, TIM8, UART4
};
use embassy_stm32::sdmmc::{self, Sdmmc};
use embassy_stm32::time::mhz;
use embassy_stm32::usart::{self, UartRx};
use embassy_stm32::{
    adc::Resolution,
    bind_interrupts,
    gpio::{Level, Output, OutputType, Speed as PinSpeed},
    peripherals::{ADC1, DMA1_CH0, DMA1_CH1, PA2, PA3, PB10, PB11, PB9, TIM4, USART3},
    time::hz,
    timer::{
        simple_pwm::{PwmPin, SimplePwm},
        Channel as TimerChannel, low_level::CountingMode,
    },
    usart::{InterruptHandler, Uart},
};
use embassy_sync::signal::Signal;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, mutex::Mutex};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use fs::filesystem::filename::ShortFileName;
use fs::filesystem::files::{File, Mode};
use fs::volume_mgr::{VolumeIdx, VolumeManager};
use heapless::spsc::Queue;
use heapless::{String, Vec};
use math::distance::{Distance, DistanceUnit};
use math::resistance::Resistance;
use math::temperature::Temperature;
use parser::gcode::{GCodeParser, GCommand, GCommandType};
use stepper::stepper::{StatefulOutputPin, Stepper, StepperAttachment, StepperOptions};
use {defmt_rtt as _, panic_probe as _};
use static_cell::StaticCell;

// https://dev.to/theembeddedrustacean/sharing-data-among-tasks-in-rust-embassy-synchronization-primitives-59hk
const MAX_MESSAGE_LEN: usize = 255;
static COMMAND_DISPATCHER_CHANNEL: Channel<ThreadModeRawMutex, String<MAX_MESSAGE_LEN>, 8> =
    Channel::new();
static SD_CARD_CHANNEL: Channel<ThreadModeRawMutex, GCommand, 8> = Channel::new();
static HEATBED_TARGET_TEMPERATURE: Signal<ThreadModeRawMutex, Temperature> = Signal::new();
static HOTEND_TARGET_TEMPERATURE: Signal<ThreadModeRawMutex, Temperature> = Signal::new();
static PLANNER_CHANNEL: Channel<ThreadModeRawMutex, GCommand, 8> = Channel::new();

#[link_section = ".ram_d3"]
static UART_RX_DMA_BUF: StaticCell<[u8; MAX_MESSAGE_LEN]> = StaticCell::new();
#[link_section = ".ram_d3"]
static HOTEND_DMA_BUF: StaticCell<thermistor::DmaBufType> = StaticCell::new();
#[link_section = ".ram_d3"]
static HEATBED_DMA_BUF: StaticCell<thermistor::DmaBufType> = StaticCell::new();

bind_interrupts!(struct Irqs {
    UART4 => usart::InterruptHandler<UART4>;
    SDMMC1 => sdmmc::InterruptHandler<SDMMC1>;
});

struct StepperPin<'a> {
    pin: Output<'a>,
}

impl<'d> StatefulOutputPin for StepperPin<'d> {
    fn set_high(&mut self) {
        self.pin.set_high();
    }

    fn set_low(&mut self) {
        self.pin.set_low();
    }

    fn is_high(&self) -> bool {
        self.pin.is_set_high()
    }
}

#[embassy_executor::task]
async fn input_handler(peri: UART4, rx: PC11, dma_rx: DMA1_CH0) {
    let mut config = embassy_stm32::usart::Config::default();
    config.baudrate = 19200;
    let mut uart = UartRx::new(peri, Irqs, rx, dma_rx, config).expect("Cannot initialize UART RX");

    let mut msg: String<MAX_MESSAGE_LEN> = String::new();
    let tmp = UART_RX_DMA_BUF.init([0u8;MAX_MESSAGE_LEN]);

    info!("Starting input handler loop");

    loop {
        if let Ok(n) = uart.read_until_idle(tmp).await {
            for b in 0..n {
                if tmp[b] == b'\n' {
                    COMMAND_DISPATCHER_CHANNEL.send(msg.clone()).await;
                    info!("[INPUT_HANDLER] {}", msg.as_str());
                    msg.clear();
                } else {
                    // TODO handle buffer overflow
                    msg.push(tmp[b].into()).unwrap();
                }
            }
            tmp.fill(0u8);
        }
        else{
            error!("Cannot read from UART");
        }
    }
}

// #[embassy_executor::task]
// async fn output_handler(peri: USART3, rx: PB11, dma_rx: DMA1_CH0) {
//     let mut config = embassy_stm32::usart::Config::default();
//     config.baudrate = 19200;
//     let mut uart = UartRx::new(peri, Irqs, rx, dma_rx, config).expect("Cannot initialize UART RX");

//     let mut msg: String<MAX_MESSAGE_LEN> = String::new();
//     let mut tmp = [0u8; MAX_MESSAGE_LEN];

//     loop {
//         if let Ok(n) = uart.read_until_idle(&mut tmp).await {
//             for b in tmp {
//                 if b == b'\n' {
//                     COMMAND_DISPATCHER_CHANNEL.send(msg.clone()).await;
//                     msg.clear();
//                 } else {
//                     // TODO handle buffer overflow
//                     msg.push(b.into()).unwrap();
//                 }
//             }
//             tmp = [0u8; MAX_MESSAGE_LEN];
//         }
//     }
// }

#[embassy_executor::task]
async fn command_dispatcher_task() {
    let mut parser = GCodeParser::new();
    let dt = Duration::from_millis(500);

    info!("Starting command dispatcher loop");

    loop {
        let msg = COMMAND_DISPATCHER_CHANNEL.receive().await;
        info!("[COMMAND DISPATCHER] received message {}", msg.as_str());
        if let Some(cmd) = parser.parse(msg.as_str()) {
            info!("[COMMAND DISPATCHER] {}", cmd);
            match cmd {
                // every movement command is redirected to the planner channel
                GCommand::G0 { .. }
                | GCommand::G1 { .. }
                | GCommand::G2 { .. }
                | GCommand::G3 { .. }
                | GCommand::G4 { .. }
                | GCommand::G90
                | GCommand::G91 => {
                    PLANNER_CHANNEL.send(cmd).await;
                },
                GCommand::G20 => parser.set_distance_unit(DistanceUnit::Inch),
                GCommand::G21 => parser.set_distance_unit(DistanceUnit::Millimeter),
                // hotend target temperature is used to update the target temperature of the hotend task
                GCommand::M104 { s } => {
                    info!("Setting hotend target");
                    HOTEND_TARGET_TEMPERATURE.signal(s);
                },
                // heatbed target temperature is used to update the target temperature of the hotend task
                GCommand::M140 { s } => {
                    HEATBED_TARGET_TEMPERATURE.signal(s);
                },
                GCommand::M149 => todo!(),
                GCommand::M20
                | GCommand::M21
                | GCommand::M22
                | GCommand::M23 { .. }
                | GCommand::M24 { .. }
                | GCommand::M25 => {
                    SD_CARD_CHANNEL.send(cmd).await;
                },
                _ => error!("[COMMAND DISPATCHER] command not handler"),
            }
        }
        else{
            error!("[COMMAND DISPATCHER] Invalid command");
        }

        Timer::after(dt).await;
    }
}

// https://dev.to/apollolabsbin/embedded-rust-embassy-analog-sensing-with-adcs-1e2n
#[embassy_executor::task]
async fn hotend_handler(adc_peri: ADC1, dma_peri: DMA1_CH2, read_pin: PA3, heater_tim: TIM4, heater_out_pin: PB9) {
    let readings = HOTEND_DMA_BUF.init([0u16; 1]);

    let thermistor = Thermistor::new(
        adc_peri,
        dma_peri,
        read_pin.degrade_adc(),
        Resolution::BITS12,
        Resistance::from_ohm(100_000),
        Resistance::from_ohm(10_000),
        Temperature::from_kelvin(3950.0),
        readings
    );

    let heater_out = SimplePwm::new(
        heater_tim,
        None,
        None,
        None,
        Some(PwmPin::new_ch4(heater_out_pin, OutputType::PushPull)),
        hz(1),
        CountingMode::EdgeAlignedUp,
    );
    let heater = Heater::new(heater_out, TimerChannel::Ch4);
    let mut hotend = Hotend::new(heater, thermistor);

    let dt = Duration::from_millis(500);
    loop {
        // try to read the target temperature on each iterator
        // we cannot lock to read the target temperature because the update of the hotend must be performed regardless
        {
            let signal = HOTEND_TARGET_TEMPERATURE.try_take();
            if let Some(t) = signal {
                info!("[HOTEND HANDLER] Target temperature: {}", t.to_celsius());
                hotend.set_temperature(t);
            }
        }
        hotend.update(dt).await;
        Timer::after(dt).await;
    }
}

// https://dev.to/apollolabsbin/embedded-rust-embassy-analog-sensing-with-adcs-1e2n
// TODO test with HEATBED_TARGET_TEMPERATURE
#[embassy_executor::task]
async fn heatbed_handler(adc_peri: ADC2, dma_peri: DMA1_CH3, read_pin: PA2, heater_tim: TIM8, heater_out_pin: PC8) {
    let readings = HEATBED_DMA_BUF.init([0u16; 1]);

    let thermistor = Thermistor::new(
        adc_peri,
        dma_peri,
        read_pin.degrade_adc(),
        Resolution::BITS12,
        Resistance::from_ohm(100_000),
        Resistance::from_ohm(10_000),
        Temperature::from_kelvin(3950.0),
        readings
    );

    let heater_out = SimplePwm::new(
        heater_tim,
        None,
        None,
        Some(PwmPin::new_ch3(heater_out_pin, OutputType::PushPull)),
        None,
        hz(1),
        CountingMode::EdgeAlignedUp,
    );
    let heater = Heater::new(heater_out, TimerChannel::Ch4);
    let mut heatbed = Hotend::new(heater, thermistor);

    let dt = Duration::from_millis(500);
    // try to read the target temperature on each iterator
    // we cannot lock to read the target temperature because the update of the hotend must be performed regardless
    loop {
        {
            let signal = HEATBED_TARGET_TEMPERATURE.try_take();
            if let Some(t) = signal {
                heatbed.set_temperature(t);
            }
        }
        heatbed.update(dt).await;
        Timer::after(dt).await;
    }
}

#[embassy_executor::task]
async fn sdcard_handler(
    spi_peri: SDMMC1,
    clk: PC12,
    cmd: PD2,
    d0: PC8,
    d1: PC9,
    d2: PC10,
    d3: PC11,
) {
    let sdmmc = Sdmmc::new_4bit(spi_peri, Irqs, clk, cmd, d0, d1, d2, d3, Default::default());

    let clock = Clock::new();
    let device = SdmmcDevice::new(sdmmc);
    let mut volume_manager = VolumeManager::new(device, clock);
    let mut working_dir = None;
    let mut working_file = None;
    let mut running = false;
    let mut buf: [u8; MAX_MESSAGE_LEN] = [0u8; MAX_MESSAGE_LEN];
    let mut clock = Clock::new();

    let dt = Duration::from_millis(500);
    loop {
        if let Ok(cmd) = SD_CARD_CHANNEL.try_receive() {
            match cmd {
                GCommand::M20 => {
                    let dir = working_dir.expect("Working directory not set");
                    let mut msg: String<MAX_MESSAGE_LEN> =
                        String::from_str("Begin file list").unwrap();
                    volume_manager
                        .iterate_dir(dir, |d| {
                            let name_vec: Vec<u8, 16> =
                                Vec::from_slice(d.clone().name.base_name()).unwrap();
                            let name = String::from_utf8(name_vec).unwrap();
                            msg.push_str(name.as_str()).unwrap();
                            msg.push('\n').unwrap();
                        })
                        .await
                        .expect("Error while listing files");
                    msg.push_str("End file list").unwrap();
                    // TODO send message to UART
                }
                GCommand::M21 => {
                    let working_volume = match volume_manager.open_raw_volume(VolumeIdx(0)).await {
                        Ok(v) => Some(v),
                        Err(_) => defmt::panic!("Cannot find module"),
                    };
                    working_dir = match volume_manager.open_root_dir(working_volume.unwrap()) {
                        Ok(d) => Some(d),
                        Err(_) => defmt::panic!("Cannot open root dir"),
                    };
                }
                GCommand::M23 { filename } => {
                    let dir = working_dir.expect("Working directory not set");
                    working_file = match volume_manager
                        .open_file_in_dir(dir, filename, Mode::ReadOnly)
                        .await
                    {
                        Ok(f) => Some(f),
                        Err(_) => defmt::panic!("File not found"),
                    }
                }
                // ignore the parameters of M24, just start/resume the print
                GCommand::M24 { .. } => {
                    if !running{
                        clock.start();
                        running = true;
                    }
                }
                GCommand::M25 => {
                    if running{
                        clock.stop();
                        running = false;
                    }
                }
                _ => todo!(),
            }
        }

        if running && working_file.is_some() {
            // we can safely unwrap because the existence of the file has been checked during M23
            volume_manager
                .read_line(working_file.unwrap(), &mut buf)
                .await
                .unwrap();
            let vec: Vec<u8, MAX_MESSAGE_LEN> = Vec::from_slice(&buf).expect("Malformed string");
            let str = String::from_utf8(vec).unwrap();
            COMMAND_DISPATCHER_CHANNEL.send(str).await;
        }
        Timer::after(dt).await;
    }
}

#[embassy_executor::task]
async fn planner_handler(
    x_step_pin: PA0,
    x_dir_pin: PB0,
    y_step_pin: PA6,
    y_dir_pin: PB1,
    z_step_pin: PA5,
    z_dir_pin: PB2,
    e_step_pin: PA1,
    e_dir_pin: PB4,
) {
    // --------- X AXIS -----------------

    let x_step = StepperPin {
        pin: Output::new(x_step_pin, Level::Low, PinSpeed::Low),
    };

    let x_dir = StepperPin {
        pin: Output::new(x_dir_pin, Level::Low, PinSpeed::Low),
    };

    let x_stepper = Stepper::new(x_step, x_dir, StepperOptions::default(), Some(StepperAttachment::default()));

    // --------- Y AXIS -----------------

    let y_step = StepperPin {
        pin: Output::new(y_step_pin, Level::Low, PinSpeed::Low),
    };

    let y_dir = StepperPin {
        pin: Output::new(y_dir_pin, Level::Low, PinSpeed::Low),
    };

    let y_stepper = Stepper::new(y_step, y_dir, StepperOptions::default(), Some(StepperAttachment::default()));

    // --------- Z AXIS -----------------

    let z_step = StepperPin {
        pin: Output::new(z_step_pin, Level::Low, PinSpeed::Low),
    };

    let z_dir = StepperPin {
        pin: Output::new(z_dir_pin, Level::Low, PinSpeed::Low),
    };

    let z_stepper = Stepper::new(z_step, z_dir, StepperOptions::default(), Some(StepperAttachment::default()));

    // --------- E AXIS -----------------

    let e_step = StepperPin {
        pin: Output::new(e_step_pin, Level::Low, PinSpeed::Low),
    };

    let e_dir = StepperPin {
        pin: Output::new(e_dir_pin, Level::Low, PinSpeed::Low),
    };

    let e_stepper = Stepper::new(e_step, e_dir, StepperOptions::default(), Some(StepperAttachment::default()));

    let mut planner = Planner::new(x_stepper, y_stepper, z_stepper, e_stepper);

    let dt = Duration::from_millis(500);

    loop {
        let cmd = PLANNER_CHANNEL.receive().await;
        info!("[PLANNER HANDLER] {}", cmd);
        match cmd {
            GCommand::G0 { .. }
            | GCommand::G1 { .. }
            | GCommand::G2 { .. }
            | GCommand::G3 { .. }
            | GCommand::G4 { .. }
            | GCommand::G90
            | GCommand::G91 => {
                planner.execute(cmd).await.expect("Planner error");
            }
            _ => error!("[PLANNER HANDLER] command not handled"),
        }
        info!("[PLANNER HANDLER] Move completed");
        Timer::after(dt).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // TODO check this configuration. It's in the embassy stm32 examples of ADC. Not so sure why it's needed but without this the
    // program won't run
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8), // SPI1 cksel defaults to pll1_q
            divr: None,
        });
        config.rcc.pll2 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV8), // 100mhz
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.mux.adcsel = mux::Adcsel::PLL2_P;
    }
    let p = embassy_stm32::init(config);

    spawner
        .spawn(input_handler(p.UART4, p.PC11, p.DMA1_CH0))
        .unwrap();

    spawner
        .spawn(command_dispatcher_task())
        .unwrap();

    spawner
        .spawn(hotend_handler(p.ADC1, p.DMA1_CH2, p.PA3, p.TIM4, p.PB9))
        .unwrap();

    spawner
        .spawn(heatbed_handler(p.ADC2, p.DMA1_CH3, p.PA2, p.TIM8, p.PC8))
        .unwrap();

    spawner
        .spawn(planner_handler(
            p.PA0, p.PB0, p.PA6, p.PB1, p.PA5, p.PB2, p.PA1, p.PB4,
        ))
        .unwrap();

    // _spawner
    //     .spawn(sdcard_handler(
    //         p.SDMMC1, p.PC12, p.PD2, p.PC8, p.PC9, p.PC10, p.PC11
    //     ))
    //     .unwrap();

    loop {
        info!("[MAIN LOOP] alive");
        Timer::after(Duration::from_secs(1)).await;
    }
}
