use core::time::Duration;

use common::{MyPwm, PidConfig};
use math::measurements::Temperature;
use pid_lite::Controller;

pub struct Heater<P: MyPwm>{
    ch: P::Channel,
    pid: Controller,
    target_temperature: Option<Temperature>,
}

impl <P: MyPwm> Heater<P>{
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


    pub fn get_target_temperature(&self) -> Option<Temperature>{
        self.target_temperature
    }

    pub fn set_target_temperature(&mut self, temperature: Temperature) {
        self.target_temperature.replace(temperature);
        // SAFETY: unwrap target temperature because it has been set the previous line
        self.pid
            .set_target(self.target_temperature.unwrap().as_celsius());
    }

    #[cfg(test)]
    pub fn get_pid_target(&self) -> f64{
        self.pid.target()
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

#[cfg(test)]
mod tests{
    use super::*;

    #[derive(Default)]
    struct PwmChannel{
        enabled: bool,
        duty_cycle: u64
    }

    #[derive(Clone, Copy)]
    enum Channel{
        Ch1,
        Ch2,
        Ch3,
        Ch4,
    }

    struct PwmWrapper{
        pub ch1: PwmChannel,
        pub ch2: PwmChannel,
        pub ch3: PwmChannel,
        pub ch4: PwmChannel,
        pub max_duty: u64
    }

    impl MyPwm for PwmWrapper{
        type Channel = Channel;
    
        type Pwm = ();
    
        fn new(_p: Self::Pwm) -> Self {
            Self { ch1: PwmChannel::default(), ch2: PwmChannel::default(), ch3: PwmChannel::default(), ch4: PwmChannel::default(), max_duty: 4096 }
        }
    
        fn enable(&mut self, channel: Self::Channel) {
            match channel{
                Channel::Ch1 => self.ch1.enabled = true,
                Channel::Ch2 => self.ch2.enabled = true,
                Channel::Ch3 => self.ch3.enabled = true,
                Channel::Ch4 => self.ch4.enabled = true,
            }
        }
    
        fn disable(&mut self, channel: Self::Channel) {
            match channel{
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
            match channel{
                Channel::Ch1 => self.ch1.duty_cycle = duty_cycle,
                Channel::Ch2 => self.ch2.duty_cycle = duty_cycle,
                Channel::Ch3 => self.ch3.duty_cycle = duty_cycle,
                Channel::Ch4 => self.ch4.duty_cycle = duty_cycle,
            }
        }
    }

    #[test]
    fn test_heater_enable(){
        let mut pwm = PwmWrapper::new(());
        let heater: Heater<PwmWrapper> = Heater::new(Channel::Ch2, PidConfig{k_p: 30.0, k_i: 0.0, k_d: 3.0});
        assert_eq!(false, pwm.ch2.enabled);
        heater.enable(&mut pwm);
        assert_eq!(true, pwm.ch2.enabled);
    }

    #[test]
    fn test_heater_disable(){
        let mut pwm = PwmWrapper::new(());
        let heater: Heater<PwmWrapper> = Heater::new(Channel::Ch2, PidConfig{k_p: 30.0, k_i: 0.0, k_d: 3.0});
        heater.enable(&mut pwm);
        assert_eq!(true, pwm.ch2.enabled);
        heater.disable(&mut pwm);
        assert_eq!(false, pwm.ch2.enabled);
    }


    #[test]
    fn test_heater_set_target_temperature(){
        let target = Temperature::from_celsius(150.0);
        let mut heater: Heater<PwmWrapper> = Heater::new(Channel::Ch2, PidConfig{k_p: 30.0, k_i: 0.0, k_d: 3.0});
        assert!(heater.get_target_temperature().is_none());
        heater.set_target_temperature(target);
        assert_eq!(target.as_celsius(), heater.get_pid_target());
        assert!(heater.get_target_temperature().is_some());
        assert_eq!(target, heater.get_target_temperature().unwrap());
    }

    #[test]
    fn test_heater_reset_target_temperature(){
        let target = Temperature::from_celsius(150.0);
        let mut heater: Heater<PwmWrapper> = Heater::new(Channel::Ch2, PidConfig{k_p: 30.0, k_i: 0.0, k_d: 3.0});
        assert!(heater.get_target_temperature().is_none());
        heater.set_target_temperature(target);
        assert!(heater.get_target_temperature().is_some());
        heater.reset_target_temperature();
        assert!(heater.get_target_temperature().is_none());
    }

    #[test]
    fn test_heater_update(){
        let mut pwm = PwmWrapper::new(());
        let target_temp = Temperature::from_celsius(150.0);
        let current_temp = Temperature::from_celsius(110.0);
        let mut heater: Heater<PwmWrapper> = Heater::new(Channel::Ch2, PidConfig{k_p: 30.0, k_i: 0.0, k_d: 3.0});
        heater.set_target_temperature(target_temp);
        let duty_cycle_new = heater.update(current_temp, Duration::from_millis(30), &mut pwm);
        assert!(duty_cycle_new.is_ok());
        assert_eq!(1204, duty_cycle_new.unwrap());
    }

}