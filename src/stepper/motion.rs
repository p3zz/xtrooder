#[no_std]

use embassy_stm32::pwm::CaptureCompare16bitInstance;
use micromath::F32Ext;
use futures::join;
use core::f64::consts::PI;
use crate::stepper::a4988::Stepper;

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
    pub fn from_mmps(value: f64) -> Speed{
        Speed{
            value
        }
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
    pub fn from_mm(value: f64) -> Length{
        Length{
            value
        }
    }

    pub fn to_mm(self) -> f64{
        self.value
    }
}

// TODO add the z axis handling
pub async fn move_to<X: CaptureCompare16bitInstance, Y: CaptureCompare16bitInstance>(dst: Position3D, speed: Speed, x_stepper: &mut Stepper<'_, '_, X>, y_stepper: &mut Stepper<'_, '_, Y>){
    let src = Position3D::new(x_stepper.get_position(), y_stepper.get_position(), Position1D::from_mm(0.0));
    let x_delta = Length::from_mm(dst.x.to_mm() - src.x.to_mm());
    let y_delta = Length::from_mm(dst.y.to_mm() - src.y.to_mm());
    let th = (y_delta.to_mm() as f32).atan2(x_delta.to_mm() as f32);
    info!("angle: {}", th);

    let x_speed = Speed::from_mmps(speed.to_mmps() * th.cos() as f64);
    info!("x speed: {}", x_speed.to_mmps());
    x_stepper.set_speed(x_speed);

    let y_speed = Speed::from_mmps(speed.to_mmps() * th.sin() as f64);
    info!("y speed: {}", y_speed.to_mmps());
    y_stepper.set_speed(y_speed);

    join!(x_stepper.move_to(dst.get_x()), y_stepper.move_to(dst.get_y()));
}