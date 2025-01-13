#![no_std]
#![no_main]

use core::cell::RefCell;
use core::fmt::Write;
use core::str::FromStr;

use app::config::{
    EndstopsConfig, FanConfig, MotionConfig, SdCardConfig, SteppersConfig, ThermalActuatorConfig,
};
use app::ext::{
    peripherals_init, AdcDma, AdcPeripheral, EDirPin, EStepPin, HeatbedAdcInputPin,
    HotendAdcInputPin, PwmTimer, SdCardSpiCsPin, SdCardSpiMisoPin, SdCardSpiMosiPin,
    SdCardSpiPeripheral, SdCardSpiTimer, XDirPin, XEndstopExti, XEndstopPin, XStepPin, YDirPin,
    YEndstopExti, YEndstopPin, YStepPin, ZDirPin, ZEndstopExti, ZEndstopPin, ZStepPin,
};
use app::{init_input_pin, init_output_pin, init_stepper, timer_channel, PrinterEvent};
use app::{task_write, Clock, ExtiInputPinWrapper, OutputPinWrapper, StepperTimer};
use app::{AdcWrapper, ResolutionWrapper, SimplePwmWrapper};
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_executor::{InterruptExecutor, Spawner};
use embassy_futures::join::join;
use embassy_stm32::adc::{Adc, AdcChannel, SampleTime};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{OutputType, Pull, Speed};
use embassy_stm32::mode::{Async, Blocking};
use embassy_stm32::peripherals::UART4;
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::{hz, khz};
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::usart::{self, Uart, UartRx, UartTx};
use embassy_stm32::Config;
use embassy_stm32::{
    adc::Resolution,
    bind_interrupts,
    gpio::{Level, Output},
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::priority_channel::{PriorityChannel, Max};
use embassy_sync::pubsub::PubSubChannel;
use embassy_sync::signal::Signal;
use embassy_sync::watch::Watch;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::{Delay, Duration, Timer};
use embedded_sdmmc::{SdCard, VolumeIdx, VolumeManager};
use fan::FanController;
use heapless::{String, Vec};
use math::{measurements::Temperature, DistanceUnit};
use parser::gcode::{GCodeParser, GCommand};
use static_cell::{ConstStaticCell, StaticCell};
use stepper::planner::Planner;
use stepper::stepper::{StepperAttachment, StepperOptions};
use thermal_actuator::{
    controller::ThermalActuator, heater::Heater, thermistor, thermistor::Thermistor,
};
use common::PwmBase;
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::{InterruptExt, Priority};
use {defmt_rtt as _, panic_probe as _};

#[cfg(feature = "defmt-log")]
use defmt::{error, info};

static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();

#[interrupt]
unsafe fn TIM2() {
    EXECUTOR_HIGH.on_interrupt()
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq)]
enum TaskMessagePriority{
    Low,
    Medium,
    High
}

#[derive(Clone, PartialEq, PartialOrd, Eq)]
struct TaskMessage{
    msg: String<MAX_MESSAGE_LEN>,
    priority: TaskMessagePriority
}

impl Ord for TaskMessage{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        if self.priority > other.priority{
            core::cmp::Ordering::Greater
        }
        else if self.priority < other.priority{
            core::cmp::Ordering::Less
        }
        else{
            core::cmp::Ordering::Equal
        }
    }
    
    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        core::cmp::max_by(self, other, Ord::cmp)
    }
    
    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        core::cmp::min_by(self, other, Ord::cmp)
    }
    
    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
    {
        assert!(min <= max);
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}

#[derive(Clone, Copy)]
enum TaskId{
    Input,
    Output,
    CommandDispatcher,
    Hotend,
    Heatbed,
    SdCard,
    Planner
}

impl From<TaskId> for u8{
    fn from(value: TaskId) -> Self {
        match value{
            TaskId::Input => 0,
            TaskId::Output => 1,
            TaskId::CommandDispatcher => 2,
            TaskId::Hotend => 3,
            TaskId::Heatbed => 4,
            TaskId::SdCard => 5,
            TaskId::Planner => 6,
        }
    }
}

#[derive(Clone)]
struct TaskGCommand{
    cmd: GCommand,
    destination: u8
}

// https://dev.to/theembeddedrustacean/sharing-data-among-tasks-in-rust-embassy-synchronization-primitives-59hk
const MAX_MESSAGE_LEN: usize = 256;
const COMMAND_DISPATCHER_CHANNEL_LEN: usize = 16;
const FEEDBACK_CHANNEL_LEN: usize = 16;
const EVENT_CHANNEL_CAPACITY: usize = 8;
const EVENT_CHANNEL_SUBSCRIBERS: usize = 7;
const EVENT_CHANNEL_PUBLISHERS: usize = 7;

const HOTEND_LABEL: &'_ str = "HOTEND";
const HEATBED_LABEL: &'_ str = "HEATBED";
const PLANNER_LABEL: &'_ str = "PLANNER";
const SD_CARD_LABEL: &'_ str = "SD-CARD";

static WATCH: Watch<
    CriticalSectionRawMutex,
    TaskGCommand, 
    7
> = Watch::new();

static SIGNAL: Signal<
    CriticalSectionRawMutex,
    TaskId
> = Signal::new();

static COMMAND_DISPATCHER_CHANNEL: PriorityChannel<
    CriticalSectionRawMutex,
    TaskMessage,
    Max,
    COMMAND_DISPATCHER_CHANNEL_LEN,
> = PriorityChannel::new();

static FEEDBACK_CHANNEL: Channel<
    CriticalSectionRawMutex,
    String<MAX_MESSAGE_LEN>,
    FEEDBACK_CHANNEL_LEN,
> = Channel::new();

static EVENT_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    PrinterEvent,
    EVENT_CHANNEL_CAPACITY,
    EVENT_CHANNEL_SUBSCRIBERS,
    EVENT_CHANNEL_PUBLISHERS,
> = PubSubChannel::new();

static UART_RX: Mutex<ThreadModeRawMutex, Option<UartRx<'_, Async>>> = Mutex::new(None);
static UART_TX: Mutex<ThreadModeRawMutex, Option<UartTx<'_, Async>>> = Mutex::new(None);
static PMW: Mutex<ThreadModeRawMutex, Option<SimplePwmWrapper<'_, PwmTimer>>> = Mutex::new(None);
static ADC: Mutex<ThreadModeRawMutex, Option<AdcWrapper<'_, AdcPeripheral, AdcDma>>> =
    Mutex::new(None);

// WARN these buffers are used by the DMA to write/read data through ADC and UART
// it would be useful to find a struct that can wrap these buffers, such as ConstStaticCell,
// which unfortunately panics when a soft reset happens, both via reset button or removing power
// to the board and them repowering.
#[link_section = ".ram_d3"]
static mut UART_RX_DMA_BUF: [u8; MAX_MESSAGE_LEN] =[0u8; MAX_MESSAGE_LEN];
#[link_section = ".ram_d3"]
static mut UART_TX_DMA_BUF: [u8; MAX_MESSAGE_LEN] = [0u8; MAX_MESSAGE_LEN];
#[link_section = ".ram_d3"]
static mut HOTEND_DMA_BUF: thermistor::DmaBufType = [0u16; 1];
#[link_section = ".ram_d3"]
static mut HEATBED_DMA_BUF: thermistor::DmaBufType = [0u16; 1];

bind_interrupts!(struct Irqs {
    UART4 => usart::InterruptHandler<UART4>;
});

#[embassy_executor::task]
async fn input_handler() {
    let mut msg: String<MAX_MESSAGE_LEN> = String::new();
    // SAFETY - UART_RX_DMA_BUF is used only in this task
    let tmp = unsafe { &mut UART_RX_DMA_BUF };
    let mut rx = UART_RX.lock().await;
    let rx = rx.as_mut().expect("UART RX not initialized");
    let dt = Duration::from_millis(50);

    #[cfg(feature = "defmt-log")]
    info!("Starting input handler loop");

    loop {
        match rx.read_until_idle(tmp).await{
            Ok(n) => {
                for b in &tmp[0..n] {
                    if *b == b'\n' {
                        let cmd = TaskMessage{
                            msg: msg.clone(),
                            priority: TaskMessagePriority::High
                        };
                        COMMAND_DISPATCHER_CHANNEL.send(cmd).await;
                        #[cfg(feature = "defmt-log")]
                        info!("[INPUT_HANDLER] {}", msg.as_str());
                        msg.clear();
                    } else if msg.push((*b).into()).is_err() {
                        msg.clear();
                        #[cfg(feature = "defmt-log")]
                        error!("Message too long");
                    }
                }
            }
            Err(e) => {
                #[cfg(feature = "defmt-log")]
                error!("Cannot read from UART: {}", e);
            }
        }
        // Timer::after(dt).await;
    }
}

#[embassy_executor::task]
async fn output_handler() {
    // SAFETY - UART_TX_DMA_BUF is used only in this task
    let tmp = unsafe { &mut UART_TX_DMA_BUF };
    let mut tx = UART_TX.lock().await;
    let tx = tx.as_mut().expect("UART TX not initialized");
    let dt = Duration::from_millis(100);

    loop {
        // retrieve the channel content and copy the message inside the shared memory of DMA to send t
        // over UART
        let msg = FEEDBACK_CHANNEL.receive().await;
        let len = msg.as_bytes().len().min(MAX_MESSAGE_LEN);
        tmp[0..len].copy_from_slice(&msg.as_bytes()[0..len]);
        match tx.write(&tmp[0..len]).await {
            Ok(_) => (),
            Err(_) => {
                #[cfg(feature = "defmt-log")]
                error!("Cannot write to UART")
            }
        }
        Timer::after(dt).await;
    }
}

#[embassy_executor::task]
async fn command_dispatcher_task() {
    let mut parser = GCodeParser::new();
    let dt = Duration::from_millis(50);
    let command_receiver = COMMAND_DISPATCHER_CHANNEL.receiver();
    let watch_sender = WATCH.sender();

    #[cfg(feature = "defmt-log")]
    info!("Starting command dispatcher loop");

    loop {
        let msg = command_receiver.receive().await;
        #[cfg(feature = "defmt-log")]
        info!("[COMMAND DISPATCHER] received message {}", msg.msg.as_str());
        let mut destination = 0u8;
        if let Some(cmd) = parser.parse(&msg.msg) {
            match cmd {
                GCommand::G20 => parser.set_distance_unit(DistanceUnit::Inch),
                GCommand::G21 => parser.set_distance_unit(DistanceUnit::Millimeter),
                GCommand::M149 { u } => parser.set_temperature_unit(u),
                GCommand::G0 { .. }
                | GCommand::G1 { .. }
                | GCommand::G2 { .. }
                | GCommand::G3 { .. }
                | GCommand::G4 { .. }
                | GCommand::G10
                | GCommand::G11
                | GCommand::G28 { .. }
                | GCommand::G90
                | GCommand::G91
                | GCommand::G92 { .. }
                | GCommand::M207 { .. }
                | GCommand::M208 { .. }
                | GCommand::M220 { .. } => {
                    destination = 1u8 << u8::from(TaskId::Planner);
                }
                GCommand::M104 { .. }
                | GCommand::M106 { .. } => {
                    destination = 1u8 << u8::from(TaskId::Hotend);
                }
                GCommand::M105 { .. } | GCommand::M155 { .. } => {
                    destination = 1u8 << u8::from(TaskId::Hotend) | 1u8 << u8::from(TaskId::Heatbed);
                }
                GCommand::M140 { .. } => {
                    destination = 1u8 << u8::from(TaskId::Heatbed);
                }
                | GCommand::M20
                | GCommand::M21
                | GCommand::M22
                | GCommand::M23 { .. }
                | GCommand::M24 { .. }
                | GCommand::M25
                | GCommand::M31
                | GCommand::M524 => {
                    destination = 1u8 << u8::from(TaskId::SdCard);
                }
                _ => {
                    #[cfg(feature = "defmt-log")]
                    error!("[COMMAND DISPATCHER] command not handler")
                }
            }
            let task_command = TaskGCommand{
                cmd,
                destination
            };
            watch_sender.send(task_command);
            // wait for tasks response before proceeding to parse the next command
            let mut res = 0u8;
            while res & destination != destination {
                let s = SIGNAL.wait().await;
                res |= 1u8 << u8::from(s);
                #[cfg(feature = "defmt-log")]
                info!("Signal received from {}", u8::from(s));
            }
            #[cfg(feature = "defmt-log")]
            info!("Every response has been received");
        } else {
            #[cfg(feature = "defmt-log")]
            error!("[COMMAND DISPATCHER] Invalid command");
        }

        // Timer::after(dt).await;
    }
}

// https://dev.to/apollolabsbin/embedded-rust-embassy-analog-sensing-with-adcs-1e2n
#[embassy_executor::task]
async fn hotend_handler(
    config: ThermalActuatorConfig<HotendAdcInputPin>,
    fan_config: FanConfig
) {
    // SAFETY - HOTEND_DMA_BUF is used only in this task
    let readings = unsafe { &mut HOTEND_DMA_BUF };

    let thermistor: Thermistor<'_, AdcWrapper<_, _>> = Thermistor::new(
        config.thermistor.input.degrade_adc(),
        readings,
        config.thermistor.options,
    );

    let channel = timer_channel!(fan_config.pwm.channel).expect("Invalid timer channel");
    let mut fan_controller = FanController::new(channel, fan_config.max_speed);

    let channel = timer_channel!(config.heater.pwm.channel).expect("Invalid timer channel");
    let heater = Heater::new(channel, config.heater.pid);
    let mut hotend = ThermalActuator::new(heater, thermistor);

    // TODO adjust the period using the dt of the loop
    let mut temperature_report_dt: Option<Duration> = None;
    let dt = Duration::from_millis(100);
    let mut counter = Duration::from_secs(0);
    let mut report: String<MAX_MESSAGE_LEN> = String::new();
    let mut event_channel_subscriber = EVENT_CHANNEL
        .subscriber()
        .expect("Cannot retrieve error subscriber");
    let event_channel_publisher = EVENT_CHANNEL
        .publisher()
        .expect("Cannot retrieve error subscriber");
    let mut watch_receiver = WATCH.receiver().expect("Cannot retrieve receiver");
    let mut last_temperature: Option<Temperature> = None;

    loop {

        {
            let mut pwm = PMW.lock().await;
            let pwm = pwm.as_mut().expect("PWM not initialized");
            let mut adc = ADC.lock().await;
            let adc = adc.as_mut().expect("ADC not initialized");
            let data = hotend.update(dt.into(), pwm, adc).await;
            last_temperature.replace(data.0);
            // #[cfg(feature = "defmt-log")]
            // info!("[HOTEND] Temperature: {}\tDuty cycle: {}", data.0.as_celsius(), data.1);
        }

        // #[cfg(feature="defmt-log")]
        // info!("[{}] Temperature: {}", HOTEND_LABEL, last_temperature.unwrap().as_celsius());
        // SAFETY - unwrap last_temperature because it's set on the previous line
        if last_temperature.unwrap() > config.heater.temperature_limit.1 {
            // SAFETY - unwrap last_temperature because it's set on the previous line
            let e = PrinterEvent::HotendOverheating(last_temperature.unwrap());
            event_channel_publisher.publish(e).await;
            report.clear();
            task_write!(&mut report, HOTEND_LABEL, "{}", e).unwrap();
            FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
        }

        if let Some(e) = event_channel_subscriber.try_next_message_pure() {
            match e {
                PrinterEvent::HotendOverheating(_)
                | PrinterEvent::HotendUnderheating(_)
                | PrinterEvent::HeatbedUnderheating(_)
                | PrinterEvent::HeatbedOverheating(_)
                | PrinterEvent::Stepper(_)
                | PrinterEvent::PrintCompleted => {
                    let mut pwm = PMW.lock().await;
                    let pwm = pwm.as_mut().expect("PWM not initialized");
                    hotend.disable(pwm);
                }
                _ => (),
            }
        }
        // temperature report period must be a multiple of the loop delay
        // #[cfg(feature = "defmt-log")]
        // info!("{} {}", counter, temperature_report_dt);

        if temperature_report_dt.is_some()
            // SAFETY - unwrap temperature_report_dt because it's set on the previous line
            && counter >= temperature_report_dt.unwrap()
        {
            report.clear();
            let temp = last_temperature.unwrap();
            // SAFETY: last temperature is set before this instruction
            task_write!(
                &mut report,
                HOTEND_LABEL,
                "Temperature: {:.2}°C",
                temp.as_celsius()
            )
            .unwrap();
            FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
            counter = Duration::from_secs(0);
        }
        if let Some(cmd) = watch_receiver.try_changed() {
            if cmd.destination & (1u8 << u8::from(TaskId::Hotend)) != 0{
                match cmd.cmd {
                    GCommand::M104 { s } => {
                        #[cfg(feature = "defmt-log")]
                        info!("[HOTEND] Target temperature: {}", s.as_celsius());
                        hotend.set_temperature(s);
                        {
                            let mut pwm = PMW.lock().await;
                            let pwm = pwm.as_mut().expect("PWM not initialized");
                            hotend.enable(pwm);
                        }
                    }
                    GCommand::M105 => {
                        report.clear();
                        // SAFETY: last temperature is set before this instruction
                        task_write!(
                            &mut report,
                            HOTEND_LABEL,
                            "Temperature: {}",
                            last_temperature.unwrap()
                        )
                        .unwrap();
                        FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(())
                    }
                    GCommand::M106 { s } => {
                        let multiplier = f64::from(255) / f64::from(s);
                        let speed = fan_controller.get_max_speed() * multiplier;
                        {
                            let mut pwm = PMW.lock().await;
                            let pwm = pwm.as_mut().expect("PWM not initialized");
                            fan_controller.set_speed(speed, pwm);
                        }
                        #[cfg(feature = "defmt-log")]
                        info!(
                            "[ThermalActuator HANDLER] Fan speed: {} revs/s",
                            speed.as_rpm()
                        );
                    }
                    GCommand::M155 { s } => {
                        let duration = Duration::from_millis(s.as_millis() as u64);
                        temperature_report_dt.replace(duration);
                    }
                    _ => (),
                }
                SIGNAL.signal(TaskId::Hotend);
            }
        }

        Timer::after(dt).await;
        if let Some(d) = counter.checked_add(dt) {
            counter = d;
        }else{
            counter = Duration::from_secs(0);
        }
    }
}

// // https://dev.to/apollolabsbin/embedded-rust-embassy-analog-sensing-with-adcs-1e2n
#[embassy_executor::task]
async fn heatbed_handler(config: ThermalActuatorConfig<HeatbedAdcInputPin>) {
    // TODO adjust the period using the dt of the loop
    let mut temperature_report_dt: Option<Duration> = None;
    // SAFETY - HEATBED_DMA_BUF is used only in this task
    let readings = unsafe { &mut HEATBED_DMA_BUF };

    let thermistor: Thermistor<'_, AdcWrapper<'_, _, _>> = Thermistor::new(
        config.thermistor.input.degrade_adc(),
        readings,
        config.thermistor.options,
    );

    let channel = timer_channel!(config.heater.pwm.channel).expect("Invalid timer channel");
    let heater = Heater::new(channel, config.heater.pid);
    let mut heatbed = ThermalActuator::new(heater, thermistor);

    let dt = Duration::from_millis(100);
    let mut counter = Duration::from_secs(0);
    let mut report: String<MAX_MESSAGE_LEN> = String::new();
    let mut event_channel_subscriber = EVENT_CHANNEL
        .subscriber()
        .expect("Cannot retrieve error subscriber");
    let event_channel_publisher = EVENT_CHANNEL
        .publisher()
        .expect("Cannot retrieve error subscriber");
    let mut watch_receiver = WATCH.receiver().expect("Cannot retrieve receiver");
    let mut last_temperature: Option<Temperature> = None;

    loop {
        {
            let mut pwm = PMW.lock().await;
            let pwm = pwm.as_mut().expect("PWM not initialized");
            let mut adc = ADC.lock().await;
            let adc = adc.as_mut().expect("ADC not initialized");
            let data = heatbed.update(dt.into(), pwm, adc).await;
            last_temperature.replace(data.0);
            // #[cfg(feature = "defmt-log")]
            // info!("[HEATBED] Temperature: {}\tDuty cycle: {}", data.0.as_celsius(), data.1);
        }

        // #[cfg(feature="defmt-log")]
        // info!("[{}] Temperature: {}", HEATBED_LABEL, last_temperature.unwrap().as_celsius());

        if last_temperature.unwrap() > config.heater.temperature_limit.1 {
            let e = PrinterEvent::HeatbedOverheating(last_temperature.unwrap());
            event_channel_publisher.publish(e).await;
            report.clear();
            task_write!(&mut report, HEATBED_LABEL, "{}", e).unwrap();
            FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
        }

        if let Some(e) = event_channel_subscriber.try_next_message_pure() {
            match e {
                PrinterEvent::HeatbedOverheating(_)
                | PrinterEvent::HeatbedUnderheating(_)
                | PrinterEvent::HotendUnderheating(_)
                | PrinterEvent::HotendOverheating(_)
                | PrinterEvent::Stepper(_)
                | PrinterEvent::PrintCompleted => {
                    let mut pwm = PMW.lock().await;
                    let pwm = pwm.as_mut().expect("PWM not initialized");
                    heatbed.disable(pwm);
                }
                _ => (),
            }
        }

        // #[cfg(feature = "defmt-log")]
        // info!("{} {}", counter, temperature_report_dt);

        // temperature report period must be a multiple of the loop delay
        if temperature_report_dt.is_some()
            && counter >= temperature_report_dt.unwrap()
        {
            let temp = last_temperature.unwrap();
            report.clear();
            task_write!(&mut report, HEATBED_LABEL, "Temperature: {:.2}°C", temp.as_celsius()).unwrap();
            FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
            counter = Duration::from_secs(0);
        }

        if let Some(cmd) = watch_receiver.try_changed() {
            if cmd.destination & (1u8 << u8::from(TaskId::Heatbed)) != 0{
                match cmd.cmd {
                    GCommand::M140 { s } => {
                        heatbed.set_temperature(s);
                        {
                            let mut pwm = PMW.lock().await;
                            let pwm = pwm.as_mut().expect("PWM not initialized");
                            heatbed.enable(pwm);
                        }
                    },
                    GCommand::M105 => {
                        let temp = last_temperature.unwrap();
                        report.clear();
                        task_write!(&mut report, HEATBED_LABEL, "Temperature: {}", temp).unwrap();
                        FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(())
                    }
                    GCommand::M155 { s } => {
                        let duration = Duration::from_millis(s.as_millis() as u64);
                        temperature_report_dt.replace(duration);
                    }
                    _ => (),
                }
                SIGNAL.signal(TaskId::Heatbed);
            }
        };

        Timer::after(dt).await;
        if let Some(d) = counter.checked_add(dt) {
            counter = d;
        }else{
            counter = Duration::from_secs(0);
        }
    }
}

#[embassy_executor::task]
async fn sdcard_handler(
    config: SdCardConfig<
        SdCardSpiPeripheral,
        SdCardSpiTimer,
        SdCardSpiMosiPin,
        SdCardSpiMisoPin,
        SdCardSpiCsPin,
    >,
) {
    static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<'static, Blocking>>>> = StaticCell::new();
    let spi = spi::Spi::new_blocking(
        config.spi.peripheral,
        config.spi.clk,
        config.spi.mosi,
        config.spi.miso,
        Default::default(),
    );
    let spi_bus = NoopMutex::new(RefCell::new(spi));
    let spi_bus = SPI_BUS.init(spi_bus);

    // Device 1, using embedded-hal compatible driver for ST7735 LCD display
    let cs_pin = Output::new(config.spi.cs, Level::High, embassy_stm32::gpio::Speed::Low);

    let spi = SpiDevice::new(spi_bus, cs_pin);
    let sdcard = SdCard::new(spi, Delay);
    let clock = Clock::new();
    let mut volume_manager = VolumeManager::new(sdcard, clock);
    let mut working_dir = None;
    let mut working_file = None;
    let mut working_volume = None;
    let mut running = false;
    let mut msg: String<MAX_MESSAGE_LEN> = String::new();
    let mut tmp: [u8; 128] = [0u8; 128];
    let mut clock = Clock::new();
    let mut report: String<MAX_MESSAGE_LEN> = String::new();
    let mut event_channel_subscriber = EVENT_CHANNEL
        .subscriber()
        .expect("Cannot retrieve error subscriber");
    let event_channel_publisher = EVENT_CHANNEL
        .publisher()
        .expect("Cannot retrieve error subscriber");
    let dt = Duration::from_millis(100);
    let mut watch_receiver = WATCH.receiver().expect("Cannot retrieve receiver");

    loop {
        if event_channel_subscriber.try_next_message_pure().is_some() {
            if let Some(wf) = working_file {
                volume_manager.close_file(wf).expect("cannot close file");
                working_file = None;
                #[cfg(feature = "defmt-log")]
                info!("File closed");
            }
            if let Some(wd) = working_dir {
                volume_manager.close_dir(wd).expect("cannot close directory");
                working_dir = None;
                #[cfg(feature = "defmt-log")]
                info!("Directory closed");
            }
            if let Some(wv) = working_volume {
                volume_manager.close_volume(wv).expect("cannot close volume");
                working_volume = None;
                #[cfg(feature = "defmt-log")]
                info!("Volume closed");
            }
            running = false;
        }

        if let Some(cmd) = watch_receiver.try_changed() {
            if cmd.destination & (1 << u8::from(TaskId::SdCard)) != 0{
                match cmd.cmd {
                    GCommand::M20 => {
                        let dir = working_dir.expect("Working directory not set");
                        let mut msg: String<MAX_MESSAGE_LEN> =
                            String::from_str("Begin file list\n").unwrap();
                        volume_manager
                            .iterate_dir(dir, |d| {
                                let mut name_vec: Vec<u8, 16> = 
                                    Vec::from_slice(d.name.base_name()).unwrap();
                                if d.name.extension().len() > 0{
                                    name_vec.extend_from_slice(&[b'.']).unwrap();
                                    name_vec.extend_from_slice(d.name.extension()).unwrap();
                                }
                                let name = String::from_utf8(name_vec).unwrap();
                                msg.push_str(name.as_str()).unwrap();
                                msg.push('\n').unwrap();
                            })
                            .expect("Error while listing files");
                        msg.push_str("End file list\n").unwrap();
                        FEEDBACK_CHANNEL.send(msg.clone()).await;
                        // TODO send message to UART
                    }
                    GCommand::M21 => {
                        working_volume = match volume_manager.open_raw_volume(VolumeIdx(0)) {
                            Ok(v) => Some(v),
                            Err(_) => panic!("Cannot find module"),
                        };
                        working_dir = match volume_manager.open_root_dir(working_volume.unwrap()) {
                            Ok(d) => Some(d),
                            Err(_) => panic!("Cannot open root dir"),
                        };
                        #[cfg(feature = "defmt-log")]
                        info!("Directory open");
                    }
                    GCommand::M22 => {
                        if working_file.is_some() {
                            volume_manager.close_file(working_file.unwrap()).unwrap();
                            #[cfg(feature = "defmt-log")]
                            info!("File closed");
                        }
                        if working_dir.is_some() {
                            volume_manager.close_dir(working_dir.unwrap()).unwrap();
                            #[cfg(feature = "defmt-log")]
                            info!("Directory closed");
                        }
                        if working_volume.is_some() {
                            volume_manager
                                .close_volume(working_volume.unwrap())
                                .unwrap();
                            #[cfg(feature = "defmt-log")]
                            info!("Volume closed");
                        }
                    }
                    GCommand::M23 { filename } => {
                        let dir = working_dir.expect("Working directory not set");
                        working_file = match volume_manager.open_file_in_dir(
                            dir,
                            filename.as_str(),
                            embedded_sdmmc::Mode::ReadOnly,
                        ) {
                            Ok(f) => Some(f),
                            Err(_) => panic!("File not found"),
                        };
                        #[cfg(feature = "defmt-log")]
                        info!("Working file set");
                    }
                    // ignore the parameters of M24, just start/resume the print
                    GCommand::M24 { .. } => {
                        if !running {
                            clock.start();
                            running = true;
                            // event_channel_publisher
                            // .publish(PrinterEvent::PrintStarted)
                            // .await;
                        }
                    }
                    GCommand::M25 => {
                        if running {
                            clock.stop();
                            running = false;
                            // event_channel_publisher
                            // .publish(PrinterEvent::PrintStarted)
                            // .await;
                        }
                    }
                    GCommand::M31 => {
                        report.clear();
                        task_write!(
                            &mut report,
                            SD_CARD_LABEL,
                            "Time elapsed: {}",
                            clock.measure().as_millis()
                        )
                        .unwrap();
                        FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
                    }
                    GCommand::M524 => {
                        event_channel_publisher
                            .publish(PrinterEvent::PrintAborted)
                            .await;
                        report.clear();
                        task_write!(&mut report, PLANNER_LABEL, "Print aborted", ).unwrap();
                        FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
                    }
                    _ => (),
                }
                SIGNAL.signal(TaskId::SdCard);
            }
        }

        if running && working_file.is_some() {
            let n = volume_manager.read(
                working_file.unwrap(),
                &mut tmp
            ).expect("Something went wrong during the SD-Card file reading");
            if n == 0{
                event_channel_publisher.publish(PrinterEvent::EOF).await;
            }
            else{
                for b in &tmp[0..n] {
                    if *b == b'\n' {
                        let cmd = TaskMessage{
                            msg: msg.clone(),
                            priority: TaskMessagePriority::Low
                        };
                        COMMAND_DISPATCHER_CHANNEL.send(cmd).await;
                        #[cfg(feature = "defmt-log")]
                        info!("[{}] {}", SD_CARD_LABEL, msg.as_str());
                        msg.clear();
                    } else {
                        if msg.push((*b).into()).is_err(){
                            msg.clear();
                            #[cfg(feature = "defmt-log")]
                            error!("[{}] Message too long", SD_CARD_LABEL);
                        }
                    }
                }
            }
        }
        Timer::after(dt).await;
    }
}

#[embassy_executor::task]
async fn planner_handler(
    config: SteppersConfig<
        XStepPin,
        XDirPin,
        YStepPin,
        YDirPin,
        ZStepPin,
        ZDirPin,
        EStepPin,
        EDirPin,
    >,
    motion_config: MotionConfig,
    endstops_config: EndstopsConfig<
        XEndstopPin,
        XEndstopExti,
        YEndstopPin,
        YEndstopExti,
        ZEndstopPin,
        ZEndstopExti,
    >,
) {
    let mut report: String<MAX_MESSAGE_LEN> = String::new();
    let mut debug = false;

    let x_stepper = init_stepper!(
        config.x.step_pin,
        config.x.dir_pin,
        StepperOptions {
            steps_per_revolution: config.x.steps_per_revolution,
            stepping_mode: config.x.stepping_mode,
            bounds: Some(config.x.bounds),
            positive_direction: config.x.positive_direction,
            acceleration: None
        },
        StepperAttachment {
            distance_per_step: config.x.distance_per_step,
        }
    );

    let y_stepper = init_stepper!(
        config.y.step_pin,
        config.y.dir_pin,
        StepperOptions {
            steps_per_revolution: config.y.steps_per_revolution,
            stepping_mode: config.y.stepping_mode,
            bounds: Some(config.y.bounds),
            positive_direction: config.y.positive_direction,
            acceleration: None
        },
        StepperAttachment {
            distance_per_step: config.y.distance_per_step,
        }
    );

    let z_stepper = init_stepper!(
        config.z.step_pin,
        config.z.dir_pin,
        StepperOptions {
            steps_per_revolution: config.z.steps_per_revolution,
            stepping_mode: config.z.stepping_mode,
            bounds: Some(config.z.bounds),
            positive_direction: config.z.positive_direction,
            acceleration: None
        },
        StepperAttachment {
            distance_per_step: config.z.distance_per_step,
        }
    );

    let e_stepper = init_stepper!(
        config.e.step_pin,
        config.e.dir_pin,
        StepperOptions {
            steps_per_revolution: config.e.steps_per_revolution,
            stepping_mode: config.e.stepping_mode,
            bounds: Some(config.e.bounds),
            positive_direction: config.e.positive_direction,
            acceleration: None
        },
        StepperAttachment {
            distance_per_step: config.e.distance_per_step,
        }
    );

    let x_endstop = ExtiInput::new(endstops_config.x.pin, endstops_config.x.exti, Pull::Down);
    let x_endstop = init_input_pin!(x_endstop);

    let y_endstop = ExtiInput::new(endstops_config.y.pin, endstops_config.y.exti, Pull::Down);
    let y_endstop = init_input_pin!(y_endstop);

    let z_endstop = ExtiInput::new(endstops_config.z.pin, endstops_config.z.exti, Pull::Down);
    let z_endstop = init_input_pin!(z_endstop);

    let endstops = (Some(x_endstop), Some(y_endstop), Some(z_endstop), None);

    let mut planner: Planner<OutputPinWrapper<'_>, StepperTimer, ExtiInputPinWrapper> =
        Planner::new(
            x_stepper,
            y_stepper,
            z_stepper,
            e_stepper,
            motion_config,
            endstops,
        );

    let dt = Duration::from_millis(20);
    let mut event_channel_subscriber = EVENT_CHANNEL
        .subscriber()
        .expect("Cannot retrieve error subscriber");
    let event_channel_publisher = EVENT_CHANNEL
        .publisher()
        .expect("Cannot retrieve error subscriber");
    let mut watch_receiver = WATCH.receiver().expect("Cannot retrieve receiver");

    loop {
        if let Some(e) = event_channel_subscriber.try_next_message_pure() {
            match e {
                PrinterEvent::EOF => {
                    event_channel_publisher
                    .publish(PrinterEvent::PrintCompleted)
                    .await;
                }
                _ => {}
            }
        }

        let cmd = watch_receiver.changed().await;

        if cmd.destination & (1u8 << u8::from(TaskId::Planner)) != 0{

            #[cfg(feature = "defmt-log")]
            info!("[PLANNER HANDLER] command received");
            match cmd.cmd {
                GCommand::G0 { .. }
                | GCommand::G1 { .. }
                | GCommand::G2 { .. }
                | GCommand::G3 { .. }
                | GCommand::G4 { .. }
                | GCommand::G10
                | GCommand::G11
                | GCommand::G28 { .. }
                | GCommand::G90
                | GCommand::G91
                | GCommand::G92 { .. }
                | GCommand::M207 { .. }
                | GCommand::M208 { .. }
                | GCommand::M220 { .. } => match planner.execute(cmd.cmd.clone()).await {
                    Ok(duration) => {}
                    Err(e) => {
                        event_channel_publisher
                            .publish(PrinterEvent::Stepper(e))
                            .await;
                        report.clear();
                        task_write!(&mut report, PLANNER_LABEL, "{}", e).unwrap();
                        FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
                    }
                },
                GCommand::M114 => {
                    report.clear();
                    task_write!(
                        &mut report,
                        PLANNER_LABEL,
                        "Head position: [X:{}] [Y:{}] [Z:{}]",
                        planner.get_x_position(),
                        planner.get_y_position(),
                        planner.get_z_position(),
                    )
                    .unwrap();
                    FEEDBACK_CHANNEL.try_send(report.clone()).unwrap_or(());
                }
                _ => {
                    // #[cfg(feature = "defmt-log")]
                    // error!("[PLANNER HANDLER] command not handled")
                }
            }
            SIGNAL.signal(TaskId::Planner);
        }
        // Timer::after(dt).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

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

    let printer_config = peripherals_init(p);

    let mut uart_config = embassy_stm32::usart::Config::default();
    uart_config.baudrate = printer_config.uart.baudrate as u32;

    let uart = Uart::new(
        printer_config.uart.peripheral,
        printer_config.uart.rx.pin,
        printer_config.uart.tx.pin,
        Irqs,
        printer_config.uart.tx.dma,
        printer_config.uart.rx.dma,
        uart_config,
    )
    .expect("UART configuration not valid");

    let (tx, rx) = uart.split();

    {
        let mut uart_rx = UART_RX.lock().await;
        uart_rx.replace(rx);
    }

    {
        let mut uart_tx = UART_TX.lock().await;
        uart_tx.replace(tx);
    }

    let pwm = SimplePwm::new(
        printer_config.pwm.timer,
        Some(PwmPin::new_ch1(
            printer_config.pwm.ch1,
            OutputType::PushPull,
        )),
        Some(PwmPin::new_ch2(
            printer_config.pwm.ch2,
            OutputType::PushPull,
        )),
        Some(PwmPin::new_ch3(
            printer_config.pwm.ch3,
            OutputType::PushPull,
        )),
        None,
        // FIXME change PWM configuration to u32
        hz(printer_config.pwm.frequency as u32),
        CountingMode::EdgeAlignedUp,
    );
    let mut pwm = SimplePwmWrapper::new(pwm);
    pwm.enable(embassy_stm32::timer::Channel::Ch1);
    pwm.set_duty(embassy_stm32::timer::Channel::Ch1, 0);
    pwm.disable(embassy_stm32::timer::Channel::Ch2);
    pwm.set_duty(embassy_stm32::timer::Channel::Ch2, 0);
    pwm.disable(embassy_stm32::timer::Channel::Ch3);
    pwm.set_duty(embassy_stm32::timer::Channel::Ch3, 0);
    pwm.disable(embassy_stm32::timer::Channel::Ch4);
    pwm.set_duty(embassy_stm32::timer::Channel::Ch4, 0);
    #[cfg(feature="defmt-log")]
    info!("Duty cycle: {}", pwm.get_max_duty());

    {
        let mut pwm_global = PMW.lock().await;
        pwm_global.replace(pwm);
    }

    let adc = Adc::new(printer_config.adc.peripheral);
    let adc = AdcWrapper::new(
        adc,
        printer_config.adc.dma,
        ResolutionWrapper::new(Resolution::BITS12),
        SampleTime::CYCLES64_5
    );
    {
        let mut adc_global = ADC.lock().await;
        adc_global.replace(adc);
    }

    spawner.spawn(input_handler()).unwrap();

    spawner.spawn(output_handler()).unwrap();

    spawner.spawn(command_dispatcher_task()).unwrap();

    spawner
        .spawn(heatbed_handler(printer_config.heatbed))
        .unwrap();

    spawner
        .spawn(hotend_handler(printer_config.hotend, printer_config.fan))
        .unwrap();

    interrupt::TIM2.set_priority(interrupt::Priority::P6);
    let planner_spawner = EXECUTOR_HIGH.start(interrupt::TIM2);

    planner_spawner
        .spawn(planner_handler(
            printer_config.steppers,
            printer_config.motion,
            printer_config.endstops,
        ))
        .unwrap();

    spawner
        .spawn(sdcard_handler(printer_config.sdcard))
        .unwrap();

    let mut alive_led = init_output_pin!(printer_config.debug.alive_led);

    loop {
        #[cfg(feature = "defmt-log")]
        info!("[MAIN LOOP] alive");
        alive_led.toggle();
        Timer::after(Duration::from_secs(2)).await;
    }
}
