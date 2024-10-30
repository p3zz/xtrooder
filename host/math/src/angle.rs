pub use measurements::Angle;
use micromath::F32Ext;

/*
contains some wrapping functions that enables the use of micromath
with f64.
As explained in the rust book:
- Casting from an f32 to an f64 is perfect and lossless
- Casting from an f64 to an f32 will produce the closest possible f32 **
if necessary, rounding is according to roundTiesToEven mode ***
on overflow, infinity (of the same sign as the input) is produced
** if f64-to-f32 casts with this rounding mode and overflow behavior are
not supported natively by the hardware, these casts will likely be slower than expected.
*** as defined in IEEE 754-2008 §4.3.1: pick the nearest floating point
number, preferring the one with an even least significant digit if exactly
halfway between two floating point numbers.
*/
pub fn cos(angle: Angle) -> f64 {
    (angle.as_radians() as f32).cos() as f64
}

pub fn sin(angle: Angle) -> f64 {
    (angle.as_radians() as f32).sin() as f64
}

pub fn atan2(y: f64, x: f64) -> Angle {
    let mut th = (y as f32).atan2(x as f32) as f64;
    if th.is_nan() {
        th = 0.0;
    }
    Angle::from_radians(th)
}

// TODO check nan
pub fn acos(value: f64) -> Angle {
    let th = (value as f32).acos() as f64;
    Angle::from_radians(th)
}

// TODO check nan
pub fn asin(value: f64) -> Angle {
    let th = (value as f32).asin() as f64;
    Angle::from_radians(th)
}
