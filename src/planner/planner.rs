#![no_std]
use embassy_stm32::pwm::CaptureCompare16bitInstance;
use heapless::{String, Vec, LinearMap};
use crate::stepper::a4988::{Stepper, StepperDirection};
use crate::stepper::motion::{Position1D, Position3D, Speed, Length};
use micromath::F32Ext;
use futures::join;
use {defmt_rtt as _, panic_probe as _};
use defmt::*;

pub enum GCommand{
    G0{x: f64, y: f64, z: f64},
    G1{x: f64, y: f64, z: f64, e: f64, f: f64},
}

pub fn parse_line(line: String<64>) -> Result<GCommand, ()>{
    let tokens: Vec<String<8>, 16> = line.split(' ').map(String::from).collect();
    // cmd is a command 
    let mut cmd: LinearMap<&str, f64, 16> = LinearMap::new();
    if tokens.len() < 2{
        return Err(());
    }
    for t in &tokens{
        let key = match t.get(0..1){
            Some(v) => v,
            None => return Err(())
        };
        let value = match t.get(1..){
            Some(v) => match v.parse::<f64>(){
                Ok(n) => n,
                Err(_) => return Err(())
            },
            None => return Err(())
        };
        info!("key: {}, value: {}", key, value);
        cmd.insert(key, value).unwrap();
    }
    let code = (*cmd.get("G").unwrap()).to_bits();
    match code {
        0 => {
            let x = *cmd.get("X").unwrap();
            let y = *cmd.get("Y").unwrap();    
            let z = *cmd.get("Z").unwrap();
            Ok(GCommand::G0{ x, y, z })
        },
        1 => {
            let x = *cmd.get("X").unwrap();
            let y = *cmd.get("Y").unwrap();    
            let z = *cmd.get("Z").unwrap();
            let e = *cmd.get("E").unwrap();
            let f = *cmd.get("F").unwrap();
            Ok(GCommand::G1{x, y, z, e, f})
        },
        _ => Err(())
    }
}

pub struct Planner<'sx, 'dx, 'sy, 'dy, X, Y> {
    x_stepper: Stepper<'sx, 'dx, X>,
    y_stepper: Stepper<'sy, 'dy, Y>,
}
impl <'sx, 'dx, 'sy, 'dy, X, Y>Planner<'sx, 'dx, 'sy, 'dy, X, Y>
where X: CaptureCompare16bitInstance, Y: CaptureCompare16bitInstance{
    
    pub fn new(x_stepper: Stepper<'sx, 'dx, X>, y_stepper: Stepper<'sy, 'dy, Y>) -> Planner<'sx, 'dx, 'sy, 'dy, X, Y>{
        Planner{x_stepper, y_stepper}
    }

    pub async fn move_to(&mut self, dst: Position3D, speed: Speed){
        let src = Position3D::new(self.x_stepper.get_position(), self.y_stepper.get_position(), Position1D::from_mm(0.0));
        let x_delta = dst.get_x().to_mm() - src.get_x().to_mm();
        let y_delta = dst.get_y().to_mm() - src.get_y().to_mm();
    
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