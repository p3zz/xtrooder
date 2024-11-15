#![no_std]
#![no_main]

use core::fmt::Display;

use common::MyPwm;
use math::measurements::Temperature;
use stepper::stepper::StepperError;
use embassy_stm32::timer::{simple_pwm::SimplePwm, Channel, GeneralInstance4Channel};

pub mod config;
pub mod ext;
pub mod hotend;
pub mod utils;

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
            },
            PrinterEvent::PrintCompleted => {
                core::write!(f, "Print completed")
            }
        }
    }
}

#[macro_export]
macro_rules! init_output_pin {
    ($config: ident) => {
        StepperOutputPin {
            pin: Output::new($config, Level::Low, PinSpeed::Low),
        }
    };
}

#[macro_export]
macro_rules! init_input_pin {
    ($config: ident) => {
        StepperInputPin { pin: $config }
    };
}

#[macro_export]
macro_rules! init_stepper {
    ($step_pin: ident, $dir_pin: ident, $options: ident, $attachment: ident) => {
        Stepper::new_with_attachment(
            init_output_pin!($step_pin),
            init_output_pin!($dir_pin),
            $options,
            $attachment,
        )
    };
}

#[macro_export]
macro_rules! timer_channel {
    ($channel: ident) => {
        match $channel {
            1 => Some(TimerChannel::Ch1),
            2 => Some(TimerChannel::Ch2),
            3 => Some(TimerChannel::Ch3),
            4 => Some(TimerChannel::Ch4),
            _ => None,
        }
    };
}

pub struct SimplePwmWrapper<'a, T: GeneralInstance4Channel>{
    inner: SimplePwm<'a, T>
}

impl<'a, T: GeneralInstance4Channel> MyPwm for SimplePwmWrapper<'a, T>{
    type Channel = Channel;

    fn enable(&mut self, channel: Channel) {
        self.inner.enable(channel);
    }

    fn disable(&mut self, channel: Channel) {
        self.inner.disable(channel);
    }
    
    fn get_max_duty(&self) -> u64 {
        u64::from(self.inner.get_max_duty())
    }
    
    fn set_duty(&mut self, channel: Channel, duty_cycle: u64) {
        self.inner.set_duty(channel, duty_cycle as u32);
    }
    
}
