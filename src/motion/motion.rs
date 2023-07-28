use embassy_stm32::pwm::{simple_pwm::SimplePwm, CaptureCompare16bitInstance};
use crate::stepper::a4988::{Stepper, Length, Speed, StepperDirection};
use micromath::F32Ext;
use futures::join;

pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}

impl Position{
    pub fn new(x: f64, y: f64, z: f64) -> Position{
        Position { x, y, z }
    }
}

pub async fn move_to<X: CaptureCompare16bitInstance, Y: CaptureCompare16bitInstance>(dst: Position, speed: Speed, x_stepper: &mut Stepper<'_, '_, X>, y_stepper: &mut Stepper<'_, '_, Y>){
    let src = Position::new(x_stepper.get_position().to_mm(), y_stepper.get_position().to_mm(), 0.0);
    let x_delta = Length::from_mm(dst.x - src.x);
    let y_delta = Length::from_mm(dst.y - src.y);
    let th = (x_delta.to_mm() as f32).atan2(y_delta.to_mm() as f32);

    let x_speed = Speed::from_rps((speed.to_rps() as f32 * th.cos()) as u64);
    let x_direction = if x_delta.to_mm() < 0.0 {StepperDirection::Clockwise} else {StepperDirection::CounterClockwise};  
    x_stepper.set_speed(x_speed);
    x_stepper.set_direction(x_direction);

    let y_speed = Speed::from_rps((speed.to_rps() as f32 * th.sin()) as u64);
    let y_direction = if x_delta.to_mm() < 0.0 {StepperDirection::Clockwise} else {StepperDirection::CounterClockwise};
    y_stepper.set_speed(y_speed);
    y_stepper.set_direction(y_direction);

    join!(x_stepper.move_for(x_delta), y_stepper.move_for(y_delta));
}