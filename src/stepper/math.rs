use embassy_time::Duration;

use super::units::{Length, Speed};
use core::f64::consts::PI;

// get distance per step from pulley's radius
// used for X/Y axis
pub fn dps_from_radius(r: Length, steps_per_revolution: u64) -> Length {
    let p = 2.0 * r.to_mm() * PI;
    Length::from_mm(p / (steps_per_revolution as f64)).unwrap()
}

// get distance per step from bar's pitch
// used for Z axis
pub fn dps_from_pitch(pitch: Length, steps_per_revolution: u64) -> Length {
    Length::from_mm(pitch.to_mm() / (steps_per_revolution as f64)).unwrap()
}

// compute the step duration, known as the time taken to perform a single step (active + inactive time)
// spr -> step per revolution
// dps -> distance per step
pub fn compute_step_duration(spr: u64, dps: Length, speed: Speed) -> Duration {
    // distance per revolution
    let distance_per_revolution = Length::from_mm(spr as f64 * dps.to_mm()).unwrap();
    let revolution_per_second = speed.to_mmps() / distance_per_revolution.to_mm();
    let second_per_revolution = 1.0 / revolution_per_second;
    let second_per_step = second_per_revolution / (spr as f64);
    let usecond_per_step = (second_per_step * 1_000_000.0) as u64;
    // we have to take into account also the time the stepper in inactive, so multiply the us per 2
    Duration::from_micros(usecond_per_step * 2)
}
