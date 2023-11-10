#![allow(dead_code)]

use embassy_stm32::pwm::CaptureCompare16bitInstance;
use heapless::spsc::Queue;
use crate::parser::parser::GCommand;
use crate::stepper::a4988::Stepper;
use crate::stepper::units::{Speed, Position2D, Position};
use super::motion;

use futures::join;

// we need to have a triple(s, d, T) for every stepper
pub struct Planner<'sx, 'dx, 'sy, 'dy, 'sz, 'dz, 'se, 'de, X, Y, Z, E> {
    command_queue: Queue<GCommand, 16>,
    running: bool,
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
        Planner{x_stepper, y_stepper, z_stepper, e_stepper, command_queue: Queue::new(), running: false}
    }

    pub fn add_command(&mut self, command: GCommand) -> Result<(), GCommand>{
        self.command_queue.enqueue(command)
    }

    pub async fn execute(&mut self, command: GCommand){
        match command{
            GCommand::G0 { x, y, z, f } => self.g0(x, y, z, f).await,
            GCommand::G1 { x, y, z, e, f } => todo!(),
        }
    }

    pub async fn g0(&mut self, x: Option<f64>, y: Option<f64>, z: Option<f64>, f: Option<f64>){
        match (x,y,z){
            (None, None, None) => todo!(),
            (None, None, Some(z)) => todo!(),
            (None, Some(_), None) => todo!(),
            (None, Some(_), Some(_)) => todo!(),
            (Some(_), None, None) => todo!(),
            (Some(_), None, Some(_)) => todo!(),
            (Some(_), Some(_), None) => todo!(),
            (Some(_), Some(_), Some(_)) => todo!(),
        }
    }

    pub async fn start(&mut self){
        self.running = true;
        self.process().await
    }

    pub async fn stop(&mut self){
        self.running = false;
    }

    async fn process(&mut self){
        while self.running {
            match self.command_queue.dequeue(){
                Some(cmd) => self.execute(cmd).await,
                None => (),
            };
        }
    }

    pub async fn linear_move_x(&mut self, dest: Position, feedrate: Speed){
        motion::linear_move(&mut self.x_stepper, dest, feedrate).await
    }

    pub async fn linear_move_y(&mut self, dest: Position, feedrate: Speed){
        motion::linear_move(&mut self.y_stepper, dest, feedrate).await
    }

    pub async fn linear_move_z(&mut self, dest: Position, feedrate: Speed){
        motion::linear_move(&mut self.z_stepper, dest, feedrate).await
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

    pub async fn linear_move_xze(&mut self, dest: Position2D, feedrate: Speed, e_dst: Position){
        motion::linear_move_2d_e(&mut self.x_stepper, &mut self.z_stepper, &mut self.e_stepper, dest, e_dst, feedrate).await
    }

    pub async fn linear_move_yz(&mut self, dest: Position2D, feedrate: Speed){
        motion::linear_move_2d(&mut self.y_stepper, &mut self.z_stepper, dest, feedrate).await;
    }

    pub async fn linear_move_yze(&mut self, dest: Position2D, feedrate: Speed, e_dst: Position){
        motion::linear_move_2d_e(&mut self.y_stepper, &mut self.z_stepper, &mut self.e_stepper, dest, e_dst, feedrate).await
    }
    

}