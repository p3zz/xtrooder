#![no_std]
#![no_main]

use core::{fmt::Display, marker::PhantomData};

use common::{MyAdc, MyPwm, StatefulInputPin, StatefulOutputPin, TimerTrait};
use embassy_stm32::{
    adc::{Adc, AnyAdcChannel, Instance, Resolution, RxDma, SampleTime}, exti::ExtiInput, gpio::Output, timer::{simple_pwm::SimplePwm, Channel, GeneralInstance4Channel}
};
use math::measurements::Temperature;
use stepper::stepper::StepperError;
use embassy_time::{Duration, Instant, Timer};
use embedded_sdmmc::{TimeSource, Timestamp};

pub mod config;
pub mod ext;

#[derive(Clone, Copy, Debug)]
pub enum PrinterEvent {
    HotendOverheating(Temperature),
    HotendUnderheating(Temperature),
    HeatbedOverheating(Temperature),
    HeatbedUnderheating(Temperature),
    Stepper(StepperError),
    EOF,
    PrintCompleted,
}

impl Display for PrinterEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self {
            PrinterEvent::HotendOverheating(temperature) => {
                core::write!(f, "Hotend overheating: {}C", temperature.as_celsius())
            }
            PrinterEvent::HotendUnderheating(temperature) => {
                core::write!(f, "Hotend underheating: {}C", temperature.as_celsius())
            }
            PrinterEvent::HeatbedOverheating(temperature) => {
                core::write!(f, "Heatbed overheating: {}C", temperature.as_celsius())
            }
            PrinterEvent::HeatbedUnderheating(temperature) => {
                core::write!(f, "Heatbed underheating: {}C", temperature.as_celsius())
            }
            PrinterEvent::Stepper(stepper_error) => {
                core::write!(f, "Stepper error: {}", stepper_error)
            }
            PrinterEvent::EOF => {
                core::write!(f, "SD-card EOF")
            }
            PrinterEvent::PrintCompleted => {
                core::write!(f, "Print completed")
            }
        }
    }
}

pub struct OutputPinWrapper<'a> {
    pin: Output<'a>,
}

impl <'a> OutputPinWrapper<'a>{
    pub fn new(pin: Output<'a>) -> Self {
        Self{pin}
    }
}

impl StatefulOutputPin for OutputPinWrapper<'_> {
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

pub struct ExtiInputPinWrapper<'a> {
    pin: ExtiInput<'a>,
}

impl <'a> ExtiInputPinWrapper<'a>{
    pub fn new(pin: ExtiInput<'a>) -> Self {
        Self{pin}
    }
}

impl StatefulInputPin for ExtiInputPinWrapper<'_> {
    fn is_high(&self) -> bool {
        self.pin.is_high()
    }
    fn wait_for_high(&mut self) -> impl core::future::Future<Output = ()> {
        self.pin.wait_for_high()
    }

    fn wait_for_low(&mut self) -> impl core::future::Future<Output = ()> {
        self.pin.wait_for_low()
    }
}

pub struct StepperTimer {}

impl TimerTrait for StepperTimer {
    async fn after(duration: core::time::Duration) {
        let duration = embassy_time::Duration::from_micros(duration.as_micros() as u64);
        Timer::after(duration).await
    }
}

#[macro_export]
macro_rules! init_output_pin {
    ($config: ident) => {
        app::OutputPinWrapper::new(
            Output::new(
                $config,
                Level::Low,
                PinSpeed::Low
            )
        )
    };
}

#[macro_export]
macro_rules! init_input_pin {
    ($config: ident) => {
        app::ExtiInputPinWrapper::new($config)
    };
}

#[macro_export]
macro_rules! init_stepper {
    ($step_pin: ident, $dir_pin: ident, $options: ident, $attachment: ident) => {
        stepper::stepper::Stepper::new_with_attachment(
            app::init_output_pin!($step_pin),
            app::init_output_pin!($dir_pin),
            $options,
            $attachment,
        )
    };
}

#[macro_export]
macro_rules! timer_channel {
    ($channel: ident) => {{
        match $channel {
            1 => Some(embassy_stm32::timer::Channel::Ch1),
            2 => Some(embassy_stm32::timer::Channel::Ch2),
            3 => Some(embassy_stm32::timer::Channel::Ch3),
            4 => Some(embassy_stm32::timer::Channel::Ch4),
            _ => None,
        }
    }};
}

pub struct SimplePwmWrapper<'a, T: GeneralInstance4Channel> {
    inner: SimplePwm<'a, T>,
}

impl<'a, T: GeneralInstance4Channel> MyPwm for SimplePwmWrapper<'a, T> {
    type Channel = Channel;
    type Pwm = SimplePwm<'a, T>;

    fn new(p: Self::Pwm) -> Self {
        Self { inner: p }
    }

    fn enable(&mut self, channel: Self::Channel) {
        self.inner.enable(channel);
    }

    fn disable(&mut self, channel: Self::Channel) {
        self.inner.disable(channel);
    }

    fn get_max_duty(&self) -> u64 {
        u64::from(self.inner.get_max_duty())
    }

    fn set_duty(&mut self, channel: Self::Channel, duty_cycle: u64) {
        self.inner.set_duty(channel, duty_cycle as u32);
    }
}

#[derive(Clone, Copy)]
pub struct ResolutionWrapper {
    inner: Resolution,
}

impl ResolutionWrapper {
    pub fn new(inner: Resolution) -> Self {
        Self { inner }
    }
}

impl From<ResolutionWrapper> for u64 {
    fn from(val: ResolutionWrapper) -> Self {
        match val.inner {
            Resolution::BITS16 => 1 << 16,
            Resolution::BITS14 => 1 << 14,
            Resolution::BITS12 => 1 << 12,
            Resolution::BITS10 => 1 << 10,
            Resolution::BITS14V => 1 << 14,
            Resolution::BITS12V => 1 << 12,
            Resolution::BITS8 => 1 << 8,
            _ => 0,
        }
    }
}

pub struct AdcWrapper<'a, T: Instance, DmaType> {
    inner: Adc<'a, T>,
    _dma_type: PhantomData<DmaType>,
}

impl<'a, T: Instance, DmaType: RxDma<T>> MyAdc for AdcWrapper<'a, T, DmaType> {
    type PeriType = T;

    type PinType = AnyAdcChannel<T>;

    type DmaType = DmaType;

    type SampleTime = SampleTime;

    type Resolution = ResolutionWrapper;

    fn new(peripheral: Self::PeriType) -> Self {
        Self {
            inner: Adc::new(peripheral),
            _dma_type: PhantomData,
        }
    }

    fn set_sample_time(&mut self, sample_time: Self::SampleTime) {
        self.inner.set_sample_time(sample_time);
    }

    fn sample_time(&self) -> Self::SampleTime {
        self.inner.sample_time()
    }

    fn set_resolution(&mut self, resolution: Self::Resolution) {
        self.inner.set_resolution(resolution.inner);
    }

    fn read(
        &mut self,
        dma: &mut Self::DmaType,
        pin: core::array::IntoIter<(&mut Self::PinType, Self::SampleTime), 1>,
        readings: &mut [u16],
    ) -> impl core::future::Future<Output = ()> {
        self.inner.read(dma, pin, readings)
    }
}

#[derive(Clone, Copy)]
pub struct Clock {
    start_ticks: u64,
    stop_ticks: u64,
    elapsed_ticks: u64,
    running: bool,
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            start_ticks: 0,
            stop_ticks: 0,
            elapsed_ticks: 0,
            running: false,
        }
    }

    fn now() -> Instant {
        Instant::now()
    }

    pub fn start(&mut self) {
        if !self.running {
            self.start_ticks = Clock::now().as_ticks();
        }
    }

    pub fn stop(&mut self) {
        if self.running {
            self.stop_ticks = Clock::now().as_ticks();
            self.elapsed_ticks += self.stop_ticks - self.start_ticks;
            self.running = false;
        }
    }

    pub fn measure(&self) -> Duration {
        let elapsed_ticks = if self.running {
            self.elapsed_ticks + Clock::now().as_ticks() - self.start_ticks
        } else {
            self.elapsed_ticks
        };
        Duration::from_ticks(elapsed_ticks)
    }

    pub fn reset(&mut self) {
        self.start_ticks = 0;
        self.stop_ticks = 0;
        self.elapsed_ticks = 0;
        self.running = false;
    }
}

impl TimeSource for Clock {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_day: 0,
            zero_indexed_month: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}
