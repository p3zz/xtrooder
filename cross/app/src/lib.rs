#![no_std]
#![no_main]

use core::fmt::Display;

use math::measurements::Temperature;
use stepper::stepper::StepperError;

pub mod config;
pub mod ext;
pub mod fan;
pub mod hotend;
pub mod utils;

#[derive(Clone, Copy, Debug)]
pub enum EndstopType {
    X,
    Y,
    Z,
    E,
}

#[derive(Clone, Copy, Debug)]
pub enum PrinterError {
    HotendOverheating(Temperature),
    HotendUnderheating(Temperature),
    HeatbedOverheating(Temperature),
    HeatbedUnderheating(Temperature),
    Stepper(StepperError),
}

impl Display for PrinterError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self {
            PrinterError::HotendOverheating(temperature) => {
                core::write!(f, "Hotend overheating: {}C", temperature.as_celsius())
            }
            PrinterError::HotendUnderheating(temperature) => {
                core::write!(f, "Hotend underheating: {}C", temperature.as_celsius())
            }
            PrinterError::HeatbedOverheating(temperature) => {
                core::write!(f, "Heatbed overheating: {}C", temperature.as_celsius())
            }
            PrinterError::HeatbedUnderheating(temperature) => {
                core::write!(f, "Heatbed underheating: {}C", temperature.as_celsius())
            }
            PrinterError::Stepper(stepper_error) => {
                core::write!(f, "Stepper error: {}", stepper_error)
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
