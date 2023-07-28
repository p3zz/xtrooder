#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Output, Level, Speed};
use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::Channel;
use embassy_stm32::time::hz;
use futures::join;
use {defmt_rtt as _, panic_probe as _};

mod stepper;
use stepper::a4988::{Length, Stepper, StepperDirection, dps_from_radius, Speed as StepperSpeed};

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

    let mut red_stepper = Stepper::new(red_pwm, red_dir.degrade(), 200, dps_from_radius(Length::from_mm(5.0), 200));

    let mut green_pwm = SimplePwm::new(p.TIM5, Some(PwmPin::new_ch1(p.PA0)),
        None, None, None, hz(1));
    let green_max = green_pwm.get_max_duty();
    green_pwm.set_duty(Channel::Ch1, green_max/2);

    let green_dir = Output::new(p.PB14, Level::Low, Speed::Low);

    let mut green_stepper = Stepper::new(green_pwm, green_dir.degrade(), 200, dps_from_radius(Length::from_mm(5.0), 200));

    loop {
        red_stepper.set_speed(StepperSpeed::from_rps(10));
        red_stepper.set_direction(StepperDirection::Clockwise);
        green_stepper.set_speed(StepperSpeed::from_rps(10));
        green_stepper.set_direction(StepperDirection::Clockwise);
        join!(red_stepper.move_for(Length::from_mm(10.0)), green_stepper.move_for(Length::from_mm(20.0)));
        red_stepper.set_speed(StepperSpeed::from_rps(1));
        red_stepper.set_direction(StepperDirection::CounterClockwise);
        red_stepper.set_speed(StepperSpeed::from_rps(1));
        green_stepper.set_direction(StepperDirection::CounterClockwise);
        join!(red_stepper.move_for(Length::from_mm(10.0)), green_stepper.move_for(Length::from_mm(20.0)));
    }
}