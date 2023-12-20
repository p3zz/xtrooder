use core::f64::consts::PI;
use micromath::F32Ext;
use super::computable::Computable;

#[derive(Clone, Copy)]
pub struct Angle {
    // radians
    value: f64,
}

impl Angle {
    pub fn from_radians(value: f64) -> Angle {
        Angle { value }
    }

    pub fn from_degrees(angle: f64) -> Angle {
        let value = angle * PI / 180.0;
        Angle { value }
    }

    pub fn to_radians(&self) -> f64 {
        self.value
    }

    pub fn to_degrees(&self) -> f64 {
        self.value * 180.0 / PI
    }
}

impl Computable<Angle> for Angle {
    fn add(&self, other: Angle) -> Angle {
        Angle::from_radians(self.to_radians() + other.to_radians())
    }

    fn sub(&self, other: Angle) -> Angle {
        Angle::from_radians(self.to_radians() - other.to_radians())
    }
}

pub fn cos(angle: Angle) -> f64 {
    return (angle.to_radians() as f32).cos() as f64;
}

pub fn sin(angle: Angle) -> f64 {
    return (angle.to_radians() as f32).sin() as f64;
}

pub fn atan2(y: f64, x: f64) -> Angle {
    let th = (y as f32).atan2(x as f32) as f64;
    Angle::from_radians(th)
}

pub fn acos(value: f64) -> Angle {
    let th = (value as f32).acos() as f64;
    Angle::from_radians(th)
}
