use defmt::{assert, assert_eq, println};
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    pwm::{
        simple_pwm::{PwmPin, SimplePwm},
        CaptureCompare16bitInstance, Channel,
    },
    time::hz,
};

use crate::{
    math::{
        common::{abs, StopWatch},
        vector::{Vector, Vector2D},
    },
    planner::motion::{linear_move_to, linear_move_to_2d, linear_move_to_2d_e, linear_move_to_e},
    stepper::a4988::{Stepper, StepperDirection},
};

/**
 * Motion test suite
 * In 2d motion, an error has been used to cope with the inaccuracy of some of the micromath functions
 * An example is the sqrt, where the error is +-5% of the correct square root
 */

async fn test_linear_move_to<'s, S: CaptureCompare16bitInstance>(stepper: &mut Stepper<'s, S>) {
    println!("Test - Linear move to");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    linear_move_to(stepper, Vector::from_mm(15.0), Vector::from_mm(10.0)).await;
    let duration = stopwatch.measure();
    assert_eq!(duration.as_millis(), 1500);
    assert_eq!(stepper.get_position().to_mm(), 15.0);
    match stepper.get_direction() {
        StepperDirection::Clockwise => assert!(true),
        StepperDirection::CounterClockwise => assert!(false),
    };
    stopwatch.start();
    linear_move_to(stepper, Vector::from_mm(-5.0), Vector::from_mm(10.0)).await;
    let duration = stopwatch.measure();
    assert_eq!(duration.as_millis(), 2000);
    assert_eq!(stepper.get_position().to_mm(), -5.00);
    match stepper.get_direction() {
        StepperDirection::Clockwise => assert!(false),
        StepperDirection::CounterClockwise => assert!(true),
    };
}

async fn test_linear_move_to_e<
    's,
    A: CaptureCompare16bitInstance,
    E: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_e: &mut Stepper<'s, E>,
) {
    println!("Test - Linear move to e");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    linear_move_to_e(
        stepper_a,
        stepper_e,
        Vector::from_mm(10.0),
        Vector::from_mm(5.0),
        Vector::from_mm(10.0),
    )
    .await;
    let duration = stopwatch.measure();
    assert_eq!(duration.as_millis(), 1000);

    assert_eq!(stepper_a.get_position().to_mm(), 10.0);
    assert_eq!(stepper_a.get_speed().to_mm(), 10.0);
    match stepper_a.get_direction() {
        StepperDirection::Clockwise => assert!(true),
        StepperDirection::CounterClockwise => assert!(false),
    };

    assert_eq!(stepper_e.get_position().to_mm(), 5.0);
    assert_eq!(stepper_e.get_speed().to_mm(), 5.0);
    match stepper_a.get_direction() {
        StepperDirection::Clockwise => assert!(true),
        StepperDirection::CounterClockwise => assert!(false),
    };
}

async fn test_linear_move_to_2d<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
) {
    println!("Test - Linear move to 2d");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    linear_move_to_2d(
        stepper_a,
        stepper_b,
        Vector2D::new(Vector::from_mm(15.0), Vector::from_mm(-20.0)),
        Vector::from_mm(10.0),
    )
    .await;
    let duration = stopwatch.measure();
    let max_error = 0.05;

    assert!(abs(2500.0 - duration.as_millis() as f64) < 2500.0 * max_error);
    assert!(abs(15.0 - stepper_a.get_position().to_mm() as f64) < 15.0 * max_error);
    assert!(abs(6.0 - stepper_a.get_speed().to_mm()) < 6.0 * max_error);
    match stepper_a.get_direction() {
        StepperDirection::Clockwise => assert!(true),
        StepperDirection::CounterClockwise => assert!(false),
    };

    assert!(abs(-20.0 - stepper_b.get_position().to_mm() as f64) < 20.0 * max_error);
    assert!(abs(8.0 - stepper_b.get_speed().to_mm()) < 8.0 * max_error);
    match stepper_b.get_direction() {
        StepperDirection::Clockwise => assert!(false),
        StepperDirection::CounterClockwise => assert!(true),
    };
}

async fn test_linear_move_to_2d_e<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
    E: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    stepper_e: &mut Stepper<'s, E>,
) {
    println!("Test - Linear move to 2d e");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    linear_move_to_2d_e(
        stepper_a,
        stepper_b,
        stepper_e,
        Vector2D::new(Vector::from_mm(15.0), Vector::from_mm(-20.0)),
        Vector::from_mm(20.0),
        Vector::from_mm(10.0),
    )
    .await;
    let duration = stopwatch.measure();
    let max_error = 0.05;

    assert!(abs(2500.0 - duration.as_millis() as f64) < 2500.0 * max_error);
    assert!(abs(15.0 - stepper_a.get_position().to_mm() as f64) < 15.0 * max_error);
    assert!(abs(6.0 - stepper_a.get_speed().to_mm()) < 6.0 * max_error);
    match stepper_a.get_direction() {
        StepperDirection::Clockwise => assert!(true),
        StepperDirection::CounterClockwise => assert!(false),
    };

    assert!(abs(-20.0 - stepper_b.get_position().to_mm() as f64) < 20.0 * max_error);
    assert!(abs(8.0 - stepper_b.get_speed().to_mm()) < 8.0 * max_error);
    match stepper_b.get_direction() {
        StepperDirection::Clockwise => assert!(false),
        StepperDirection::CounterClockwise => assert!(true),
    };

    assert!(abs(20.0 - stepper_e.get_position().to_mm()) < 20.0 * max_error);
    assert!(abs(8.0 - stepper_e.get_speed().to_mm()) < 8.0 * max_error);
    match stepper_e.get_direction() {
        StepperDirection::Clockwise => assert!(true),
        StepperDirection::CounterClockwise => assert!(false),
    };
}

pub async fn test() {
    println!("Planner test");
    let p = embassy_stm32::init(Default::default());
    let distance_per_step = Vector::from_mm(0.5);
    let steps_per_revolution = 200;

    let a_step = SimplePwm::new(
        p.TIM5,
        Some(PwmPin::new_ch1(p.PA0)),
        None,
        None,
        None,
        hz(1),
    );

    let a_dir = Output::new(p.PB0, Level::Low, Speed::Low);

    let mut a_stepper = Stepper::new(
        a_step,
        Channel::Ch1,
        a_dir.degrade(),
        steps_per_revolution,
        distance_per_step,
    );

    let b_step = SimplePwm::new(
        p.TIM14,
        Some(PwmPin::new_ch1(p.PA7)),
        None,
        None,
        None,
        hz(1),
    );

    let b_dir = Output::new(p.PB3, Level::Low, Speed::Low);

    let mut b_stepper = Stepper::new(
        b_step,
        Channel::Ch1,
        b_dir.degrade(),
        steps_per_revolution,
        distance_per_step,
    );

    let c_step = SimplePwm::new(
        p.TIM15,
        Some(PwmPin::new_ch1(p.PA2)),
        None,
        None,
        None,
        hz(1),
    );

    let c_dir = Output::new(p.PB1, Level::Low, Speed::Low);

    let mut c_stepper = Stepper::new(
        c_step,
        Channel::Ch1,
        c_dir.degrade(),
        steps_per_revolution,
        distance_per_step,
    );

    test_linear_move_to(&mut a_stepper).await;
    a_stepper.reset();
    test_linear_move_to_e(&mut a_stepper, &mut b_stepper).await;
    a_stepper.reset();
    b_stepper.reset();
    test_linear_move_to_2d(&mut a_stepper, &mut b_stepper).await;
    a_stepper.reset();
    b_stepper.reset();
    test_linear_move_to_2d_e(&mut a_stepper, &mut b_stepper, &mut c_stepper).await;
    a_stepper.reset();
    b_stepper.reset();
    c_stepper.reset();
}
