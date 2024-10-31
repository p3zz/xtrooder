use embassy_stm32::{
    time::Hertz,
    timer::{simple_pwm::SimplePwm, Channel, GeneralInstance4Channel},
};
use micromath::F32Ext;

pub struct FanController<'s, T: GeneralInstance4Channel> {
    out: SimplePwm<'s, T>,
    ch: Channel,
    max_speed: f64,
}

impl<'s, T: GeneralInstance4Channel> FanController<'s, T> {
    pub fn new(out: SimplePwm<'s, T>, ch: Channel, max_speed: f64) -> Self {
        let mut out = out;
        out.set_frequency(Hertz::hz(100));
        out.disable(ch);
        Self { out, ch, max_speed }
    }

    pub fn enable(&mut self) {
        self.out.enable(self.ch);
    }

    pub fn disable(&mut self) {
        self.out.disable(self.ch);
    }

    pub fn set_speed(&mut self, revolutions_per_second: f64) {
        let revolutions_per_second = revolutions_per_second.max(0f64).min(self.max_speed);

        let multiplier = self.max_speed / revolutions_per_second;
        let duty_cycle = ((f64::from(self.out.get_max_duty()) * multiplier) as f32).trunc() as u32;
        self.out.set_duty(self.ch, duty_cycle);
    }

    pub fn get_max_speed(&self) -> f64 {
        self.max_speed
    }
}
