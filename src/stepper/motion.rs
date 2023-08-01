#[no_std]

use embassy_stm32::pwm::CaptureCompare16bitInstance;
use micromath::F32Ext;
use futures::join;
use core::f64::consts::PI;
use crate::stepper::a4988::{Stepper, StepperDirection};

use defmt::*;
use {defmt_rtt as _, panic_probe as _};

#[derive(Clone, Copy)]
pub struct Position1D{
    value: f64
}
impl Position1D{
    pub fn from_mm(value: f64) -> Position1D{
        Position1D { value }
    }

    pub fn to_mm(self) -> f64 {
        self.value
    }
}

#[derive(Clone, Copy)]
pub struct Position3D {
    x: Position1D,
    y: Position1D,
    z: Position1D,
}
impl Position3D{
    pub fn new(x: Position1D, y: Position1D, z: Position1D) -> Position3D{
        Position3D { x, y, z }
    }
    pub fn get_x(&self) -> Position1D{
        self.x
    }

    pub fn get_y(&self) -> Position1D{
        self.y
    }

    pub fn get_z(&self) -> Position1D{
        self.z
    }
}

#[derive(Clone, Copy)]
pub struct Speed {
    // mm per second
    value: f64
}

impl Speed {
    pub fn from_mmps(value: f64) -> Result<Speed, ()>{
        if value.is_sign_negative(){
            return Result::Err(());
        }
        Result::Ok(Speed{
            value
        })
    }

    pub fn to_mmps(&self) -> f64{
        self.value
    }
}

#[derive(Clone, Copy)]
pub struct Length{
    // mm
    value: f64
}

impl Length{
    pub fn from_mm(value: f64) -> Result<Length, ()>{
        if value.is_sign_negative(){
            return Result::Err(());
        }
        Result::Ok(Length{
            value
        })
    }

    pub fn to_mm(self) -> f64{
        self.value
    }
}

// TODO add the z axis handling
pub async fn move_to<X: CaptureCompare16bitInstance, Y: CaptureCompare16bitInstance>(dst: Position3D, speed: Speed, x_stepper: &mut Stepper<'_, '_, X>, y_stepper: &mut Stepper<'_, '_, Y>){
    info!("new move");
    let src = Position3D::new(x_stepper.get_position(), y_stepper.get_position(), Position1D::from_mm(0.0));
    let x_delta = dst.x.to_mm() - src.x.to_mm();
    let y_delta = dst.y.to_mm() - src.y.to_mm();

    let th = (y_delta as f32).atan2(x_delta as f32);

    let x_speed = speed.to_mmps() as f32 * th.cos();
    let x_direction = if x_speed.is_sign_negative() {StepperDirection::CounterClockwise} else {StepperDirection::Clockwise};

    info!("x speed: {} mm/s", x_speed);
    x_stepper.set_speed(Speed::from_mmps(x_speed.abs() as f64).unwrap());
    x_stepper.set_direction(x_direction);

    let y_speed = speed.to_mmps() as f32 * th.sin();
    let y_direction = if y_speed.is_sign_negative() {StepperDirection::CounterClockwise} else {StepperDirection::Clockwise};

    info!("y speed: {} mm/s", y_speed);
    y_stepper.set_speed(Speed::from_mmps(y_speed.abs() as f64).unwrap());
    y_stepper.set_direction(y_direction);

    let x_distance = Length::from_mm((x_delta as f32).abs() as f64).unwrap();
    let y_distance = Length::from_mm((y_delta as f32).abs() as f64).unwrap(); 
    join!(x_stepper.move_for(x_distance), y_stepper.move_for(y_distance));
}