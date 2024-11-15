use embassy_stm32::timer::{simple_pwm::SimplePwm, Channel, GeneralInstance4Channel};
use fan::MyPwm;

pub struct SimplePwmWrapper<'a, T: GeneralInstance4Channel>{
    inner: SimplePwm<'a, T>
}

impl<'a, T: GeneralInstance4Channel> MyPwm<Channel> for SimplePwmWrapper<'a, T>{
    fn enable(&mut self, channel: Channel) {
        self.inner.enable(channel);
    }

    fn disable(&mut self, channel: Channel) {
        self.inner.disable(channel);
    }
    
    fn get_max_duty(&self) -> u64 {
        u64::from(self.inner.get_max_duty())
    }
    
    fn set_duty(&mut self, channel: Channel, duty_cycle: u64) {
        self.inner.set_duty(channel, duty_cycle as u32);
    }
}
