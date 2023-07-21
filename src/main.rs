#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Peripheral;
use embassy_stm32::gpio::{Output, Level, Speed, AnyPin};
use embassy_stm32::peripherals::{TIM3, TIM5, PA6};
use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::Channel;
use embassy_stm32::time::{khz, hz, Hertz};
use embassy_time::{Duration, Timer};
use futures::join;
use {defmt_rtt as _, panic_probe as _};

// mod stepper;

async fn move_x(pwm: &mut SimplePwm<'_, TIM3>, duration: Duration, f: Hertz) {
    pwm.enable(Channel::Ch1);
    pwm.set_freq(f);
    Timer::after(duration).await;
    pwm.disable(Channel::Ch1)
}

async fn move_y(pwm: &mut SimplePwm<'_, TIM5>, duration: Duration, f: Hertz) {
    pwm.enable(Channel::Ch1);
    pwm.set_freq(f);
    Timer::after(duration).await;
    pwm.disable(Channel::Ch1)
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // let mut red = Output::new(p.PA0, Level::Low, Speed::Medium).degrade();
    // let mut green = Output::new(p.PA6, Level::Low, Speed::Medium).degrade();

    let mut red_pwm = SimplePwm::new(p.TIM3, Some(PwmPin::new_ch1(p.PA6)),
        None, None, None, hz(1));
    let red_max = red_pwm.get_max_duty();
    red_pwm.enable(Channel::Ch1);
    red_pwm.set_duty(Channel::Ch1, red_max/2);

    let mut green_pwm = SimplePwm::new(p.TIM5, Some(PwmPin::new_ch1(p.PA0)),
        None, None, None, hz(1));
    let green_max = green_pwm.get_max_duty();
    green_pwm.enable(Channel::Ch1);
    green_pwm.set_duty(Channel::Ch1, green_max/2);

    info!("PWM initialized");

    loop {
        info!("Main loop");
        join!(move_x(&mut red_pwm, Duration::from_millis(500), hz(20)),
            move_y(&mut green_pwm, Duration::from_millis(500), hz(10)));
        Timer::after(Duration::from_millis(500)).await;
    }
}