#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Output, Level, Speed};
use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::Channel;
use embassy_stm32::time::hz;
use embassy_time::{Duration, Timer};
use futures::join;
use {defmt_rtt as _, panic_probe as _};

mod stepper;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // let mut red = Output::new(p.PA0, Level::Low, Speed::Medium).degrade();
    // let mut green = Output::new(p.PA6, Level::Low, Speed::Medium).degrade();

    let mut red_pwm = SimplePwm::new(p.TIM3, Some(PwmPin::new_ch1(p.PA6)),
        None, None, None, hz(1));
    let red_max = red_pwm.get_max_duty();
    red_pwm.set_duty(Channel::Ch1, red_max/2);

    let red_dir = Output::new(p.PB0, Level::Low, Speed::Low);

    let mut red_stepper = stepper::a4988::Stepper::new(red_pwm, red_dir.degrade());

    let mut green_pwm = SimplePwm::new(p.TIM5, Some(PwmPin::new_ch1(p.PA0)),
        None, None, None, hz(1));
    let green_max = green_pwm.get_max_duty();
    green_pwm.set_duty(Channel::Ch1, green_max/2);

    let green_dir = Output::new(p.PB14, Level::Low, Speed::Low);

    let mut green_stepper = stepper::a4988::Stepper::new(green_pwm, green_dir.degrade());

    loop {
        red_stepper.set_direction(stepper::a4988::StepperDirection::Clockwise);
        green_stepper.set_direction(stepper::a4988::StepperDirection::Clockwise);
        join!(red_stepper.step(), green_stepper.step());
        red_stepper.set_direction(stepper::a4988::StepperDirection::CounterClockwise);
        green_stepper.set_direction(stepper::a4988::StepperDirection::CounterClockwise);
        join!(red_stepper.step(), green_stepper.step());
    }
}