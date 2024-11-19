#![cfg_attr(not(test), no_std)]

use core::{array::IntoIter, future::Future};

pub struct PidConfig {
    pub k_p: f64,
    pub k_i: f64,
    pub k_d: f64,
}

pub struct PwmOutputConfig {
    pub channel: u8,
}

pub trait MyPwm{
    type Channel: Copy + Clone;
    type Pwm;

    fn new(p: Self::Pwm) -> Self;
    fn enable(&mut self, channel: Self::Channel);
    fn disable(&mut self, channel: Self::Channel);
    fn get_max_duty(&self) -> u64;
    fn set_duty(&mut self, channel: Self::Channel, duty_cycle: u64);
}

pub trait MyAdc{
    type PinType;
    type DmaType;
    type PeriType;
    type SampleTime : Copy + Clone;
    type Resolution : Copy + Clone + Into<u64>;

    fn new(peripheral: Self::PeriType) -> Self;
    fn set_sample_time(&mut self, sample_time: Self::SampleTime);
    fn sample_time(&self) -> Self::SampleTime;
    fn set_resolution(&mut self, resolution: Self::Resolution);
    fn read(
        &mut self,
        dma: &mut Self::DmaType,
        pin: IntoIter<(&mut Self::PinType, Self::SampleTime), 1>,
        readings: &mut [u16]
    ) -> impl Future<Output = ()>;
}
