#![cfg_attr(not(test), no_std)]

use common::{MyPwm, PwmOutputConfig};
use math::measurements::AngularVelocity;

pub struct FanConfig {
    pub max_speed: AngularVelocity,
    pub pwm: PwmOutputConfig,
}

pub struct FanController<P: MyPwm> {
    ch: P::Channel,
    max_speed: AngularVelocity,
}

impl<P: MyPwm> FanController<P> {
    pub fn new(ch: P::Channel, max_speed: AngularVelocity) -> Self {
        Self { ch, max_speed }
    }

    pub fn enable(&self, pwm: &mut P) {
        pwm.enable(self.ch);
    }

    pub fn disable(&self, pwm: &mut P) {
        pwm.disable(self.ch);
    }

    pub fn set_speed(&mut self, rpm: AngularVelocity, pwm: &mut P) {
        let rpm = rpm.as_rpm().max(0f64).min(self.max_speed.as_rpm());

        let multiplier = self.max_speed.as_rpm() / rpm;
        let duty_cycle = (pwm.get_max_duty() as f64 * multiplier) as u64;
        pwm.set_duty(self.ch, duty_cycle);
    }

    pub fn get_max_speed(&self) -> AngularVelocity {
        self.max_speed
    }
}
