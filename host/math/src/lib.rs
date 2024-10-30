#![cfg_attr(not(test), no_std)]

pub use measurements;

pub mod angle;
pub mod common;
pub mod vector;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DurationUnit {
    Second,
    Millisecond,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TemperatureUnit {
    Celsius,
    Farhenheit,
    Kelvin,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DistanceUnit {
    Millimeter,
    Inch,
}
