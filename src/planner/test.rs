use defmt::{assert, assert_eq};
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    pwm::{
        simple_pwm::{PwmPin, SimplePwm},
        CaptureCompare16bitInstance,
    },
    time::hz,
};

use crate::stepper::{a4988::Stepper, units::Length};

use super::motion::linear_move_to;

async fn test_linear_move_to<'s, S: CaptureCompare16bitInstance>(stepper: &mut Stepper<'s, S>) {
    let distance = Length::from_mm(15.0).unwrap();
    stepper.move_for(distance).await;
    assert_eq!(stepper.get_position().to_mm(), 15.0);
}

pub fn test() {
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

    let mut x_stepper = Stepper::new(x_step, x_dir.degrade(), 200, Length::from_mm(5.0).unwrap());
    test_linear_move_to(&mut x_stepper);
}
