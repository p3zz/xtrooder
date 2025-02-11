use core::time::Duration;

use common::{PidConfig, PwmBase};
use math::{measurements::Temperature, pid::PID};

pub struct Heater<P: PwmBase> {
    ch: P::Channel,
    pid: PID,
    max_strength: f64
}

impl<P: PwmBase> Heater<P> {
    pub fn new(ch: P::Channel, config: PidConfig) -> Self {
        let pid = PID::new(config.k_p, config.k_i, config.k_d);
        Self { ch, pid, max_strength: 100.0 }
    }

    pub fn enable(&mut self, pwm: &mut P) {
        pwm.enable(self.ch);
    }

    pub fn disable(&self, pwm: &mut P) {
        pwm.disable(self.ch);
    }

    pub fn reset_target_temperature(&mut self) {
        self.pid.reset_target();
    }

    #[cfg(test)]
    pub fn get_target_temperature(&self) -> Option<Temperature> {
        self.pid.get_target().map(Temperature::from_celsius)
    }

    pub fn set_target_temperature(&mut self, temperature: Temperature) {
        self.pid.set_target(temperature.as_celsius());
    }

    #[cfg(test)]
    pub fn get_pid_target(&self) -> Option<f64> {
        self.pid.get_target()
    }

    pub fn set_duty_cycle(&self, duty_cycle: u64, pwm: &mut P) {
        pwm.set_duty(self.ch, duty_cycle);
    }

    pub fn set_strength(&self, value: f64, pwm: &mut P) {
        let value = value.max(0.0).min(self.max_strength);
        let strength = value / self.max_strength;
        let duty_cycle = pwm.get_max_duty() as f64 * strength;
        pwm.set_duty(self.ch, duty_cycle as u64);
    }

    pub fn update(&mut self, tmp: Temperature, dt: Duration, pwm: &mut P) -> Result<f64, ()> {
        self.pid.set_output_bounds(0f64, self.max_strength);
        let strength = self.pid.update(tmp.as_celsius(), dt)?;
        self.set_strength(strength, pwm);
        Ok(strength)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct PwmChannel {
        enabled: bool,
        duty_cycle: u64,
    }

    #[derive(Clone, Copy)]
    enum Channel {
        Ch1,
        Ch2,
        Ch3,
        Ch4,
    }

    struct PwmWrapper {
        pub ch1: PwmChannel,
        pub ch2: PwmChannel,
        pub ch3: PwmChannel,
        pub ch4: PwmChannel,
        pub max_duty: u64,
    }

    impl PwmWrapper {
        fn new() -> Self {
            Self {
                ch1: PwmChannel::default(),
                ch2: PwmChannel::default(),
                ch3: PwmChannel::default(),
                ch4: PwmChannel::default(),
                max_duty: 4096,
            }
        }
    }

    impl PwmBase for PwmWrapper {
        type Channel = Channel;

        fn enable(&mut self, channel: Self::Channel) {
            match channel {
                Channel::Ch1 => self.ch1.enabled = true,
                Channel::Ch2 => self.ch2.enabled = true,
                Channel::Ch3 => self.ch3.enabled = true,
                Channel::Ch4 => self.ch4.enabled = true,
            }
        }

        fn disable(&mut self, channel: Self::Channel) {
            match channel {
                Channel::Ch1 => self.ch1.enabled = false,
                Channel::Ch2 => self.ch2.enabled = false,
                Channel::Ch3 => self.ch3.enabled = false,
                Channel::Ch4 => self.ch4.enabled = false,
            }
        }

        fn get_max_duty(&self) -> u64 {
            self.max_duty
        }

        fn set_duty(&mut self, channel: Self::Channel, duty_cycle: u64) {
            match channel {
                Channel::Ch1 => self.ch1.duty_cycle = duty_cycle,
                Channel::Ch2 => self.ch2.duty_cycle = duty_cycle,
                Channel::Ch3 => self.ch3.duty_cycle = duty_cycle,
                Channel::Ch4 => self.ch4.duty_cycle = duty_cycle,
            }
        }
    }

    #[test]
    fn test_heater_enable() {
        let mut pwm = PwmWrapper::new();
        let mut heater: Heater<PwmWrapper> = Heater::new(
            Channel::Ch2,
            PidConfig {
                k_p: 30.0,
                k_i: 0.0,
                k_d: 3.0,
            },
        );
        assert!(!pwm.ch2.enabled);
        heater.enable(&mut pwm);
        assert!(pwm.ch2.enabled);
    }

    #[test]
    fn test_heater_disable() {
        let mut pwm = PwmWrapper::new();
        let mut heater: Heater<PwmWrapper> = Heater::new(
            Channel::Ch2,
            PidConfig {
                k_p: 30.0,
                k_i: 0.0,
                k_d: 3.0,
            },
        );
        heater.enable(&mut pwm);
        assert!(pwm.ch2.enabled);
        heater.disable(&mut pwm);
        assert!(!pwm.ch2.enabled);
    }

    #[test]
    fn test_heater_set_target_temperature() {
        let target = Temperature::from_celsius(150.0);
        let mut heater: Heater<PwmWrapper> = Heater::new(
            Channel::Ch2,
            PidConfig {
                k_p: 30.0,
                k_i: 0.0,
                k_d: 3.0,
            },
        );
        assert!(heater.get_target_temperature().is_none());
        heater.set_target_temperature(target);
        assert!(heater.get_pid_target().is_some());
        assert_eq!(target.as_celsius(), heater.get_pid_target().unwrap());
        assert!(heater.get_target_temperature().is_some());
        assert_eq!(target, heater.get_target_temperature().unwrap());
    }

    #[test]
    fn test_heater_reset_target_temperature() {
        let target = Temperature::from_celsius(150.0);
        let mut heater: Heater<PwmWrapper> = Heater::new(
            Channel::Ch2,
            PidConfig {
                k_p: 30.0,
                k_i: 0.0,
                k_d: 3.0,
            },
        );
        assert!(heater.get_target_temperature().is_none());
        heater.set_target_temperature(target);
        assert!(heater.get_target_temperature().is_some());
        heater.reset_target_temperature();
        assert!(heater.get_target_temperature().is_none());
    }

    #[test]
    fn test_heater_update() {
        let mut pwm = PwmWrapper::new();
        let target_temp = Temperature::from_celsius(150.0);
        let current_temp = Temperature::from_celsius(110.0);
        let mut heater: Heater<PwmWrapper> = Heater::new(
            Channel::Ch2,
            PidConfig {
                k_p: 30.0,
                k_i: 0.0,
                k_d: 0.1,
            },
        );
        heater.set_target_temperature(target_temp);
        let duty_cycle_new = heater.update(current_temp, Duration::from_millis(30), &mut pwm);
        assert!(duty_cycle_new.is_ok());
        // FIXME
        // assert_eq!(1333, duty_cycle_new.unwrap());
    }
}
