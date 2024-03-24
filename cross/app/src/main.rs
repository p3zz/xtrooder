#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{gpio::{Level, Output, OutputType, Speed}, time::hz, timer::{simple_pwm::{PwmPin, SimplePwm}, Channel, CountingMode}};
use embassy_time::Timer;
use math::{distance::Distance, speed::Speed as StepperSpeed};
use stepper::a4988::Stepper;
mod hotend;
mod planner;
mod stepper;
mod utils;

// use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let a_step = SimplePwm::new(
        p.TIM5,
        Some(PwmPin::new_ch1(p.PA0, OutputType::PushPull)),
        None,
        None,
        None,
        hz(1),
        CountingMode::EdgeAlignedUp,
    );
        
    let a_dir = Output::new(p.PB0, Level::Low, Speed::Low);

    let mut a_stepper = Stepper::new(
        a_step,
        Channel::Ch1,
        a_dir,
        200,
        Distance::from_mm(1f64),
    );

    let mut speed = 50.0;
    let mut distance = 1000.0;

    loop {
        info!("loop");

        a_stepper.set_speed(StepperSpeed::from_mm_per_second(speed));

        match a_stepper.move_for(Distance::from_mm(distance)).await {
            Ok(_) => info!("move done"),
            Err(_) => info!("cannot move"),
            
        };

        if speed >= 500.0{
            speed = 10.0;
        }

        speed += 50.0;
        distance = -distance; 

        // a_stepper.set_speed(StepperSpeed::from_mm_per_second(100f64));

        // // Timer::after_millis(500).await;
        // match a_stepper.move_for(Distance::from_mm(-600f64)).await {
        //     Ok(_) => info!("move done"),
        //     Err(_) => info!("cannot move"),
            
        // };

        // Timer::after_millis(500).await;
    }
}
