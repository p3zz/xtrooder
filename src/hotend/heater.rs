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

    pub fn set_value(&mut self, v: f64){
        // the max duty cycle is 2^16 - 1
        let duty_cycle = (65535.0 * v) as u16;
        self.out.set_duty(Channel::Ch1, duty_cycle);
    }
}