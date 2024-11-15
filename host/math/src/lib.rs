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

#[derive(Copy, Clone)]
pub enum Resolution{
    BITS16,
    BITS14,
    BITS12,
    BITS10,
    BITS8,
}

impl Into<u64> for Resolution{
    fn into(self) -> u64 {
        match self {
            Resolution::BITS16 => 1 << 16,
            Resolution::BITS14 => 1 << 14,
            Resolution::BITS12 => 1 << 12,
            Resolution::BITS10 => 1 << 10,
            Resolution::BITS8 => 1 << 8,
            _ => 0,
        }    
    }
}
