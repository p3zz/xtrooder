#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::Channel;
use embassy_stm32::time::{khz, hz};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let red_led = PwmPin::new_ch1(p.PB14);
    let mut pwm = SimplePwm::new(p.TIM12, Some(red_led), None, None, None, hz(1));
    let max = pwm.get_max_duty();
    pwm.enable(Channel::Ch1);
    pwm.set_duty(Channel::Ch1, max/2);

    info!("PWM initialized");

    loop {
        info!("Slow for 1 second");
        pwm.set_freq(hz(10));
        Timer::after(Duration::from_millis(1000)).await;
        info!("Fast for 2 seconds");
        pwm.set_freq(hz(20));
        Timer::after(Duration::from_millis(2000)).await;
    }
}