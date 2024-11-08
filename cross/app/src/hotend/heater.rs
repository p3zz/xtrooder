use embassy_stm32::timer::{simple_pwm::SimplePwm, Channel, GeneralInstance4Channel};
use embassy_time::Duration;
use math::measurements::Temperature;
use micromath::F32Ext;
use pid_lite::Controller;

use crate::config::{HeaterConfig, PidConfig};

pub struct Heater {
    ch: Channel,
    pid: Controller,
    target_temperature: Option<Temperature>,
}

impl Heater {
    pub fn new(ch: Channel, config: PidConfig) -> Heater {
        let pid = Controller::new(
            Temperature::from_celsius(30.0).as_celsius(),
            config.k_p,
            config.k_i,
            config.k_d,
        );
        Heater {
            ch,
            pid,
            target_temperature: None,
        }
    }

    pub fn enable<T: GeneralInstance4Channel>(&self, pwm: &mut SimplePwm<'_, T>) {
        pwm.enable(self.ch);
    }

    pub fn disable<T: GeneralInstance4Channel>(&self, pwm: &mut SimplePwm<'_, T>) {
        pwm.disable(self.ch);
    }

    pub fn is_enabled<T: GeneralInstance4Channel>(&self, pwm: &mut SimplePwm<'_, T>) -> bool {
        pwm.is_enabled(self.ch)
    }

    pub fn reset_target_temperature(&mut self) {
        self.target_temperature = None;
    }

    pub fn set_target_temperature(&mut self, temperature: Temperature) {
        self.target_temperature = Some(temperature);
        self.pid
            .set_target(self.target_temperature.unwrap().as_celsius());
    }

    pub fn update<T: GeneralInstance4Channel>(
        &mut self,
        tmp: Temperature,
        dt: Duration,
        pwm: &mut SimplePwm<'_, T>,
    ) -> Result<u32, ()> {
        if self.target_temperature.is_none() {
            return Err(());
        }

        let duty_cycle = self.pid.update_elapsed(
            tmp.as_celsius(),
            core::time::Duration::from_millis(dt.as_millis()),
        );

        // info!("duty cycle real value {}", duty_cycle);
        let duty_cycle = duty_cycle.max(0f64).min(f64::from(pwm.get_max_duty()));

        let duty_cycle = (duty_cycle as f32).trunc() as u32;

        // info!("duty cycle set to {}", duty_cycle);
        pwm.set_duty(self.ch, duty_cycle);

        Ok(duty_cycle)
    }
}
