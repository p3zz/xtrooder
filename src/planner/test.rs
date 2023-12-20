use defmt::assert_eq;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    pwm::{
        simple_pwm::{PwmPin, SimplePwm},
        CaptureCompare16bitInstance, Channel,
    },
    time::hz,
};
use embassy_time::{driver::now, Duration};

use crate::{stepper::a4988::{Stepper, StepperDirection}, math::{vector::Vector, common::StopWatch}};

async fn test_linear_move_to<'s, S: CaptureCompare16bitInstance>(stepper: &mut Stepper<'s, S>) {
    let mut stopwatch = StopWatch::new();
    stepper.set_speed(Vector::from_mm(10.0));
    stopwatch.start();
    stepper.move_to(Vector::from_mm(15.0)).await;
    let duration = stopwatch.measure();
    assert_eq!(duration.as_millis(), 1500);
    assert_eq!(stepper.get_position().to_mm(), 15.0);
    match stepper.get_direction(){
        StepperDirection::Clockwise => assert!(true),
        StepperDirection::CounterClockwise => assert!(false),
    }
    stopwatch.start();
    stepper.move_to(Vector::from_mm(-20.0)).await;
    let duration = stopwatch.measure();
    assert_eq!(duration.as_millis(), 2000);
    assert_eq!(stepper.get_position().to_mm(), -5.00);
    match stepper.get_direction(){
        StepperDirection::Clockwise => assert!(false),
        StepperDirection::CounterClockwise => assert!(true),
    }
}

// async fn test_linear_move_to<'s, S: CaptureCompare16bitInstance>(stepper: &mut Stepper<'s, S>) {
//     let destination = Vector::from_mm(15.0);
//     stepper.move_to(destination).await;
//     assert_eq!(stepper.get_position().to_mm(), destination.to_mm());
// }

pub async fn test() {
    let p = embassy_stm32::init(Default::default());

    let x_step = SimplePwm::new(
        p.TIM5,
        Some(PwmPin::new_ch1(p.PA0)),
        None,
        None,
        None,
        hz(1),
    );

    let x_dir = Output::new(p.PB0, Level::Low, Speed::Low);

    let mut x_stepper = Stepper::new(
        x_step,
        Channel::Ch1,
        x_dir.degrade(),
        200,
        Vector::from_mm(5.0),
    );
    test_linear_move_to(&mut x_stepper).await;
}
