#![allow(dead_code)]

use embassy_stm32::pwm::CaptureCompare16bitInstance;
use crate::stepper::a4988::{Stepper, StepperDirection};
use crate::stepper::motion::{Position, Position3D, Speed, Length};
use micromath::F32Ext;
use futures::join;
use {defmt_rtt as _, panic_probe as _};

// we need to have a triple(s, d, T) for every stepper
pub struct Planner<'sx, 'dx, 'sy, 'dy, 'sz, 'dz, 'se, 'de, X, Y, Z, E> {
    x_stepper: Stepper<'sx, 'dx, X>,
    y_stepper: Stepper<'sy, 'dy, Y>,
    z_stepper: Stepper<'sz, 'dz, Z>,
    e_stepper: Stepper<'se, 'de, E>,
}
impl <'sx, 'dx, 'sy, 'dy, 'sz, 'dz, 'se, 'de, X, Y, Z, E> Planner <'sx, 'dx, 'sy, 'dy, 'sz, 'dz, 'se, 'de, X, Y, Z, E>
where X: CaptureCompare16bitInstance, Y: CaptureCompare16bitInstance, Z: CaptureCompare16bitInstance, E: CaptureCompare16bitInstance{
    
    pub fn new(
        x_stepper: Stepper<'sx, 'dx, X>,
        y_stepper: Stepper<'sy, 'dy, Y>,
        z_stepper: Stepper<'sz, 'dz, Z>,
        e_stepper: Stepper<'se, 'de, E>
    ) -> Planner<'sx, 'dx, 'sy, 'dy, 'sz, 'dz, 'se, 'de, X, Y, Z, E>{
        Planner{x_stepper, y_stepper, z_stepper, e_stepper}
    }

    pub async fn move_to(&mut self, x: Option<Position>, y: Option<Position>, z: Option<Position>, speed: Option<Speed>, extruder_dst: Option<Position>){
        let src = Position3D::new(self.x_stepper.get_position(), self.y_stepper.get_position(), self.z_stepper.get_position());
        let delta = dst.subtract(src);

    
        let th = (y_delta as f32).atan2(x_delta as f32);
    
        let x_speed = speed.to_mmps() as f32 * th.cos();
        let x_direction = if x_speed.is_sign_negative() {StepperDirection::CounterClockwise} else {StepperDirection::Clockwise};
    
        self.x_stepper.set_speed(Speed::from_mmps(x_speed.abs() as f64).unwrap());
        self.x_stepper.set_direction(x_direction);
    
        let y_speed = speed.to_mmps() as f32 * th.sin();
        let y_direction = if y_speed.is_sign_negative() {StepperDirection::CounterClockwise} else {StepperDirection::Clockwise};
    
        self.y_stepper.set_speed(Speed::from_mmps(y_speed.abs() as f64).unwrap());
        self.y_stepper.set_direction(y_direction);
    
        let x_distance = Length::from_mm((x_delta as f32).abs() as f64).unwrap();
        let y_distance = Length::from_mm((y_delta as f32).abs() as f64).unwrap(); 
        join!(self.x_stepper.move_for(x_distance), self.y_stepper.move_for(y_distance));
    }

}