#![cfg_attr(not(test), no_std)]

use common::{MyPwm, PwmOutputConfig};
use math::measurements::AngularVelocity;

pub struct FanConfig {
    pub max_speed: AngularVelocity,
    pub pwm: PwmOutputConfig,
}

pub struct FanController<C: Copy + Clone> {
    ch: C,
    max_speed: AngularVelocity,
}

impl<C: Copy + Clone> FanController<C> {
    pub fn new(ch: C, max_speed: AngularVelocity) -> Self {
        Self { ch, max_speed }
    }

    pub fn enable<P: MyPwm<C>>(&self, pwm: &mut P) {
        pwm.enable(self.ch);
    }

    pub fn disable<P: MyPwm<C>>(&self, pwm: &mut P) {
        pwm.disable(self.ch);
    }

    pub fn set_speed<P: MyPwm<C>>(
        &mut self,
        rpm: AngularVelocity,
        pwm: &mut P,
    ) {
        let rpm = rpm.as_rpm().max(0f64).min(self.max_speed.as_rpm());

        let multiplier = self.max_speed.as_rpm() / rpm;
        let duty_cycle = (pwm.get_max_duty() as f64 * multiplier) as u64;
        pwm.set_duty(self.ch, duty_cycle);
    }

    pub fn get_max_speed(&self) -> AngularVelocity {
        self.max_speed
    }

}
