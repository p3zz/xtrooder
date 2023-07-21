#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::{TIM12, TIM5};
use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::Channel;
use embassy_stm32::time::{khz, hz};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn red_pwm_task(mut pwm: SimplePwm<'static, TIM12>){
    pwm.set_freq(hz(10));
}

#[embassy_executor::task]
async fn green_pwm_task(mut pwm: SimplePwm<'static, TIM5>){
    pwm.set_freq(hz(10));
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut red_pwm = SimplePwm::new(p.TIM12, Some(PwmPin::new_ch1(p.PB14)), None, None, None, hz(1));
    let red_max = red_pwm.get_max_duty();
    red_pwm.enable(Channel::Ch1);
    red_pwm.set_duty(Channel::Ch1, red_max/2);

    let mut green_pwm = SimplePwm::new(p.TIM5, Some(PwmPin::new_ch1(p.PA0)), None, None, None, hz(1));
    let green_max = green_pwm.get_max_duty();
    green_pwm.enable(Channel::Ch1);
    green_pwm.set_duty(Channel::Ch1, green_max/2);

    let _ = _spawner.spawn(red_pwm_task(red_pwm));

    let _ = _spawner.spawn(green_pwm_task(green_pwm));

    info!("PWM initialized");

    loop {
        info!("Main loop");
        Timer::after(Duration::from_millis(1000)).await;
    }
}