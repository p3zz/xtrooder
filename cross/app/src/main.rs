#![no_std]
#![no_main]

use core::cell::RefCell;
use core::fmt::Write;
use core::str::FromStr;

use app::config::{PrinterConfig, StepperConfig};
use app::ext::{peripherals_init, PwmTimer, XStepPin};
use app::{init_pin, init_stepper};
use app::fan::FanController;
use app::hotend::{controller::Hotend, heater::Heater, thermistor, thermistor::Thermistor};
// use app::config::{peripherals_init, PrinterConfig};
use app::utils::stopwatch::Clock;
use defmt::{error, info};
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::adc::AdcChannel;
use embassy_stm32::mode::{Async, Blocking};
use embassy_stm32::peripherals::{
    ADC1, ADC2, DMA1_CH2, DMA1_CH3, PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PB0, PB1, PB2, PB3, PB4, PB5, PB9, PC12, PC8, SPI1, TIM3, TIM4, TIM8, UART4
};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::timer::GeneralInstance4Channel;
use embassy_stm32::usart::{self, Uart, UartRx, UartTx};
use embassy_stm32::Config;
use embassy_stm32::{
    adc::Resolution,
    bind_interrupts,
    gpio::{Level, Output, OutputType, Speed as PinSpeed},
    time::hz,
    timer::{
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm},
        Channel as TimerChannel,
    },
};
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::{Delay, Duration, Timer};
use embedded_sdmmc::{SdCard, VolumeIdx, VolumeManager};
use heapless::{String, Vec};
use math::{
    measurements::{Resistance, Temperature},
    DistanceUnit,
};
use parser::gcode::{GCodeParser, GCommand};
use static_cell::StaticCell;
use stepper::stepper::{Attached, StatefulOutputPin, Stepper, StepperAttachment, StepperOptions};
use stepper::{planner, TimerTrait};
use stepper::planner::Planner;
use {defmt_rtt as _, panic_probe as _};

// https://dev.to/theembeddedrustacean/sharing-data-among-tasks-in-rust-embassy-synchronization-primitives-59hk
const MAX_MESSAGE_LEN: usize = 255;
static COMMAND_DISPATCHER_CHANNEL: Channel<ThreadModeRawMutex, String<MAX_MESSAGE_LEN>, 8> =
    Channel::new();
static SD_CARD_CHANNEL: Channel<ThreadModeRawMutex, GCommand, 8> = Channel::new();
static HOTEND_CHANNEL: Channel<ThreadModeRawMutex, GCommand, 8> = Channel::new();
static HEATBED_CHANNEL: Channel<ThreadModeRawMutex, GCommand, 8> = Channel::new();
static PLANNER_CHANNEL: Channel<ThreadModeRawMutex, GCommand, 8> = Channel::new();
static FEEDBACK_CHANNEL: Channel<ThreadModeRawMutex, String<MAX_MESSAGE_LEN>, 8> = Channel::new();

static UART_RX: Mutex<ThreadModeRawMutex, Option<UartRx<'_, Async>>> = Mutex::new(None);
static UART_TX: Mutex<ThreadModeRawMutex, Option<UartTx<'_, Async>>> = Mutex::new(None);
static PLANNER: Option<Planner<StepperPin, StepperTimer>> = None;
static PMW: Mutex<ThreadModeRawMutex, Option<SimplePwm<'_, PwmTimer>>> = Mutex::new(None);

#[link_section = ".ram_d3"]
static UART_RX_DMA_BUF: StaticCell<[u8; MAX_MESSAGE_LEN]> = StaticCell::new();
#[link_section = ".ram_d3"]
static UART_TX_DMA_BUF: StaticCell<[u8; MAX_MESSAGE_LEN]> = StaticCell::new();
#[link_section = ".ram_d3"]
static HOTEND_DMA_BUF: StaticCell<thermistor::DmaBufType> = StaticCell::new();
#[link_section = ".ram_d3"]
static HEATBED_DMA_BUF: StaticCell<thermistor::DmaBufType> = StaticCell::new();

struct MyStruct<T, P>{
    pin1: T,
    pin2: P,
}

bind_interrupts!(struct Irqs {
    UART4 => usart::InterruptHandler<UART4>;
});

struct StepperPin<'a> {
    pin: Output<'a>,
}

impl StatefulOutputPin for StepperPin<'_> {
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

struct StepperTimer {}

impl TimerTrait for StepperTimer {
    async fn after(duration: core::time::Duration) {
        let duration = embassy_time::Duration::from_micros(duration.as_micros() as u64);
        Timer::after(duration).await
    }
}

#[embassy_executor::task]
async fn input_handler() {
    let mut msg: String<MAX_MESSAGE_LEN> = String::new();
    let tmp = UART_RX_DMA_BUF.init([0u8; MAX_MESSAGE_LEN]);
    let mut rx = UART_RX.lock().await;
    let rx = rx.as_mut().expect("UART RX not initialized");

    info!("Starting input handler loop");

    loop {
        if let Ok(n) = rx.read_until_idle(tmp).await {
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
        } else {
            error!("Cannot read from UART");
        }
    }
}

#[embassy_executor::task]
async fn output_handler() {
    let mut clock = Clock::new();
    let tmp = UART_TX_DMA_BUF.init([0u8; MAX_MESSAGE_LEN]);
    let mut tx = UART_TX.lock().await;
    let tx = tx.as_mut().expect("UART TX not initialized");
    let dt = Duration::from_millis(100);
    let mut report: String<MAX_MESSAGE_LEN> = String::new();

    clock.start();

    loop {
        // retrieve the channel content and copy the message inside the shared memory of DMA to send t
        // over UART
        let msg = FEEDBACK_CHANNEL.receive().await;
        core::write!(&mut report, "[{}] {}", clock.measure(), &msg).unwrap();
        let mut len = 0;
        for (i, b) in msg.into_bytes().iter().enumerate() {
            tmp[i] = *b;
            len += 1;
        }
        match tx.write(&tmp[0..len]).await {
            Ok(_) => (),
            Err(_) => error!("Cannot write to UART"),
        };

        Timer::after(dt).await;
    }
}

#[embassy_executor::task]
async fn command_dispatcher_task() {
    let mut parser = GCodeParser::new();
    let dt = Duration::from_millis(500);
    let mut response: String<MAX_MESSAGE_LEN> = String::new();
    info!("Starting command dispatcher loop");

    loop {
        let msg = COMMAND_DISPATCHER_CHANNEL.receive().await;
        info!("[COMMAND DISPATCHER] received message {}", msg.as_str());
        core::write!(&mut response, "Hello DMA World {}!\r\n", msg.as_str()).unwrap();
        FEEDBACK_CHANNEL.send(response.clone()).await;
        response.clear();
        if let Some(cmd) = parser.parse(msg.as_str()) {
            // info!("[COMMAND DISPATCHER] {}", cmd);
            match cmd {
                // every movement command is redirected to the planner channel
                GCommand::G0 { .. }
                | GCommand::G1 { .. }
                | GCommand::G2 { .. }
                | GCommand::G3 { .. }
                | GCommand::G4 { .. }
                | GCommand::G90
                | GCommand::G91
                | GCommand::M114 => {
                    PLANNER_CHANNEL.send(cmd).await;
                }
                GCommand::G20 => parser.set_distance_unit(DistanceUnit::Inch),
                GCommand::G21 => parser.set_distance_unit(DistanceUnit::Millimeter),
                // hotend target temperature is used to update the target temperature of the hotend task
                GCommand::M104 { .. } | GCommand::M106 { .. } => {
                    HOTEND_CHANNEL.send(cmd).await;
                }
                GCommand::M105 { .. } => {
                    HOTEND_CHANNEL.send(cmd.clone()).await;
                    HEATBED_CHANNEL.send(cmd.clone()).await;
                }
                // heatbed target temperature is used to update the target temperature of the hotend task
                GCommand::M140 { .. } => {
                    HOTEND_CHANNEL.send(cmd).await;
                }
                GCommand::M149 { u } => {
                    parser.set_temperature_unit(u);
                }
                GCommand::M155 { .. } => {
                    HOTEND_CHANNEL.send(cmd.clone()).await;
                    HEATBED_CHANNEL.send(cmd.clone()).await;
                }
                GCommand::M20
                | GCommand::M21
                | GCommand::M22
                | GCommand::M23 { .. }
                | GCommand::M24 { .. }
                | GCommand::M25
                | GCommand::M31 => {
                    SD_CARD_CHANNEL.send(cmd).await;
                }
                _ => error!("[COMMAND DISPATCHER] command not handler"),
            }
        } else {
            error!("[COMMAND DISPATCHER] Invalid command");
        }

        Timer::after(dt).await;
    }
}

// https://dev.to/apollolabsbin/embedded-rust-embassy-analog-sensing-with-adcs-1e2n
// #[embassy_executor::task]
// async fn hotend_handler(
//     adc_peri: ADC1,
//     dma_peri: DMA1_CH2,
//     read_pin: PA3,
//     heater_tim: TIM4,
//     heater_out_pin: PB9,
//     fan_time: TIM3,
//     fan_out_pin: PA7,
// ) {
//     // TODO adjust the period using the dt of the loop
//     let mut temperature_report_dt: Option<Duration> = None;
//     let readings = HOTEND_DMA_BUF.init([0u16; 1]);

//     let thermistor = Thermistor::new(
//         adc_peri,
//         dma_peri,
//         read_pin.degrade_adc(),
//         Resolution::BITS12,
//         Resistance::from_ohms(100_000.0),
//         Resistance::from_ohms(10_000.0),
//         Temperature::from_kelvin(3950.0),

//         readings,
//     );

//     let heater_out = SimplePwm::new(
//         heater_tim,
//         None,
//         None,
//         None,
//         Some(PwmPin::new_ch4(heater_out_pin, OutputType::PushPull)),
//         hz(1),
//         CountingMode::EdgeAlignedUp,
//     );

//     let fan_out = SimplePwm::new(
//         fan_time,
//         None,
//         Some(PwmPin::new_ch2(fan_out_pin, OutputType::PushPull)),
//         None,
//         None,
//         hz(1),
//         CountingMode::EdgeAlignedUp,
//     );

//     let mut fan_controller = FanController::new(fan_out, TimerChannel::Ch2, 10f64);
//     let heater = Heater::new(heater_out, TimerChannel::Ch4);
//     let mut hotend = Hotend::new(heater, thermistor);

//     let dt = Duration::from_millis(100);
//     let mut counter = Duration::from_secs(0);
//     let mut report: String<MAX_MESSAGE_LEN> = String::new();

//     loop {
//         // temperature report period must be a multiple of the loop delay
//         if temperature_report_dt.is_some()
//             && counter.as_millis() % temperature_report_dt.unwrap().as_millis() == 0
//         {
//             let temp = hotend.read_temperature().await;
//             report.clear();
//             core::write!(&mut report, "Hotend temperature: {}", temp).unwrap();
//             FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
//             counter = Duration::from_secs(0);
//         }
//         if let Ok(cmd) = HOTEND_CHANNEL.try_receive() {
//             match cmd {
//                 GCommand::M104 { s } => {
//                     info!("[HOTEND HANDLER] Target temperature: {}", s.as_celsius());
//                     hotend.set_temperature(s);
//                 }
//                 GCommand::M105 => {
//                     let temp = hotend.read_temperature().await;
//                     report.clear();
//                     core::write!(&mut report, "Hotend temperature: {}", temp).unwrap();
//                     FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(())
//                 }
//                 GCommand::M106 { s } => {
//                     let multiplier = f64::from(255) / f64::from(s);
//                     let speed = fan_controller.get_max_speed() * multiplier;
//                     fan_controller.set_speed(speed);
//                     info!("[HOTEND HANDLER] Fan speed: {} revs/s", speed);
//                 }
//                 GCommand::M155 { s } => {
//                     let duration = Duration::from_millis(s.as_millis() as u64);
//                     temperature_report_dt.replace(duration);
//                 }
//                 _ => (),
//             }
//         }

//         if let Ok(duty_cycle) = hotend.update(dt).await {
//             info!("[HEATBED] duty cycle: {}", duty_cycle);
//         };

//         Timer::after(dt).await;

//         if counter.checked_add(dt).is_none() {
//             counter = Duration::from_secs(0);
//         }
//     }
// }

// // https://dev.to/apollolabsbin/embedded-rust-embassy-analog-sensing-with-adcs-1e2n
// // TODO test with HEATBED_TARGET_TEMPERATURE
// #[embassy_executor::task]
// async fn heatbed_handler(
//     adc_peri: ADC2,
//     dma_peri: DMA1_CH3,
//     read_pin: PA2,
//     heater_tim: TIM8,
//     heater_out_pin: PC8,
// ) {
//     // TODO adjust the period using the dt of the loop
//     let mut temperature_report_dt: Option<Duration> = None;
//     let readings = HEATBED_DMA_BUF.init([0u16; 1]);

//     let thermistor = Thermistor::new(
//         adc_peri,
//         dma_peri,
//         read_pin.degrade_adc(),
//         Resolution::BITS12,
//         Resistance::from_ohms(100_000.0),
//         Resistance::from_ohms(10_000.0),
//         Temperature::from_kelvin(3950.0),
//         readings,
//     );

//     let heater_out = SimplePwm::new(
//         heater_tim,
//         None,
//         None,
//         Some(PwmPin::new_ch3(heater_out_pin, OutputType::PushPull)),
//         None,
//         hz(1),
//         CountingMode::EdgeAlignedUp,
//     );
//     let heater = Heater::new(heater_out, TimerChannel::Ch4);
//     let mut heatbed = Hotend::new(heater, thermistor);

//     let dt = Duration::from_millis(100);
//     let mut counter = Duration::from_secs(0);
//     let mut report: String<MAX_MESSAGE_LEN> = String::new();

//     loop {
//         // temperature report period must be a multiple of the loop delay
//         if temperature_report_dt.is_some()
//             && counter.as_millis() % temperature_report_dt.unwrap().as_millis() == 0
//         {
//             let temp = heatbed.read_temperature().await;
//             report.clear();
//             core::write!(&mut report, "Heatbed temperature: {}", temp).unwrap();
//             FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
//             counter = Duration::from_secs(0);
//         }

//         if let Ok(cmd) = HEATBED_CHANNEL.try_receive() {
//             match cmd {
//                 GCommand::M140 { s } => heatbed.set_temperature(s),
//                 GCommand::M105 => {
//                     let temp = heatbed.read_temperature().await;
//                     report.clear();
//                     core::write!(&mut report, "Heatbed temperature: {}", temp).unwrap();
//                     FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(())
//                 }
//                 GCommand::M155 { s } => {
//                     let duration = Duration::from_millis(s.as_millis() as u64);
//                     temperature_report_dt.replace(duration);
//                 }
//                 _ => (),
//             }
//         };

//         if let Ok(duty_cycle) = heatbed.update(dt).await {
//             info!("[HEATBED] duty cycle: {}", duty_cycle);
//         };

//         Timer::after(dt).await;
//         if counter.checked_add(dt).is_none() {
//             counter = Duration::from_secs(0);
//         }
//     }
// }

// #[embassy_executor::task]
// async fn sdcard_handler(spi_peri: SPI1, clk: PB3, mosi: PB5, miso: PB4, cs: PC12) {
//     static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<'static, Blocking>>>> = StaticCell::new();
//     let spi = spi::Spi::new_blocking(spi_peri, clk, mosi, miso, Default::default());
//     let spi_bus = NoopMutex::new(RefCell::new(spi));
//     let spi_bus = SPI_BUS.init(spi_bus);

//     // Device 1, using embedded-hal compatible driver for ST7735 LCD display
//     let cs_pin = Output::new(cs, Level::High, embassy_stm32::gpio::Speed::Low);

//     let spi = SpiDevice::new(spi_bus, cs_pin);
//     let sdcard = SdCard::new(spi, Delay);
//     let clock = Clock::new();
//     let mut volume_manager = VolumeManager::new(sdcard, clock);
//     let mut working_dir = None;
//     let mut working_file = None;
//     let mut working_volume = None;
//     let mut running = false;
//     let mut msg: String<MAX_MESSAGE_LEN> = String::new();
//     let mut tmp: [u8; MAX_MESSAGE_LEN] = [0u8; MAX_MESSAGE_LEN];
//     let mut clock = Clock::new();
//     let mut report: String<MAX_MESSAGE_LEN> = String::new();

//     let dt = Duration::from_millis(500);
//     loop {
//         if let Ok(cmd) = SD_CARD_CHANNEL.try_receive() {
//             // info!("[SDCARD] command received: {}", cmd);
//             match cmd {
//                 GCommand::M20 => {
//                     let dir = working_dir.expect("Working directory not set");
//                     let mut msg: String<MAX_MESSAGE_LEN> =
//                         String::from_str("Begin file list").unwrap();
//                     volume_manager
//                         .iterate_dir(dir, |d| {
//                             let name_vec: Vec<u8, 16> =
//                                 Vec::from_slice(d.clone().name.base_name()).unwrap();
//                             let name = String::from_utf8(name_vec).unwrap();
//                             msg.push_str(name.as_str()).unwrap();
//                             msg.push('\n').unwrap();
//                         })
//                         .expect("Error while listing files");
//                     msg.push_str("End file list").unwrap();
//                     // TODO send message to UART
//                 }
//                 GCommand::M21 => {
//                     working_volume = match volume_manager.open_raw_volume(VolumeIdx(0)) {
//                         Ok(v) => Some(v),
//                         Err(_) => defmt::panic!("Cannot find module"),
//                     };
//                     working_dir = match volume_manager.open_root_dir(working_volume.unwrap()) {
//                         Ok(d) => Some(d),
//                         Err(_) => defmt::panic!("Cannot open root dir"),
//                     };
//                     info!("Directory open");
//                 }
//                 GCommand::M22 => {
//                     if working_file.is_some() {
//                         volume_manager.close_file(working_file.unwrap()).unwrap();
//                         info!("File closed");
//                     }
//                     if working_dir.is_some() {
//                         volume_manager.close_dir(working_dir.unwrap()).unwrap();
//                         info!("Directory closed");
//                     }
//                     if working_volume.is_some() {
//                         volume_manager
//                             .close_volume(working_volume.unwrap())
//                             .unwrap();
//                         info!("Volume closed");
//                     }
//                 }
//                 GCommand::M23 { filename } => {
//                     let dir = working_dir.expect("Working directory not set");
//                     working_file = match volume_manager.open_file_in_dir(
//                         dir,
//                         filename.as_str(),
//                         embedded_sdmmc::Mode::ReadOnly,
//                     ) {
//                         Ok(f) => Some(f),
//                         Err(_) => defmt::panic!("File not found"),
//                     };
//                     info!("Working file set");
//                 }
//                 // ignore the parameters of M24, just start/resume the print
//                 GCommand::M24 { .. } => {
//                     if !running {
//                         clock.start();
//                         running = true;
//                     }
//                 }
//                 GCommand::M25 => {
//                     if running {
//                         clock.stop();
//                         running = false;
//                     }
//                 }
//                 GCommand::M31 => {
//                     report.clear();
//                     core::write!(&mut report, "Time elapsed: {}", clock.measure()).unwrap();
//                     FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
//                 }
//                 _ => todo!(),
//             }
//         }

//         if running && working_file.is_some() {
//             if let Ok(n) = volume_manager.read(working_file.unwrap(), &mut tmp) {
//                 for b in 0..n {
//                     if tmp[b] == b'\n' {
//                         COMMAND_DISPATCHER_CHANNEL.send(msg.clone()).await;
//                         info!("[INPUT_HANDLER] {}", msg.as_str());
//                         msg.clear();
//                     } else {
//                         // TODO handle buffer overflow
//                         msg.push(tmp[b].into()).unwrap();
//                     }
//                 }
//                 tmp.fill(0u8);
//             } else {
//                 error!("Cannot read from SD-card");
//             }
//         }
//         Timer::after(dt).await;
//     }
// }

#[embassy_executor::task]
async fn planner_handler() {
    let mut report: String<MAX_MESSAGE_LEN> = String::new();
    let mut debug = false;

    // let x_step_pin =  unsafe {
    //     CONFIG.unwrap().steppers.x.step_pin
    // };

    // let cfg = CONFIG.unwrap();

    // let x_step_pin = cfg.steppers.x.step_pin;
    // let x_dir_pin = cfg.steppers.x.dir_pin;
    // let x_options = StepperOptions::default();
    // let x_attachment = StepperAttachment::default();

    // let x_stepper = init_stepper!(x_step_pin, x_dir_pin, x_options, x_attachment);

    // let y_step_pin = config.steppers.y.step_pin;
    // let y_dir_pin = config.steppers.y.dir_pin;
    // let y_options = StepperOptions::default();
    // let y_attachment = StepperAttachment::default();

    // let y_stepper = init_stepper!(y_step_pin, y_dir_pin, y_options, y_attachment);

    // let z_step_pin = config.steppers.z.step_pin;
    // let z_dir_pin = config.steppers.z.dir_pin;
    // let z_options = StepperOptions::default();
    // let z_attachment = StepperAttachment::default();

    // let z_stepper = init_stepper!(z_step_pin, z_dir_pin, z_options, z_attachment);

    // let e_step_pin = config.steppers.e.step_pin;
    // let e_dir_pin = config.steppers.e.dir_pin;
    // let e_options = StepperOptions::default();
    // let e_attachment = StepperAttachment::default();

    // let e_stepper = init_stepper!(e_step_pin, e_dir_pin, e_options, e_attachment);

    // let planner: Planner<StepperPin<'_>, StepperTimer> = Planner::new(x_stepper, y_stepper, z_stepper, e_stepper);


    // let mut planner = PLANNER.lock().await;
    // let planner = planner.as_mut().expect("Planner not initialized");

    // let dt = Duration::from_millis(500);

    // loop {
    //     let cmd = PLANNER_CHANNEL.receive().await;
    //     // info!("[PLANNER HANDLER] {}", cmd);
    //     match cmd {
    //         GCommand::G0 { .. }
    //         | GCommand::G1 { .. }
    //         | GCommand::G2 { .. }
    //         | GCommand::G3 { .. }
    //         | GCommand::G4 { .. }
    //         | GCommand::G10
    //         | GCommand::G11
    //         | GCommand::G28
    //         | GCommand::G90
    //         | GCommand::G91
    //         | GCommand::M207 { .. }  
    //         | GCommand::M208 { .. }  => {
    //             let duration = planner.execute(cmd.clone()).await.expect("Planner error");
    //             if debug {
    //                 match cmd {
    //                     GCommand::G0 { .. } => {
    //                         let x = planner.get_x_position();
    //                         let y = planner.get_x_position();
    //                         let z = planner.get_x_position();
    //                         let t = duration.unwrap();
    //                         let res = GCommand::D0 { x, y, z, t };
    //                         report.clear();
    //                         write!(&mut report, "{}", &res).unwrap();
    //                         FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
    //                     }
    //                     GCommand::G1 { .. } => {
    //                         let x = planner.get_x_position();
    //                         let y = planner.get_x_position();
    //                         let z = planner.get_x_position();
    //                         let e = planner.get_e_position();
    //                         let t = duration.unwrap();
    //                         let res = GCommand::D1 { x, y, z, e, t };
    //                         report.clear();
    //                         write!(&mut report, "{}", &res).unwrap();
    //                         FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
    //                     }
    //                     _ => todo!(),
    //                 }
    //             }
    //         }
    //         GCommand::M114 => {
    //             report.clear();
    //             write!(
    //                 &mut report,
    //                 "Head position: [X:{}] [Y:{}] [Z:{}] [E:{}]",
    //                 planner.get_x_position(),
    //                 planner.get_y_position(),
    //                 planner.get_z_position(),
    //                 planner.get_e_position(),
    //             )
    //             .unwrap();
    //             FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
    //         },
    //         GCommand::D114 => {
    //             debug = true;
    //         }
    //         GCommand::D115 => {
    //             debug = false;
    //         }
    //         _ => error!("[PLANNER HANDLER] command not handled"),
    //     }
    //     info!("[PLANNER HANDLER] Move completed");
    //     Timer::after(dt).await;
    // }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());
    
    let config = peripherals_init(p);
    
    // let x_step_pin = config.steppers.x.step_pin;
    // let x_dir_pin = config.steppers.x.dir_pin;
    // let x_options = StepperOptions::default();
    // let x_attachment = StepperAttachment::default();

    // let x_stepper = init_stepper!(x_step_pin, x_dir_pin, x_options, x_attachment);

    // let y_step_pin = config.steppers.y.step_pin;
    // let y_dir_pin = config.steppers.y.dir_pin;
    // let y_options = StepperOptions::default();
    // let y_attachment = StepperAttachment::default();

    // let y_stepper = init_stepper!(y_step_pin, y_dir_pin, y_options, y_attachment);

    // let z_step_pin = config.steppers.z.step_pin;
    // let z_dir_pin = config.steppers.z.dir_pin;
    // let z_options = StepperOptions::default();
    // let z_attachment = StepperAttachment::default();

    // let z_stepper = init_stepper!(z_step_pin, z_dir_pin, z_options, z_attachment);

    // let e_step_pin = config.steppers.e.step_pin;
    // let e_dir_pin = config.steppers.e.dir_pin;
    // let e_options = StepperOptions::default();
    // let e_attachment = StepperAttachment::default();

    // let e_stepper = init_stepper!(e_step_pin, e_dir_pin, e_options, e_attachment);

    // let planner: Planner<StepperPin<'_>, StepperTimer> = Planner::new(x_stepper, y_stepper, z_stepper, e_stepper);

    // {
    //     let mut p = PLANNER.lock().await;
    //     p.replace(planner);
    // }

    // let heater_out = SimplePwm::new(
    //     heater_tim,
    //     None,
    //     None,
    //     Some(PwmPin::new_ch3(heater_out_pin, OutputType::PushPull)),
    //     None,
    //     hz(1),
    //     CountingMode::EdgeAlignedUp,
    // );

    // // TODO check this configuration. It's in the embassy stm32 examples of ADC. Not so sure why it's needed but without this the
    // // program won't run
    // {
    //     use embassy_stm32::rcc::*;
    //     config.rcc.hsi = Some(HSIPrescaler::DIV1);
    //     config.rcc.csi = true;
    //     config.rcc.pll1 = Some(Pll {
    //         source: PllSource::HSI,
    //         prediv: PllPreDiv::DIV4,
    //         mul: PllMul::MUL50,
    //         divp: Some(PllDiv::DIV2),
    //         divq: Some(PllDiv::DIV8), // SPI1 cksel defaults to pll1_q
    //         divr: None,
    //     });
    //     config.rcc.pll2 = Some(Pll {
    //         source: PllSource::HSI,
    //         prediv: PllPreDiv::DIV4,
    //         mul: PllMul::MUL50,
    //         divp: Some(PllDiv::DIV8), // 100mhz
    //         divq: None,
    //         divr: None,
    //     });
    //     config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
    //     config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
    //     config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
    //     config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
    //     config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
    //     config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
    //     config.rcc.voltage_scale = VoltageScale::Scale1;
    //     config.rcc.mux.adcsel = mux::Adcsel::PLL2_P;
    // }
    // let mut uart_config = embassy_stm32::usart::Config::default();
    // uart_config.baudrate = 19200;

    // let uart = Uart::new(
    //     config.peri, config.rx_pin, config.tx_pin, Irqs, config.tx_dma, config.rx_dma, uart_config,
    // )
    // .unwrap();
    // let (tx, rx) = uart.split();

    // {
    //     let mut uart_rx = UART_RX.lock().await;
    //     uart_rx.replace(rx);
    // }

    // {
    //     let mut uart_tx = UART_TX.lock().await;
    //     uart_tx.replace(tx);
    // }

    // spawner.spawn(input_handler()).unwrap();

    // spawner.spawn(output_handler()).unwrap();

    // spawner.spawn(command_dispatcher_task()).unwrap();

    // spawner
    //     .spawn(hotend_handler(
    //         p.ADC1, p.DMA1_CH2, p.PA3, p.TIM4, p.PB9, p.TIM3, p.PA7,
    //     ))
    //     .unwrap();

    // spawner
    //     .spawn(heatbed_handler(p.ADC2, p.DMA1_CH3, p.PA2, p.TIM8, p.PC8))
    //     .unwrap();

    // // spawner
    // //     .spawn(planner_handler(
    // //         p.PA0, p.PB0, p.PA6, p.PB1, p.PA5, p.PB2, p.PA1, p.PB4,
    // //     ))
    // //     .unwrap();

    // spawner
    //     .spawn(sdcard_handler(p.SPI1, p.PB3, p.PB5, p.PB4, p.PC12))
    //     .unwrap();

    loop {
        info!("[MAIN LOOP] alive");
        Timer::after(Duration::from_secs(1)).await;
    }
}
