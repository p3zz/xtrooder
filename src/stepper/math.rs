use embassy_time::Duration;

use crate::math::vector::Vector;
use core::f64::consts::PI;

// get distance per step from pulley's radius
// used for X/Y axis
pub fn dps_from_radius(r: Vector, steps_per_revolution: u64) -> Option<Vector> {
    if r.to_mm() == 0f64 || steps_per_revolution == 0 {
        return None;
    }
    let p = 2.0 * r.to_mm() * PI;
    Some(Vector::from_mm(p / (steps_per_revolution as f64)))
}

// get distance per step from bar's pitch
// used for Z axis
pub fn dps_from_pitch(pitch: Vector, steps_per_revolution: u64) -> Option<Vector> {
    if pitch.to_mm() == 0f64 || steps_per_revolution == 0{
        return None;
    }
    Some(Vector::from_mm(pitch.to_mm() / (steps_per_revolution as f64)))
}

// compute the step duration, known as the delay between two successive steps
// the step duration is comprehensive of the HIGH period and the LOW period to perform a single step
// spr -> step per revolution
// dps -> distance per step
// speed -> mm/s
pub fn compute_step_duration(spr: u64, dps: Vector, speed: Vector) -> Option<Duration> {
    // distance per revolution
    if spr == 0 || dps.to_mm() == 0f64 || speed.to_mm() == 0f64 {
        return None;
    }
    let distance_per_revolution = Vector::from_mm(spr as f64 * dps.to_mm());
    let revolution_per_second = speed.to_mm() / distance_per_revolution.to_mm();
    let second_per_revolution = 1.0 / revolution_per_second;
    let second_per_step = second_per_revolution / (spr as f64);
    let usecond_per_step = (second_per_step * 1_000_000.0) as u64;
    Some(Duration::from_micros(usecond_per_step))
}
