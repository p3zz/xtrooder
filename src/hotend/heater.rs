// use embassy_stm32::pwm::{simple_pwm::SimplePwm, CaptureCompare16bitInstance, Channel};

use embassy_stm32::timer::{simple_pwm::SimplePwm, CaptureCompare16bitInstance, Channel};

pub struct Heater<'s, S> {
    out: SimplePwm<'s, S>,
    ch: Channel,
}

impl<'s, S> Heater<'s, S>
where
    S: CaptureCompare16bitInstance,
{
    pub fn new(out: SimplePwm<'s, S>, ch: Channel) -> Heater<'s, S> {
        Heater { out, ch }
    }

    pub fn set_value(&mut self, v: f64) {
        // the max duty cycle is 2^16 - 1
        let duty_cycle = (65535.0 * v) as u16;
        self.out.set_duty(self.ch, duty_cycle);
    }
}
