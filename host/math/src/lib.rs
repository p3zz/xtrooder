#![cfg_attr(not(test), no_std)]

pub use measurements;

pub mod angle;
pub mod common;
pub mod vector;

pub enum DurationUnit {
    Second,
    Millisecond,
}

pub enum TemperatureUnit{
    Celsius,
    Farhenheit,
    Kelvin
}

pub enum DistanceUnit{
    Millimeter,
    Inch,
}