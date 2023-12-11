use core::f64::consts::PI;

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

impl Computable<Angle> for Angle{
    fn add(&self, other: Angle) -> Angle {
        Angle::from_radians(self.to_radians() + other.to_radians())
    }

    fn sub(&self, other: Angle) -> Angle {
        Angle::from_radians(self.to_radians() - other.to_radians())
    }
}
