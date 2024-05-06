#![no_std]
#![no_main]

use app::stepper::{
    self,
    a4988::{Stepper, StepperAttachment, StepperOptions, SteppingMode},
};
use defmt::info;
use embassy_stm32::{
    gpio::{Level, Output, OutputType, Speed as PinSpeed},
    time::hz,
    timer::{
        simple_pwm::{PwmPin, SimplePwm},
        Channel, CountingMode,
    },
};
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use math::{common::RotationDirection, distance::Distance, speed::Speed};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(embassy_stm32::Config::default());

    // let step = SimplePwm::new(
    //     p.TIM5,
    //     Some(PwmPin::new_ch1(p.PA0, OutputType::PushPull)),
    //     None,
    //     None,
    //     None,
    //     hz(1),
    //     CountingMode::EdgeAlignedUp,
    // );

    let step = Output::new(p.PA0, Level::Low, PinSpeed::Low);

    let dir = Output::new(p.PB0, Level::Low, PinSpeed::Low);

    let mut stepper = Stepper::new(
        step,
        dir,
        StepperOptions::default()
    );

    stepper.set_stepping_mode(SteppingMode::HalfStep);

    stepper.set_attachment(StepperAttachment::default());

    stepper.set_speed(1.0);

    // let mut d = Distance::from_mm(80.0);

    loop {
        stepper.set_direction(RotationDirection::CounterClockwise);
        if let Err(_) = stepper.move_for_steps(50).await{
            info!("Cannot move");
        };

        Timer::after(Duration::from_millis(100)).await;

        stepper.set_direction(RotationDirection::Clockwise);

        if let Err(_) = stepper.move_for_steps(50).await{
            info!("Cannot move");
        };
        
        // info!("Moving to {}mm", d.to_mm());
        // if let Err(e) = stepper.move_to_destination(d).await {
        //     match e {
        //         stepper::a4988::StepperError::MoveTooShort => info!("Move too short"),
        //         stepper::a4988::StepperError::MoveOutOfBounds => info!("Move out of bounds"),
        //         stepper::a4988::StepperError::MoveNotValid => info!("Move not valid"),
        //         stepper::a4988::StepperError::MissingAttachment => info!("Missing attachment"),
        //     }
        // };

        // d = Distance::from_mm(-d.to_mm());

        Timer::after(Duration::from_millis(100)).await;
    }
}
