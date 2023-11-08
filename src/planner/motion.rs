use crate::stepper::a4988::{Stepper, StepperDirection};
use crate::stepper::units::{Speed, Length, Position2D, Position};
use micromath::F32Ext;
use futures::join;
use embassy_stm32::pwm::CaptureCompare16bitInstance;

pub async fn linear_move_2d<'sa, 'da, 'sb, 'db, A: CaptureCompare16bitInstance, B: CaptureCompare16bitInstance>(stepper_a: &mut Stepper<'sa, 'da, A>, stepper_b: &mut Stepper<'sb, 'db, B>, dest: Position2D, feedrate: Speed){
    let src = Position2D::new(stepper_a.get_position(), stepper_b.get_position());
    let th = src.angle(dest);

    // compute the velocity out of the speed and its angle
    let a_f = feedrate.to_mmps() as f32 * th.cos();
    let a_feedrate = Speed::from_mmps(a_f.abs() as f64).unwrap();

    let b_f = feedrate.to_mmps() as f32 * th.sin();
    let b_feedrate = Speed::from_mmps(b_f.abs() as f64).unwrap();

    join!(linear_move(stepper_a, dest.get_x(), a_feedrate), linear_move(stepper_b, dest.get_y(), b_feedrate));
}

pub async fn linear_move<'s, 'd, T: CaptureCompare16bitInstance>(stepper: &mut Stepper<'s, 'd, T>, dest: Position, feedrate: Speed){
    let delta = dest.subtract(stepper.get_position());
    let direction = if delta.to_mm().is_sign_negative() {StepperDirection::CounterClockwise} else {StepperDirection::Clockwise};
    let distance = Length::from_mm((delta.to_mm() as f32).abs() as f64).unwrap();
    stepper.set_speed(feedrate);
    stepper.set_direction(direction);
    stepper.move_for(distance).await
}