use embassy_stm32::{
    gpio::{Level, Output, Speed},
    pwm::{
        simple_pwm::{PwmPin, SimplePwm},
        CaptureCompare16bitInstance, Channel,
    },
    time::hz,
};

use crate::stepper::units::{Length, Speed as StepperSpeed};

use super::{a4988::Stepper, math::dps_from_radius};

async fn test_stepper_1<'s, S: CaptureCompare16bitInstance>(stepper: &mut Stepper<'s, S>) {
    stepper.reset();
    stepper.set_direction(super::a4988::StepperDirection::Clockwise);
    stepper.set_speed(StepperSpeed::from_mmps(10.0).unwrap());
    stepper.move_for(Length::from_mm(10.0).unwrap()).await;
}

pub async fn test() {
    let steps_per_revolution: u64 = 200;
    let pulley_radius: Length = Length::from_mm(5.0).unwrap();
    let distance_per_step = dps_from_radius(pulley_radius, steps_per_revolution);

    let p = embassy_stm32::init(Default::default());

    let step = SimplePwm::new(
        p.TIM3,
        Some(PwmPin::new_ch1(p.PA6)),
        None,
        None,
        None,
        hz(1),
    );

    let direction = Output::new(p.PB0, Level::Low, Speed::Low);

    let mut stepper = Stepper::new(
        step,
        Channel::Ch1,
        direction.degrade(),
        steps_per_revolution,
        distance_per_step,
    );

    test_stepper_1(&mut stepper).await
}
