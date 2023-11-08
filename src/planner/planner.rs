#![allow(dead_code)]

use embassy_stm32::pwm::CaptureCompare16bitInstance;
use crate::stepper::a4988::Stepper;
use crate::stepper::units::{Speed, Position2D, Position};
use super::motion;

use futures::join;

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

    pub async fn linear_move_xy(&mut self, dest: Position2D, feedrate: Speed){
        motion::linear_move_2d(&mut self.x_stepper, &mut self.y_stepper, dest, feedrate).await
    }


    pub async fn linear_move_xye(&mut self, dest: Position2D, feedrate: Speed, e_dst: Position){
        motion::linear_move_2d_e(&mut self.x_stepper, &mut self.y_stepper, &mut self.e_stepper, dest, e_dst, feedrate).await
    }

    pub async fn linear_move_xz(&mut self, dest: Position2D, feedrate: Speed){
        motion::linear_move_2d(&mut self.x_stepper, &mut self.z_stepper, dest, feedrate).await;
    }

    pub async fn linear_move_yz(&mut self, dest: Position2D, feedrate: Speed){
        motion::linear_move_2d(&mut self.y_stepper, &mut self.z_stepper, dest, feedrate).await;
    }
    

}