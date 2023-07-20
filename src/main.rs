#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::TIM12;
use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::Channel;
use embassy_stm32::time::{khz, hz};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn pwm(mut pwm: SimplePwm<'static, TIM12>){
    pwm.set_freq(hz(10));
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let red_led = PwmPin::new_ch1(p.PB14);

    let mut simplepwm = SimplePwm::new(p.TIM12, Some(red_led), None, None, None, hz(1));

    let max = simplepwm.get_max_duty();
    simplepwm.enable(Channel::Ch1);
    simplepwm.set_duty(Channel::Ch1, max/2);

    let _ = _spawner.spawn(pwm(simplepwm));

    info!("PWM initialized");

    loop {
        info!("Main loop");
        Timer::after(Duration::from_millis(1000)).await;
    }
}