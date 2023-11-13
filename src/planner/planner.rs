#![allow(dead_code)]

use embassy_stm32::pwm::CaptureCompare16bitInstance;
use heapless::spsc::Queue;
use crate::parser::parser::GCommand;
use crate::stepper::a4988::Stepper;
use crate::stepper::units::Unit;
use crate::stepper::units::{Speed, Position2D, Position, Position3D};
use super::motion;

use futures::join;

pub enum Positioning {
    Relative,
    Absolute
}

// we need to have a triple(s, d, T) for every stepper
pub struct Planner<'sx, 'dx, 'sy, 'dy, 'sz, 'dz, 'se, 'de, X, Y, Z, E> {
    feedrate: Speed,
    unit: Unit,
    positioning: Positioning,
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
        Planner{
            x_stepper,
            y_stepper,
            z_stepper,
            e_stepper,
            command_queue: Queue::new(),
            running: false,
            feedrate: Speed::from_mmps(0.0).unwrap(),
            unit: Unit::Millimeter,
            positioning: Positioning::Absolute
        }
    }

    pub fn add_command(&mut self, command: GCommand) -> Result<(), GCommand>{
        self.command_queue.enqueue(command)
    }

    pub async fn execute(&mut self, command: GCommand){
        match command{
            GCommand::G0 { x, y, z, f } => self.g0(x, y, z, f).await,
            GCommand::G1 { x, y, z, e, f } => self.g1(x, y, z, e, f).await,
            GCommand::G20 => self.g20(),
            GCommand::G21 => self.g21(),
            GCommand::G90 => self.g90(),
            GCommand::G91 => self.g91(),
        }
    }

    fn g20(&mut self){
        self.unit = Unit::Inch;
    }

    fn g21(&mut self){
        self.unit = Unit::Inch;
    }

    fn g90(&mut self){
        self.positioning = Positioning::Absolute;
    }

    fn g91(&mut self){
        self.positioning = Positioning::Relative;
    }

    pub async fn g0(&mut self, x: Option<f64>, y: Option<f64>, z: Option<f64>, f: Option<f64>){
        self.feedrate = match f {
            Some(speed) => Speed::from_mmps(speed).unwrap(),
            None => self.feedrate
        };
        match (x,y,z){
            (None, None, None) => (),
            (None, None, Some(z)) => self.linear_move_z(Position::from_unit(z, self.unit), self.feedrate).await,
            (None, Some(y), None) => self.linear_move_y(Position::from_unit(y, self.unit), self.feedrate).await,
            (Some(x), None, None) => self.linear_move_x(Position::from_unit(x, self.unit), self.feedrate).await,
            (None, Some(y), Some(z)) => self.linear_move_yz(Position2D::new(Position::from_unit(y, self.unit), Position::from_unit(z, self.unit)), self.feedrate).await,
            (Some(x), None, Some(z)) => self.linear_move_xz(Position2D::new(Position::from_unit(x, self.unit), Position::from_unit(z, self.unit)), self.feedrate).await,
            (Some(x), Some(y), None) => self.linear_move_xy(Position2D::new(Position::from_unit(x, self.unit), Position::from_unit(y, self.unit)), self.feedrate).await,
            (Some(x), Some(y), Some(z)) => self.linear_move_xyz(Position3D::new(Position::from_unit(x, self.unit), Position::from_unit(y, self.unit), Position::from_unit(z, self.unit)), self.feedrate).await,
        }
    }

    pub async fn g1(&mut self, x: Option<f64>, y: Option<f64>, z: Option<f64>, e: Option<f64>, f: Option<f64>){
        let e_dest = match e {
            Some(e_dest) => Position::from_mm(e_dest),
            None => return self.g0(x, y, z, f).await,
        };

        self.feedrate = match f {
            Some(speed) => Speed::from_mmps(speed).unwrap(),
            None => self.feedrate
        };

        match (x,y,z){
            (None, None, None) => (),
            (None, None, Some(z)) => self.linear_move_ze(Position::from_unit(z, self.unit), e_dest, self.feedrate).await,
            (None, Some(y), None) => self.linear_move_ye(Position::from_unit(y, self.unit), e_dest, self.feedrate).await,
            (Some(x), None, None) => self.linear_move_xe(Position::from_unit(x, self.unit), e_dest, self.feedrate).await,
            (None, Some(y), Some(z)) => self.linear_move_yze(Position2D::new(Position::from_unit(y, self.unit), Position::from_unit(z, self.unit)), self.feedrate, e_dest ).await,
            (Some(x), None, Some(z)) => self.linear_move_xze(Position2D::new(Position::from_unit(x, self.unit), Position::from_unit(z, self.unit)), self.feedrate, e_dest).await,
            (Some(x), Some(y), None) => self.linear_move_xye(Position2D::new(Position::from_unit(x, self.unit), Position::from_unit(y, self.unit)), self.feedrate, e_dest).await,
            (Some(x), Some(y), Some(z)) => self.linear_move_xyze(Position3D::new(Position::from_unit(x, self.unit), Position::from_unit(y, self.unit), Position::from_unit(z, self.unit)), self.feedrate, e_dest).await,
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

    pub async fn linear_move_xe(&mut self, dest: Position, e_dest: Position, feedrate: Speed){
        motion::linear_move_e(&mut self.x_stepper, &mut self.e_stepper, dest, e_dest, feedrate).await
    }

    pub async fn linear_move_y(&mut self, dest: Position, feedrate: Speed){
        motion::linear_move(&mut self.y_stepper, dest, feedrate).await
    }

    pub async fn linear_move_ye(&mut self, dest: Position, e_dest: Position, feedrate: Speed){
        motion::linear_move_e(&mut self.y_stepper, &mut self.e_stepper, dest, e_dest, feedrate).await
    }

    pub async fn linear_move_z(&mut self, dest: Position, feedrate: Speed){
        motion::linear_move(&mut self.z_stepper, dest, feedrate).await
    }

    pub async fn linear_move_ze(&mut self, dest: Position, e_dest: Position, feedrate: Speed){
        motion::linear_move_e(&mut self.z_stepper, &mut self.e_stepper, dest, e_dest, feedrate).await
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

    pub async fn linear_move_xyz(&mut self, dest: Position3D, feedrate: Speed){
        todo!()
    }

    pub async fn linear_move_xyze(&mut self, dest: Position3D, feedrate: Speed, e_dst: Position){
        todo!()
    }
    

}