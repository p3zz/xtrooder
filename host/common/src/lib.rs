#![cfg_attr(not(test), no_std)]

pub struct PidConfig {
    pub k_p: f64,
    pub k_i: f64,
    pub k_d: f64,
}

pub struct PwmOutputConfig {
    pub channel: u8,
}

pub trait MyPwm<C>{
    fn enable(&mut self, channel: C);
    fn disable(&mut self, channel: C);
    fn get_max_duty(&self) -> u64;
    fn set_duty(&mut self, channel: C, duty_cycle: u64);
}
