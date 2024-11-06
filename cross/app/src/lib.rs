#![no_std]
#![no_main]

pub mod fan;
pub mod hotend;
pub mod utils;
pub mod config;
pub mod ext;

#[macro_export]
macro_rules! init_pin {
    ($config: ident) => {
        StepperPin {
            pin: Output::new($config, Level::Low, PinSpeed::Low),
        }    
    };
}

#[macro_export]
macro_rules! init_stepper{
    ($step_pin: ident, $dir_pin: ident, $options: ident, $attachment: ident) => {
        Stepper::new_with_attachment(
            init_pin!($step_pin),
            init_pin!($dir_pin),
            $options,
            $attachment,
        )    
    };
}

