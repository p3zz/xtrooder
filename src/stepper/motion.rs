#[no_std]

use embassy_stm32::pwm::CaptureCompare16bitInstance;
use micromath::F32Ext;
use futures::join;
use core::f64::consts::PI;
use crate::stepper::a4988::Stepper;

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
}
pub struct Speed {
    // rps
    value: u64
}

impl Speed {
    // round per second
    pub fn from_rps(rps: u64) -> Speed{
        Speed{
            value: rps
        }
    }

    // mm per second
    pub fn from_mmps(mmps: f64, radius: Length) -> Speed{
        let perimeter = 2.0 * PI * radius.to_mm();
        Speed{
            value: (mmps/perimeter) as u64
        }
    }

    pub fn to_rps(&self) -> u64{
        self.value
    }

    pub fn to_mmps(&self, radius: Length) -> f64{
        let perimeter = 2.0 * PI * radius.to_mm();
        self.value as f64 * perimeter
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


pub async fn move_to<X: CaptureCompare16bitInstance, Y: CaptureCompare16bitInstance>(dst: Position3D, speed: Speed, x_stepper: &mut Stepper<'_, '_, X>, y_stepper: &mut Stepper<'_, '_, Y>){
    let src = Position3D::new(x_stepper.get_position(), y_stepper.get_position(), Position1D::from_mm(0.0));
    let x_delta = Length::from_mm(dst.x.to_mm() - src.x.to_mm());
    let y_delta = Length::from_mm(dst.y.to_mm() - src.y.to_mm());
    let th = (x_delta.to_mm() as f32).atan2(y_delta.to_mm() as f32);

    let x_speed = Speed::from_rps((speed.to_rps() as f32 * th.cos()) as u64);
    x_stepper.set_speed(x_speed);

    let y_speed = Speed::from_rps((speed.to_rps() as f32 * th.sin()) as u64);
    y_stepper.set_speed(y_speed);

    join!(x_stepper.move_for(x_delta), y_stepper.move_for(y_delta));
}