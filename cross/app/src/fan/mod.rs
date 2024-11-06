use embassy_stm32::{
    time::Hertz,
    timer::{simple_pwm::SimplePwm, Channel, GeneralInstance4Channel},
};
use micromath::F32Ext;

pub struct FanController {
    ch: Channel,
    max_speed: f64,
}

impl FanController {
    pub fn new(ch: Channel, max_speed: f64) -> Self {
        Self { ch, max_speed }
    }

    pub fn enable<T: GeneralInstance4Channel>(&self, pwm: &mut SimplePwm<'_, T>) {
        pwm.enable(self.ch);
    }

    pub fn disable<T: GeneralInstance4Channel>(&self, pwm: &mut SimplePwm<'_, T>) {
        pwm.disable(self.ch);
    }

    pub fn set_speed<T: GeneralInstance4Channel>(&mut self, revolutions_per_second: f64, pwm: &mut SimplePwm<'_, T>) {
        let revolutions_per_second = revolutions_per_second.max(0f64).min(self.max_speed);

        let multiplier = self.max_speed / revolutions_per_second;
        let duty_cycle = ((f64::from(pwm.get_max_duty()) * multiplier) as f32).trunc() as u32;
        pwm.set_duty(self.ch, duty_cycle);
    }

    pub fn get_max_speed(&self) -> f64 {
        self.max_speed
    }
}
