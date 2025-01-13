#![no_std]
#![no_main]

use common::ExtiInputPinBase;
use embassy_futures::join::join;
use {
    app::{init_input_pin, init_stepper, ExtiInputPinWrapper, StepperTimer},
    defmt::info,
    defmt_rtt as _,
    embassy_executor::Spawner,
    embassy_stm32::{exti::ExtiInput, gpio::Pull},
    embassy_time::{Duration, Timer},
    math::{
        common::RotationDirection,
        measurements::{AngularVelocity, Distance, Length, Speed},
    },
    panic_probe as _,
    stepper::{
        motion::{auto_home, calibrate, linear_move_to},
        stepper::{StepperAttachment, StepperOptions, SteppingMode},
    },
};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let endstop_z1 = ExtiInput::new(p.PE7, p.EXTI7, Pull::Down);
    let endstop_z1 = init_input_pin!(endstop_z1);

    let mut stepper_z1 = init_stepper!(
        p.PG2,
        p.PG3,
        StepperOptions {
            steps_per_revolution: 200,
            stepping_mode: SteppingMode::HalfStep,
            bounds: Some((
                Distance::from_millimeters(-150.0),
                Distance::from_millimeters(150.0)
            )),
            positive_direction: RotationDirection::Clockwise,
            acceleration: None
        },
        StepperAttachment {
            distance_per_step: Length::from_millimeters(0.15)
        }
    );

    let endstop_z2 = ExtiInput::new(p.PE8, p.EXTI8, Pull::Down);
    let endstop_z2 = init_input_pin!(endstop_z2);

    let mut stepper_z2 = init_stepper!(
        p.PD2,
        p.PC12,
        StepperOptions {
            steps_per_revolution: 200,
            stepping_mode: SteppingMode::HalfStep,
            bounds: Some((
                Distance::from_millimeters(-150.0),
                Distance::from_millimeters(150.0)
            )),
            positive_direction: RotationDirection::Clockwise,
            acceleration: None
        },
        StepperAttachment {
            distance_per_step: Length::from_millimeters(0.15)
        }
    );

    loop {
        #[cfg(feature = "defmt-log")]
        info!("waiting for endstop hit");
        let res = join(
            calibrate::<_, _, StepperTimer>(&mut stepper_z1, &endstop_z1),
            calibrate::<_, _, StepperTimer>(&mut stepper_z2, &endstop_z2),
        )
        .await;
        info!("endstop hit!");
        Timer::after(Duration::from_millis(500)).await;
        let res = join(
            stepper_z1.home::<StepperTimer>(),
            stepper_z2.home::<StepperTimer>(),
        )
        .await;
        Timer::after(Duration::from_millis(1000)).await;
    }
}
