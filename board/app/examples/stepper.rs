#![no_std]
#![no_main]

use app::{init_stepper, ExtiInputPinWrapper};
use common::{OutputPinBase, TimerBase};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed as PinSpeed};
use embassy_time::{Duration, Timer};
use math::{
    common::RotationDirection,
    measurements::{AngularVelocity, Distance, Length, Speed},
    vector::{Vector2D, Vector3D},
};
use stepper::{
    motion::{linear_move_to, linear_move_to_2d, linear_move_to_3d},
    stepper::{Stepper, StepperAttachment, StepperOptions, SteppingMode},
};

use {defmt_rtt as _, panic_probe as _};

#[cfg(feature = "defmt-log")]
use defmt::info;

struct StepperTimer {}

impl TimerBase for StepperTimer {
    async fn after(duration: core::time::Duration) {
        let duration = embassy_time::Duration::from_micros(duration.as_micros() as u64);
        Timer::after(duration).await
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(embassy_stm32::Config::default());

    let mut stepper_x = init_stepper!(
        p.PC11,
        p.PC10,
        StepperOptions {
            steps_per_revolution: 200,
            stepping_mode: SteppingMode::HalfStep,
            bounds: None,
            positive_direction: RotationDirection::CounterClockwise,
            acceleration: None
        },
        StepperAttachment {
            distance_per_step: Length::from_millimeters(0.15)
        }
    );
    stepper_x.set_speed(AngularVelocity::from_rpm(60.0));
    // let mut stepper_y = init_stepper!(
    //     p.PC11,
    //     p.PC10,
    //     StepperOptions {
    //         steps_per_revolution: 200,
    //         stepping_mode: SteppingMode::FullStep,
    //         bounds: None,
    //         positive_direction: RotationDirection::CounterClockwise,
    //         acceleration: Some(AngularVelocity::from_rpm(6.0))
    //     },
    //     StepperAttachment {
    //         distance_per_step: Length::from_millimeters(0.15)
    //     }
    // );
    // stepper_y.set_speed(AngularVelocity::from_rpm(200.0));
    // let mut stepper_z = init_stepper!(p.PD2, p.PC12, StepperOptions::default(), StepperAttachment {
    //     distance_per_step: Length::from_millimeters(0.15)
    // });
    Timer::after_millis(1000).await;

    loop {
        stepper_x.set_direction(RotationDirection::Clockwise);
        if let Ok(d) = stepper_x.move_for_steps::<StepperTimer>(600).await {
            #[cfg(feature = "defmt-log")]
            info!("duration: {}", d);
        }
        stepper_x.set_direction(RotationDirection::CounterClockwise);
        if let Ok(d) = stepper_x.move_for_steps::<StepperTimer>(600).await {
            #[cfg(feature = "defmt-log")]
            info!("duration: {}", d);
        }
        // stepper_x.set_direction(RotationDirection::CounterClockwise);
        // if let Ok(d) = stepper_x.move_for_steps_accelerated::<StepperTimer>(
        //     500,
        //     AngularVelocity::from_rpm(30.0)).await{
        //     #[cfg(feature="defmt-log")]
        //     info!("duration: {}", d);
        // }

        // stepper_y.set_direction(RotationDirection::Clockwise);
        // if let Ok(d) = stepper_y.move_for_steps_accelerated::<StepperTimer>(
        //     500,
        //     AngularVelocity::from_rpm(30.0)).await{
        //     #[cfg(feature="defmt-log")]
        //     info!("duration: {}", d);
        // }
        // stepper_y.set_direction(RotationDirection::CounterClockwise);
        // if let Ok(d) = stepper_y.move_for_steps_accelerated::<StepperTimer>(
        //     500,
        //     AngularVelocity::from_rpm(30.0)).await{
        //     #[cfg(feature="defmt-log")]
        //     info!("duration: {}", d);
        // }

        // if let Ok(d) = stepper_x.move_for_steps::<StepperTimer>(500).await{
        // }

        // stepper_x.set_direction(RotationDirection::CounterClockwise);

        // if let Ok(d) = stepper_x.move_for_steps::<StepperTimer>(500).await{
        // }

        // stepper_x.set_direction(RotationDirection::Clockwise);
        // stepper_x.move_for_steps::<StepperTimer>(400).await;
        // stepper_x.set_direction(RotationDirection::CounterClockwise);
        // stepper_x.move_for_steps::<StepperTimer>(400).await;

        // if let Ok(d) = linear_move_to::<_, StepperTimer, ExtiInputPinWrapper>(
        //     &mut stepper_x,
        //     Distance::from_centimeters(3.0),
        //     Speed::from_meters_per_second(0.17),
        //     &mut None).await{
        // }

        // #[cfg(feature="defmt-log")]
        // info!("position: {}\tsteps: {}", stepper_x.get_position().as_millimeters(), stepper_x.get_steps());

        // if let Ok(d) = linear_move_to::<_, StepperTimer, ExtiInputPinWrapper>(
        //     &mut stepper_x,
        //     Distance::from_centimeters(-3.0),
        //     Speed::from_meters_per_second(0.17),
        //     &mut None).await{
        // }

        // #[cfg(feature="defmt-log")]
        // info!("position: {}\tsteps: {}", stepper_x.get_position().as_millimeters(), stepper_x.get_steps());

        // if let Ok(d) = linear_move_to::<_, StepperTimer, ExtiInputPinWrapper>(
        //     &mut stepper_y,
        //     Distance::from_centimeters(5.0),
        //     Speed::from_meters_per_second(0.2),
        //     &mut None).await{
        //     #[cfg(feature="defmt-log")]
        //     info!("duration: {}", d.as_millis());
        // }
        // if let Ok(d) = linear_move_to::<_, StepperTimer, ExtiInputPinWrapper>(
        //     &mut stepper_y,
        //     Distance::from_centimeters(-5.0),
        //     Speed::from_meters_per_second(0.2),
        //     &mut None).await{
        //     #[cfg(feature="defmt-log")]
        //     info!("duration: {}", d.as_millis());
        // }

        // if let Ok(d) = linear_move_to::<_, StepperTimer, ExtiInputPinWrapper>(
        //     &mut stepper_z,
        //     Distance::from_centimeters(10.0),
        //     Speed::from_meters_per_second(0.1),
        //     &mut None).await{
        //     #[cfg(feature="defmt-log")]
        //     info!("duration: {}", d.as_millis());
        // }
        // if let Ok(d) = linear_move_to::<_, StepperTimer, ExtiInputPinWrapper>(
        //     &mut stepper_z,
        //     Distance::from_centimeters(0.0),
        //     Speed::from_meters_per_second(0.1),
        //     &mut None).await{
        //     #[cfg(feature="defmt-log")]
        //     info!("duration: {}", d.as_millis());
        // }

        // if let Ok(d) = linear_move_to_2d::<_, StepperTimer, ExtiInputPinWrapper>(
        //     (&mut stepper_x, &mut stepper_y),
        //     Vector2D::new(
        //         Distance::from_centimeters(5.0),
        //         Distance::from_centimeters(4.0),
        //     ),
        //     Speed::from_meters_per_second(0.18),
        //     (&mut None, &mut None),
        // )
        // .await
        // {
        //     #[cfg(feature = "defmt-log")]
        //     info!("duration: {}", d.as_millis());
        // }

        // if let Ok(d) = linear_move_to_2d::<_, StepperTimer, ExtiInputPinWrapper>(
        //     (&mut stepper_x, &mut stepper_y),
        //     Vector2D::new(
        //         Distance::from_centimeters(-5.0),
        //         Distance::from_centimeters(-4.0),
        //     ),
        //     Speed::from_meters_per_second(0.18),
        //     (&mut None, &mut None),
        // )
        // .await
        // {
        //     #[cfg(feature = "defmt-log")]
        //     info!("duration: {}", d.as_millis());
        // }
        // #[cfg(feature="defmt-log")]
        // info!("Position: {}", stepper.get_position().as_millimeters());

        // if let Ok(d) = linear_move_to_3d::<_, StepperTimer, ExtiInputPinWrapper>(
        //     (&mut stepper_x, &mut stepper_y, &mut stepper_z),
        //     Vector3D::new(Distance::from_centimeters(-5.0), Distance::from_centimeters(-4.0), Distance::from_centimeters(6.0)),
        //     Speed::from_meters_per_second(0.12), (&mut None, &mut None, &mut None)
        // ).await{
        //     #[cfg(feature="defmt-log")]
        //     info!("duration: {}", d.as_millis());
        // }

        // if let Ok(d) = linear_move_to_3d::<_, StepperTimer, ExtiInputPinWrapper>(
        //     (&mut stepper_x, &mut stepper_y, &mut stepper_z),
        //     Vector3D::new(Distance::from_centimeters(0.0), Distance::from_centimeters(0.0), Distance::from_centimeters(0.0)),
        //     Speed::from_meters_per_second(0.12), (&mut None, &mut None, &mut None)
        // ).await{
        //     #[cfg(feature="defmt-log")]
        //     info!("duration: {}", d.as_millis());
        // }

        // #[cfg(feature="defmt-log")]
        // info!("Position: {}", stepper.get_position().as_millimeters());

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

        // Timer::after(Duration::from_millis(100)).await;
    }
}
