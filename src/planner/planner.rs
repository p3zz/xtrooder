#![no_std]
use embassy_stm32::pwm::CaptureCompare16bitInstance;
use heapless::{String, Vec};
use crate::stepper::a4988::{Stepper, StepperDirection};
use crate::stepper::motion::{Position1D, Position3D, Speed, Length};
use micromath::F32Ext;
use futures::join;
use {defmt_rtt as _, panic_probe as _};
use defmt::*;

struct CommandParameter{
    key: String<8>,
    value: f64
}

pub fn parse_line(line: String<64>){
    let tokens: Vec<String<8>, 16> = line.split(' ').map(String::from).collect();
    let mut cmd: Vec<CommandParameter, 16> = Vec::new();
    for t in tokens{
        info!("Token: {}", t.as_str());
        let key = t.get(0..1).unwrap();
        info!("Key: {}", key);
        let value = t.get(1..).unwrap().parse::<f64>().unwrap();
        info!("value: {}", value);
        let param = CommandParameter{key: String::from(key), value};
        cmd.push(param);
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