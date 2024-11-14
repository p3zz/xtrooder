use embassy_stm32::timer::{simple_pwm::SimplePwm, Channel, GeneralInstance4Channel};
use math::measurements::AngularVelocity;
use micromath::F32Ext;

pub struct FanController {
    ch: Channel,
    max_speed: AngularVelocity,
}

impl FanController {
    pub fn new(ch: Channel, max_speed: AngularVelocity) -> Self {
        Self { ch, max_speed }
    }

    pub fn enable<T: GeneralInstance4Channel>(&self, pwm: &mut SimplePwm<'_, T>) {
        pwm.enable(self.ch);
    }

    pub fn disable<T: GeneralInstance4Channel>(&self, pwm: &mut SimplePwm<'_, T>) {
        pwm.disable(self.ch);
    }

    pub fn set_speed<T: GeneralInstance4Channel>(
        &mut self,
        rpm: AngularVelocity,
        pwm: &mut SimplePwm<'_, T>,
    ) {
        let rpm = rpm.as_rpm().max(0f64).min(self.max_speed.as_rpm());

        let multiplier = self.max_speed.as_rpm() / rpm;
        let duty_cycle = ((f64::from(pwm.get_max_duty()) * multiplier) as f32).trunc() as u32;
        pwm.set_duty(self.ch, duty_cycle);
    }

    pub fn get_max_speed(&self) -> AngularVelocity {
        self.max_speed
    }

}
