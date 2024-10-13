#![no_std]
#![no_main]

use core::str::FromStr;

use app::hotend::{controller::Hotend, heater::Heater, thermistor::Thermistor};
use app::sdcard::SdmmcDevice;
use app::utils::stopwatch::Clock;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::{ADC2, PC10, PC11, PC12, PC8, PC9, PD2, SDMMC1, TIM8};
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
        Channel as TimerChannel, CountingMode,
    },
    usart::{InterruptHandler, Uart},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex, channel::Channel};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use fs::filesystem::filename::ShortFileName;
use fs::filesystem::files::{File, Mode};
use fs::volume_mgr::{VolumeIdx, VolumeManager};
use heapless::spsc::Queue;
use heapless::{String, Vec};
use math::distance::{Distance, DistanceUnit};
use math::temperature::Temperature;
use parser::gcode::{GCodeParser, GCommand, GCommandType};
use stepper::stepper::{StatefulOutputPin, Stepper, StepperOptions};
use {defmt_rtt as _, panic_probe as _};

// https://dev.to/theembeddedrustacean/sharing-data-among-tasks-in-rust-embassy-synchronization-primitives-59hk
const MAX_MESSAGE_LEN: usize = 255;
static COMMAND_DISPATCHER_CHANNEL: Channel<ThreadModeRawMutex, String<MAX_MESSAGE_LEN>, 8> = Channel::new();
static SD_CARD_CHANNEL: Channel<ThreadModeRawMutex, GCommand, 8> = Channel::new();
static HEATBED_TARGET_TEMPERATURE: Mutex<ThreadModeRawMutex, Option<Temperature>> = Mutex::new(None);
static HOTEND_TARGET_TEMPERATURE: Mutex<ThreadModeRawMutex, Option<Temperature>> = Mutex::new(None);
static PLANNER_CHANNEL: Channel<ThreadModeRawMutex, GCommand, 8> = Channel::new();

bind_interrupts!(struct Irqs {
    USART3 => usart::InterruptHandler<USART3>;
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
async fn input_handler(peri: USART3, rx: PB11, dma_rx: DMA1_CH0) {
    let mut config = embassy_stm32::usart::Config::default();
    config.baudrate = 19200;
    let mut uart =
        UartRx::new(peri, Irqs, rx, dma_rx, config).expect("Cannot initialize UART RX");

    let mut msg: String<MAX_MESSAGE_LEN> = String::new();
    let mut tmp = [0u8; MAX_MESSAGE_LEN];

    loop {
        if let Ok(n) = uart.read_until_idle(&mut tmp).await {
            for b in tmp {
                if b == b'\n'{
                    COMMAND_DISPATCHER_CHANNEL.send(msg.clone()).await;
                    msg.clear();
                }
                else{
                    // TODO handle buffer overflow
                    msg.push(b.into()).unwrap();
                }
            }
            tmp = [0u8; MAX_MESSAGE_LEN];
        }
    }
}

#[embassy_executor::task]
async fn command_dispatcher_task() {
    let mut parser = GCodeParser::new();
    let dt = Duration::from_millis(500);
    loop{        
        let msg = COMMAND_DISPATCHER_CHANNEL.receive().await;
        if let Some(cmd) = parser.parse(msg.as_str()){
            match cmd {
                // every movement command is redirected to the planner channel
                GCommand::G0{..} |
                GCommand::G1{..} |
                GCommand::G2{..} |
                GCommand::G3{..} |
                GCommand::G4{..} | 
                GCommand::G90 |
                GCommand::G91 => {
                    PLANNER_CHANNEL.send(cmd).await;
                },
                GCommand::G20 => parser.set_distance_unit(DistanceUnit::Inch),
                GCommand::G21 => parser.set_distance_unit(DistanceUnit::Millimeter),
                // hotend target temperature is used to update the target temperature of the hotend task
                GCommand::M104 { s } => {
                    let mut t = HOTEND_TARGET_TEMPERATURE.lock().await;
                    *t = Some(s);
                },
                // heatbed target temperature is used to update the target temperature of the hotend task
                GCommand::M140 { s } => {
                    let mut t = HEATBED_TARGET_TEMPERATURE.lock().await;
                    *t = Some(s);
                },
                GCommand::M149 => todo!(),
                _ => todo!()
            }
        }
        
        Timer::after(dt).await;
    }
}

// https://dev.to/apollolabsbin/embedded-rust-embassy-analog-sensing-with-adcs-1e2n
#[embassy_executor::task]
async fn hotend_handler(adc_peri: ADC1, read_pin: PA3, heater_tim: TIM4, heater_out_pin: PB9) {
    let thermistor = Thermistor::new(
        adc_peri,
        read_pin,
        Resolution::BITS12,
        100_000.0,
        10_000.0,
        Temperature::from_kelvin(3950.0),
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
            let lock = HOTEND_TARGET_TEMPERATURE.try_lock();
            if let Ok(mut t) = lock{
                if let Some(temp) = t.take(){
                    hotend.set_temperature(temp);
                }
                *t = None;
            }
        }
        hotend.update(dt);
        Timer::after(dt).await;
    }
}

// https://dev.to/apollolabsbin/embedded-rust-embassy-analog-sensing-with-adcs-1e2n
// TODO test with HEATBED_TARGET_TEMPERATURE
#[embassy_executor::task]
async fn heatbed_handler(adc_peri: ADC2, read_pin: PA2, heater_tim: TIM8, heater_out_pin: PC8) {
    let thermistor = Thermistor::new(
        adc_peri,
        read_pin,
        Resolution::BITS12,
        100_000.0,
        10_000.0,
        Temperature::from_kelvin(3950.0),
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
            let lock = HEATBED_TARGET_TEMPERATURE.try_lock();
            if let Ok(mut t) = lock{
                if let Some(temp) = t.take(){
                    heatbed.set_temperature(temp);
                }
                *t = None;
            }
        }
        heatbed.update(dt);
        Timer::after(dt).await;
    }
}

#[embassy_executor::task]
async fn sdcard_handler(spi_peri: SDMMC1, clk: PC12, cmd: PD2, d0: PC8, d1: PC9, d2: PC10, d3: PC11) {
    let sdmmc = Sdmmc::new_4bit(
        spi_peri,
        Irqs,
        clk,
        cmd,
        d0,
        d1,
        d2,
        d3,
        Default::default(),
    );

    let clock = Clock::new();
    let device = SdmmcDevice::new(sdmmc);
    let mut volume_manager = VolumeManager::new(device, clock);
    let mut working_dir = None;
    let mut working_file = None;
    let mut running = false;
    let mut buf: [u8; MAX_MESSAGE_LEN]= [0u8; MAX_MESSAGE_LEN];

    let dt = Duration::from_millis(500);
    loop {
        if let Ok(cmd) = SD_CARD_CHANNEL.try_receive(){
            match cmd{
            GCommand::M20 => {
                let dir = working_dir.expect("Working directory not set");
                let mut str: String<256> = String::from_str("Begin file list").unwrap();
                volume_manager.iterate_dir(dir, |d| {
                    let name_vec: Vec<u8, 16> = Vec::from_slice(d.clone().name.base_name()).unwrap();
                    let name = String::from_utf8(name_vec).unwrap();
                    str.push_str(name.as_str()).unwrap();
                    str.push('\n').unwrap();
                }).await.expect("Error while listing files");
                str.push_str("End file list").unwrap();

            },
            GCommand::M21 => {
                let working_volume = match volume_manager.open_raw_volume(VolumeIdx(0)).await {
                    Ok(v) => Some(v),
                    Err(_) => defmt::panic!("Cannot find module"),
                };
                working_dir = match volume_manager.open_root_dir(working_volume.unwrap()) {
                    Ok(d) => Some(d),
                    Err(_) => defmt::panic!("Cannot open root dir")
                };
            },
            GCommand::M23 { filename } => {
                let dir = working_dir.expect("Working directory not set");
                working_file = match volume_manager.open_file_in_dir(dir, filename, Mode::ReadOnly).await {
                    Ok(f) => Some(f),
                    Err(_) => defmt::panic!("File not found")
                }
            },
            // ignore the parameters of M24, just start/resume the print
            GCommand::M24 {..} => {
                running = true;
            },
            GCommand::M25 => {
                running = false;
            },
            _ => todo!()
            }
        }

        if running && working_file.is_some(){
            // we can safely unwrap because the existence of the file has been checked during M23
            volume_manager.read_line(working_file.unwrap(), &mut buf).await.unwrap();
            let vec: Vec<u8, MAX_MESSAGE_LEN> = Vec::from_slice(&buf).expect("Malformed string");
            let str = String::from_utf8(vec).unwrap();
            COMMAND_DISPATCHER_CHANNEL.send(str).await;
        }
        Timer::after(dt).await;
    }
}


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
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

    // --------- X AXIS -----------------

    let x_step = StepperPin {
        pin: Output::new(p.PA0, Level::Low, PinSpeed::Low),
    };

    let x_dir = StepperPin {
        pin: Output::new(p.PB0, Level::Low, PinSpeed::Low),
    };

    let x_stepper = Stepper::new(x_step, x_dir, StepperOptions::default(), None);

    // --------- Y AXIS -----------------

    let y_step = StepperPin {
        pin: Output::new(p.PA6, Level::Low, PinSpeed::Low),
    };

    let y_dir = StepperPin {
        pin: Output::new(p.PB1, Level::Low, PinSpeed::Low),
    };

    let y_stepper = Stepper::new(y_step, y_dir, StepperOptions::default(), None);

    // --------- Z AXIS -----------------

    let z_step = StepperPin {
        pin: Output::new(p.PA5, Level::Low, PinSpeed::Low),
    };

    let z_dir = StepperPin {
        pin: Output::new(p.PB2, Level::Low, PinSpeed::Low),
    };

    let z_stepper = Stepper::new(z_step, z_dir, StepperOptions::default(), None);

    let mut led = Output::new(p.PD5, Level::Low, PinSpeed::Low);
    led.set_high();

    _spawner
        .spawn(input_handler(
            p.USART3, p.PB11, p.DMA1_CH0
        ))
        .unwrap();

    _spawner
        .spawn(hotend_handler(p.ADC1, p.PA3, p.TIM4, p.PB9))
        .unwrap();

    _spawner
        .spawn(heatbed_handler(p.ADC2, p.PA2, p.TIM8, p.PC8))
        .unwrap();

    loop {
        // let mut c: Option<GCommand> = None;
        // {
        //     let mut q = COMMAND_DISPATCHER_CHANNEL.lock().await;
        //     c = q.dequeue();
        // } // mutex is freed here

        // match c {
        //     Some(cmd) => match cmd {
        //         GCommand::G0 { x, y, z, f } => {
        //             info!("performing a linear movement");
        //             linear_move_to_3d(
        //                 &mut x_stepper,
        //                 &mut y_stepper,
        //                 &mut z_stepper,
        //                 Vector3D::new(
        //                     Distance::from_mm(x.unwrap()),
        //                     Distance::from_mm(y.unwrap()),
        //                     Distance::from_mm(z.unwrap()),
        //                 ),
        //                 StepperSpeed::from_mm_per_second(f.unwrap()),
        //             )
        //             .await
        //             .unwrap_or_else(|_| info!("Cannot perform move"))
        //         }
        //         _ => info!("implement movement"),
        //     },
        //     None => (),
        // };

        Timer::after(Duration::from_millis(1)).await;
    }
}
