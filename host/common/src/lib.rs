#![cfg_attr(not(test), no_std)]

use core::{future::Future, time::Duration};

pub struct PidConfig {
    pub k_p: f64,
    pub k_i: f64,
    pub k_d: f64,
}

pub struct PwmOutputConfig {
    pub channel: u8,
}

pub trait PwmBase {
    type Channel: Copy + Clone;

    fn enable(&mut self, channel: Self::Channel);
    fn disable(&mut self, channel: Self::Channel);
    fn get_max_duty(&self) -> u64;
    fn set_duty(&mut self, channel: Self::Channel, duty_cycle: u64);
}

pub trait AdcBase {
    type PinType;
    type SampleTime: Copy + Clone;
    type Resolution: Copy + Clone + Into<u64>;

    fn set_sample_time(&mut self, sample_time: Self::SampleTime);
    fn sample_time(&self) -> Self::SampleTime;
    fn set_resolution(&mut self, resolution: Self::Resolution);
    fn resolution(&self) -> Self::Resolution;
    fn read(&mut self, pin: &mut Self::PinType, readings: &mut [u16]) -> impl Future<Output = ()>;
}

pub trait TimerBase {
    fn after(duration: Duration) -> impl Future<Output = ()>;
}

pub trait OutputPinBase {
    fn set_high(&mut self);
    fn set_low(&mut self);
    fn is_high(&self) -> bool;
}

pub trait ExtiInputPinBase {
    fn is_high(&self) -> bool;
    fn wait_for_high(&mut self) -> impl Future<Output = ()>;
    fn wait_for_low(&mut self) -> impl Future<Output = ()>;
}
