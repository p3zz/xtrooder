use core::{f64::consts::PI, time::Duration};
use micromath::F32Ext;

use crate::{angle::{asin, atan2, cos, sin, Angle}, computable::Computable, distance::Distance, speed::Speed, vector::Vector2D};

#[derive(Clone, Copy, PartialEq)]
pub enum RotationDirection {
    Clockwise,
    CounterClockwise,
}

impl From<RotationDirection> for u8 {
    fn from(value: RotationDirection) -> Self {
        match value {
            RotationDirection::Clockwise => 0,
            RotationDirection::CounterClockwise => 1,
        }
    }
}

pub fn abs(value: f64) -> f64 {
    let mut v = value;
    if value.is_sign_negative() {
        v = -value;
    }
    v
}

pub fn sqrt(value: f64) -> f64 {
    (value as f32).sqrt() as f64
}

// get distance per step from pulley's radius
// used for X/Y axis
pub fn dps_from_radius(r: Distance, steps_per_revolution: u64) -> Option<Distance> {
    if r.to_mm() == 0f64 || steps_per_revolution == 0 {
        return None;
    }
    let p = 2.0 * r.to_mm() * PI;
    Some(Distance::from_mm(p / (steps_per_revolution as f64)))
}

// get distance per step from bar's pitch
// used for Z axis
pub fn dps_from_pitch(pitch: Distance, steps_per_revolution: u64) -> Option<Distance> {
    if pitch.to_mm() == 0f64 || steps_per_revolution == 0 {
        return None;
    }
    Some(Distance::from_mm(
        pitch.to_mm() / (steps_per_revolution as f64),
    ))
}

// compute the step duration, known as the delay between two successive steps
// the step duration is comprehensive of the HIGH period and the LOW period to perform a single step
// spr -> step per revolution
// dps -> distance per step
// speed -> mm/s
pub fn compute_step_duration(spr: u64, dps: Distance, speed: Speed) -> Option<Duration> {
    let rps = speed.to_revolutions_per_second(spr, dps);
    if rps == 0f64 {
        return None;
    }
    let second_per_revolution = 1.0 / rps;
    let second_per_step = second_per_revolution / (spr as f64);
    let usecond_per_step = (second_per_step * 1_000_000.0) as u64;
    Some(Duration::from_micros(usecond_per_step / 2))
}

pub fn compute_arc_length(start: Vector2D<Distance>, center: Vector2D<Distance>, end: Vector2D<Distance>, direction: RotationDirection) -> Option<Distance>{
    // FIXME check if radius=0
    let start_angle = start.get_angle();
    let end_angle = end.get_angle();
    let radius = end.sub(&center).get_magnitude();
    let chord_length = end.sub(&start).get_magnitude();
    let mut th: f64 = 2.0 * asin(chord_length.to_mm() / (2.0 * radius.to_mm())).to_radians();

    if start_angle.to_radians() < end_angle.to_radians() && direction == RotationDirection::Clockwise ||
        start_angle.to_radians() > end_angle.to_radians() && direction == RotationDirection::CounterClockwise {
        th = 2.0 * PI - th
    }

    Some(Distance::from_mm(radius.to_mm() * th))
}

pub fn compute_arc_destination(start: Vector2D<Distance>, center: Vector2D<Distance>, arc_length: Distance, direction: RotationDirection) -> Option<Vector2D<Distance>> {
    let delta = start.sub(&center);
    let radius = delta.get_magnitude();
    let angle = delta.get_angle();

    let l = match direction{
        RotationDirection::Clockwise => Distance::from_mm(-arc_length.to_mm()),
        RotationDirection::CounterClockwise => arc_length,
    };

    // FIXME check if radius=0
    let angle = Angle::from_radians((angle.to_radians() + l.to_mm()) / radius.to_mm());

    let x = Distance::from_mm(center.get_x().to_mm() + radius.to_mm() * cos(angle));
    let y = Distance::from_mm(center.get_y().to_mm() + radius.to_mm() * sin(angle));
    Some(Vector2D::new(x, y))
}

#[cfg(test)]
mod tests {
    use core::f64::consts::PI;
    use assert_float_eq::*;

    use crate::{
        common::{abs, compute_arc_length, compute_step_duration, RotationDirection},
        distance::Distance,
        speed::Speed, vector::Vector2D,
    };

    use super::compute_arc_destination;

    #[test]
    fn test_rps_from_mmps_1() {
        println!("Test - RPS from MMPS 1");
        let steps_per_revolution = 100_u64;
        let distance_per_step = Distance::from_mm(1.0);
        let speed =
            Speed::from_revolutions_per_second(1.0, steps_per_revolution, distance_per_step);
        assert_eq!(speed.to_mm_per_second(), 100.0);
    }

    #[test]
    fn test_rps_from_mmps_2() {
        println!("Test - RPS from MMPS 2");
        let steps_per_revolution = 200_u64;
        let distance_per_step = Distance::from_mm(1.0);
        let speed =
            Speed::from_revolutions_per_second(1.0, steps_per_revolution, distance_per_step);
        assert_eq!(speed.to_mm_per_second(), 200.0);
    }

    #[test]
    fn test_rps_from_mmps_3() {
        println!("Test - RPS from MMPS 3");
        let steps_per_revolution = 200_u64;
        let distance_per_step = Distance::from_mm(0.1);
        let speed =
            Speed::from_revolutions_per_second(100.0, steps_per_revolution, distance_per_step);
        assert_eq!(speed.to_mm_per_second(), 2000.0);
    }

    #[test]
    fn test_compute_step_duration_1() {
        println!("Test - Compute step duration 1");
        let steps_per_revolution = 200_u64;
        let distance_per_step = Distance::from_mm(1.0);
        let speed =
            Speed::from_revolutions_per_second(1.0, steps_per_revolution, distance_per_step);
        let duration = compute_step_duration(steps_per_revolution, distance_per_step, speed);
        assert!(duration.is_some());
        assert_eq!(duration.unwrap().as_micros(), 2500);
    }

    #[test]
    fn test_compute_step_duration_2() {
        println!("Test - Compute step duration 2");
        let steps_per_revolution = 100_u64;
        let distance_per_step = Distance::from_mm(0.1);
        let speed =
            Speed::from_revolutions_per_second(10.0, steps_per_revolution, distance_per_step);
        let duration = compute_step_duration(steps_per_revolution, distance_per_step, speed);
        assert!(duration.is_some());
        assert_eq!(duration.unwrap().as_micros(), 500);
    }

    #[test]
    fn test_compute_step_duration_3() {
        println!("Test - Compute step duration 3");
        let steps_per_revolution = 100_u64;
        let distance_per_step = Distance::from_mm(0.1);
        let speed =
            Speed::from_revolutions_per_second(0.0, steps_per_revolution, distance_per_step);
        let duration = compute_step_duration(steps_per_revolution, distance_per_step, speed);
        assert!(duration.is_none());
    }

    #[test]
    fn test_compute_step_duration_4() {
        println!("Test - Compute step duration 4");
        let steps_per_revolution = 100_u64;
        let distance_per_step = Distance::from_mm(0.0);
        let speed =
            Speed::from_revolutions_per_second(1.0, steps_per_revolution, distance_per_step);
        let duration = compute_step_duration(steps_per_revolution, distance_per_step, speed);
        assert!(duration.is_none());
    }

    #[test]
    fn test_compute_arc_destination_clockwise_1(){
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(1.0), Distance::from_mm(0.0));
        let arc_length = Distance::from_mm(PI/2.0);
        let direction = RotationDirection::Clockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert!(dest.is_some());
        assert_float_absolute_eq!(dest.unwrap().get_x().to_mm(), 1.0, 0.000001);
        assert_float_absolute_eq!(dest.unwrap().get_y().to_mm(), 1.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_clockwise_2(){
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let direction = RotationDirection::Clockwise;
        let arc_length = Distance::from_mm(PI/2.0);
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert!(dest.is_some());
        assert_float_absolute_eq!(dest.unwrap().get_x().to_mm(), -1.0, 0.000001);
        assert_float_absolute_eq!(dest.unwrap().get_y().to_mm(), -1.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_counterclockwise_1(){
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(1.0), Distance::from_mm(0.0));
        let arc_length = Distance::from_mm(PI/2.0);
        let direction = RotationDirection::CounterClockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert!(dest.is_some());
        assert_float_absolute_eq!(dest.unwrap().get_x().to_mm(), 1.0, 0.000001);
        assert_float_absolute_eq!(dest.unwrap().get_y().to_mm(), -1.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_counterclockwise_2(){
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let direction = RotationDirection::CounterClockwise;
        let arc_length = Distance::from_mm(PI/2.0);
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert!(dest.is_some());
        assert_float_absolute_eq!(dest.unwrap().get_x().to_mm(), -1.0, 0.000001);
        assert_float_absolute_eq!(dest.unwrap().get_y().to_mm(), 1.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_length_counterclockwise_1(){
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let end = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(-1.0));
        let l = compute_arc_length(start, center, end, RotationDirection::CounterClockwise);
        assert!(l.is_some());
        assert_float_absolute_eq!(l.unwrap().to_mm(), PI * (3.0/2.0), 0.000001);
    }


    #[test]
    fn test_compute_arc_length_clockwise_1(){
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let end = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(-1.0));
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise);
        assert!(l.is_some());
        assert_float_absolute_eq!(l.unwrap().to_mm(), PI * (1.0/2.0), 0.000001);
    }

    #[test]
    fn test_compute_arc_length_counterclockwise_2(){
        let start = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(-1.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let end = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let l = compute_arc_length(start, center, end, RotationDirection::CounterClockwise);
        assert!(l.is_some());
        assert_float_absolute_eq!(l.unwrap().to_mm(), PI * (1.0/2.0), 0.000001);
    }


    #[test]
    fn test_compute_arc_length_clockwise_2(){
        let start = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(-1.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let end = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise);
        assert!(l.is_some());
        assert_float_absolute_eq!(l.unwrap().to_mm(), PI * (3.0/2.0), 0.000001);
    }

}

// pub struct StopWatch {
//     last_ticks: u64,
// }

// impl StopWatch {
//     pub fn new() -> StopWatch {
//         StopWatch { last_ticks: 0 }
//     }

//     pub fn start(&mut self) {
//         self.last_ticks = Instant::now().as_ticks();
//     }

//     pub fn measure(&self) -> Duration {
//         let current_ticks = Instant::now().as_ticks();
//         Duration::from_ticks(current_ticks - self.last_ticks)
//     }
// }
