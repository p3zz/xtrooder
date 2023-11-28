use embassy_stm32::{pwm::{simple_pwm::SimplePwm, CaptureCompare16bitInstance, Channel}, time::Hertz};
use embassy_time::Duration;

pub struct Heater<'s, S>{
    out: SimplePwm<'s, S>,
}

impl <'s, S> Heater <'s, S>
where
S: CaptureCompare16bitInstance,
{
    pub fn new(out: SimplePwm<'s, S>) -> Heater<'s, S> {
        Heater { out }
    }
}