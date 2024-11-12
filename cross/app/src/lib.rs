#![no_std]
#![no_main]

pub mod config;
pub mod ext;
pub mod fan;
pub mod hotend;
pub mod utils;

pub enum PrinterError{
    HotendOverheating,
    HotendUnderheating,
    HeatbedOverheating,
    EndstopHit,
    MoveOutOfBounds
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
        StepperInputPin {
            pin: Input::new($config, Pull::Down),
        }
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
macro_rules! timer_channel{
    ($channel: ident) => {
        match $channel{
            1 => Some(TimerChannel::Ch1),
            2 => Some(TimerChannel::Ch2),
            3 => Some(TimerChannel::Ch3),
            4 => Some(TimerChannel::Ch4),
            _ => None
        }  
    };
}