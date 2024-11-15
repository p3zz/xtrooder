use core::time::Duration;

use common::{MyPwm, PidConfig};
use math::measurements::Temperature;
use pid_lite::Controller;

pub struct HeaterController<P: MyPwm>{
    ch: P::Channel,
    pid: Controller,
    target_temperature: Option<Temperature>,
}

impl <P: MyPwm> HeaterController<P>{
    pub fn new(ch: P::Channel, config: PidConfig) -> Self {
        let pid = Controller::new(
            Temperature::from_celsius(30.0).as_celsius(),
            config.k_p,
            config.k_i,
            config.k_d,
        );
        Self {
            ch,
            pid,
            target_temperature: None,
        }
    }

    pub fn enable(&self, pwm: &mut P) {
        pwm.enable(self.ch);
    }

    pub fn disable(&self, pwm: &mut P) {
        pwm.disable(self.ch);
    }

    pub fn reset_target_temperature(&mut self) {
        self.target_temperature = None;
    }

    pub fn set_target_temperature(&mut self, temperature: Temperature) {
        self.target_temperature.replace(temperature);
        // SAFETY: unwrap target temperature because it has been set the previous line
        self.pid
            .set_target(self.target_temperature.unwrap().as_celsius());
    }

    pub fn update(
        &mut self,
        tmp: Temperature,
        dt: Duration,
        pwm: &mut P,
    ) -> Result<u64, ()> {
        if self.target_temperature.is_none() {
            return Err(());
        }

        let duty_cycle = self.pid.update_elapsed(
            tmp.as_celsius(),
            dt,
        );

        let duty_cycle = duty_cycle.max(0.0).min(pwm.get_max_duty() as f64) as u64;

        pwm.set_duty(self.ch, duty_cycle);

        Ok(duty_cycle)
    }
}