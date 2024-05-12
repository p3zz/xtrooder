use core::{f64::consts::PI, time::Duration};
use micromath::F32Ext;
use heapless::Vec;

use crate::{
    angle::{asin, cos, sin, Angle}, computable::Computable, distance::Distance, speed::Speed, vector::Vector2D
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RotationDirection {
    Clockwise,
    CounterClockwise,
}

impl From<RotationDirection> for i8 {
    fn from(value: RotationDirection) -> Self {
        match value {
            RotationDirection::Clockwise => 1,
            RotationDirection::CounterClockwise => -1,
        }
    }
}

pub fn max(other: &[u64]) -> Option<u64> {
    let mut max = other.get(0)?;
    for e in other {
        if e > max {
            max = e;
        }
    }
    Some(*max)
}

pub fn abs(value: f64) -> f64 {
    let mut v = value;
    if value.is_sign_negative() {
        v = -value;
    }
    v
}

pub fn floor(value: f64) -> f64 {
    (value as f32).floor() as f64
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
pub fn compute_step_duration(
    revolutions_per_second: f64,
    steps_per_revolution: u64,
) -> Result<Duration, ()> {
    if revolutions_per_second.is_sign_negative() || steps_per_revolution == 0 {
        return Err(());
    }
    if revolutions_per_second == 0.0 {
        return Ok(Duration::ZERO);
    }
    let second_per_revolution = 1.0 / revolutions_per_second;
    let second_per_step = second_per_revolution / (steps_per_revolution as f64);
    Ok(Duration::from_secs_f64(second_per_step))
}

pub fn compute_revolutions_per_second(step_duration: Duration, steps_per_revolution: u64) -> f64 {
    let second_per_step = step_duration.as_secs_f64();
    let second_per_revolution = second_per_step * steps_per_revolution as f64;
    if second_per_revolution == 0.0 {
        return 0.0;
    }
    1.0 / second_per_revolution
}

pub fn compute_arc_length(
    start: Vector2D<Distance>,
    center: Vector2D<Distance>,
    end: Vector2D<Distance>,
    direction: RotationDirection,
    full_circle_enabled: bool,
) -> Distance {
    let start_angle = start.get_angle();
    let end_angle = end.get_angle();
    let radius = end.sub(&center).get_magnitude();
    if radius.to_mm() == 0f64 {
        return Distance::from_mm(0.0);
    }

    let chord_length = end.sub(&start).get_magnitude();
    let mut th: f64 = 2.0 * asin(chord_length.to_mm() / (2.0 * radius.to_mm())).to_radians();

    if start_angle.to_radians() < end_angle.to_radians()
        && direction == RotationDirection::Clockwise
        || start_angle.to_radians() > end_angle.to_radians()
            && direction == RotationDirection::CounterClockwise
    {
        th = 2.0 * PI - th;
    }

    if th == 0f64 && full_circle_enabled {
        th = 2.0 * PI;
    }

    Distance::from_mm(radius.to_mm() * th)
}

pub fn compute_arc_destination(
    start: Vector2D<Distance>,
    center: Vector2D<Distance>,
    arc_length: Distance,
    direction: RotationDirection,
) -> Vector2D<Distance> {
    let delta = start.sub(&center);
    let radius = delta.get_magnitude();

    if radius.to_mm() == 0.0 || arc_length.to_mm() == 0.0 {
        return start;
    }

    let l = match direction {
        RotationDirection::Clockwise => Distance::from_mm(-arc_length.to_mm()),
        RotationDirection::CounterClockwise => arc_length,
    };

    let angle = Angle::from_radians(l.to_mm() / radius.to_mm());
    
    let x = center.get_x().to_mm() + (delta.get_x().to_mm() * cos(angle)) - (delta.get_y().to_mm() * sin(angle));
    let y = center.get_y().to_mm() + (delta.get_x().to_mm() * sin(angle)) + (delta.get_y().to_mm() * cos(angle));
    let x = Distance::from_mm(x);
    let y = Distance::from_mm(y);
    Vector2D::new(x, y)
}

pub fn approximate_arc(
    source: Vector2D<Distance>,
    center: Vector2D<Distance>,
    arc_length: Distance,
    direction: RotationDirection,
    unit_length: Distance
) -> Vec<Vector2D<Distance>, 1024>{
    let mut points: Vec<Vector2D<Distance>, 1024> = Vec::new();
    let arcs_n = (arc_length.div(&unit_length).unwrap() as f32).floor() as u64;
    for i in 0..(arcs_n + 1) {
        let arc_length_curr = Distance::from_mm(unit_length.to_mm() * i as f64);
        let arc_dst = compute_arc_destination(source, center, arc_length_curr, direction);
        // FIXME
        points.push(arc_dst);
    }
    points
}

#[cfg(test)]
mod tests {
    use assert_float_eq::*;
    use core::{f64::consts::PI, time::Duration};

    use crate::{
        common::{
            abs, compute_arc_length, compute_revolutions_per_second, compute_step_duration,
            RotationDirection,
        },
        distance::Distance,
        speed::Speed,
        vector::Vector2D,
    };

    use super::{approximate_arc, compute_arc_destination};

    #[test]
    fn test_rps_from_mmps_1() {
        let steps_per_revolution = 100_u64;
        let distance_per_step = Distance::from_mm(1.0);
        let speed =
            Speed::from_revolutions_per_second(1.0, steps_per_revolution, distance_per_step);
        assert_eq!(speed.to_mm_per_second(), 100.0);
    }

    #[test]
    fn test_rps_from_mmps_2() {
        let steps_per_revolution = 200_u64;
        let distance_per_step = Distance::from_mm(1.0);
        let speed =
            Speed::from_revolutions_per_second(1.0, steps_per_revolution, distance_per_step);
        assert_eq!(speed.to_mm_per_second(), 200.0);
    }

    #[test]
    fn test_rps_from_mmps_3() {
        let steps_per_revolution = 200_u64;
        let distance_per_step = Distance::from_mm(0.1);
        let speed =
            Speed::from_revolutions_per_second(100.0, steps_per_revolution, distance_per_step);
        assert_eq!(speed.to_mm_per_second(), 2000.0);
    }

    #[test]
    fn test_compute_step_duration_valid() {
        let steps_per_revolution = 200_u64;
        let revolutions_per_second = 1.0;
        let duration = compute_step_duration(revolutions_per_second, steps_per_revolution);
        assert!(duration.is_ok());
        assert_eq!(duration.unwrap().as_micros(), 5000);
    }

    #[test]
    fn test_compute_step_duration_zero() {
        let steps_per_revolution = 200_u64;
        let revolutions_per_second = 0.0;
        let duration = compute_step_duration(revolutions_per_second, steps_per_revolution);
        assert!(duration.is_ok());
        assert!(duration.unwrap().is_zero());
    }

    #[test]
    fn test_compute_step_duration_negative() {
        let steps_per_revolution = 200_u64;
        let revolutions_per_second = -2.0;
        let duration = compute_step_duration(revolutions_per_second, steps_per_revolution);
        assert!(duration.is_err());
    }

    #[test]
    fn test_compute_revolutions_per_second() {
        let steps_per_revolution = 200_u64;
        let step_duration = Duration::from_micros(5000);
        let revolutions_per_second =
            compute_revolutions_per_second(step_duration, steps_per_revolution);
        assert_eq!(revolutions_per_second, 1.0);
    }

    #[test]
    fn test_compute_arc_destination_clockwise_1() {
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(1.0), Distance::from_mm(0.0));
        let arc_length = Distance::from_mm(PI / 2.0);
        let direction = RotationDirection::Clockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_float_absolute_eq!(dest.get_x().to_mm(), 1.0, 0.000001);
        assert_float_absolute_eq!(dest.get_y().to_mm(), 1.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_clockwise_2() {
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let direction = RotationDirection::Clockwise;
        let arc_length = Distance::from_mm(PI / 2.0);
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_float_absolute_eq!(dest.get_x().to_mm(), -1.0, 0.000001);
        assert_float_absolute_eq!(dest.get_y().to_mm(), -1.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_clockwise_3() {
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(2.0), Distance::from_mm(2.0));
        let arc_length = Distance::from_mm(PI / 2.0);
        let direction = RotationDirection::Clockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_float_absolute_eq!(dest.get_x().to_mm(), -0.7539200, 0.000001);
        assert_float_absolute_eq!(dest.get_y().to_mm(), 1.35507822, 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_counterclockwise_4() {
        let start = Vector2D::new(Distance::from_mm(2.0), Distance::from_mm(-6.0));
        let center = Vector2D::new(Distance::from_mm(3.0), Distance::from_mm(-2.0));
        let arc_length = Distance::from_mm(PI / 2.0);
        let direction = RotationDirection::CounterClockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_float_absolute_eq!(dest.get_x().to_mm(), 3.5589966, 0.000001);
        assert_float_absolute_eq!(dest.get_y().to_mm(), -6.0850364, 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_counterclockwise_1() {
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(1.0), Distance::from_mm(0.0));
        let arc_length = Distance::from_mm(PI / 2.0);
        let direction = RotationDirection::CounterClockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_float_absolute_eq!(dest.get_x().to_mm(), 1.0, 0.000001);
        assert_float_absolute_eq!(dest.get_y().to_mm(), -1.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_counterclockwise_2() {
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let direction = RotationDirection::CounterClockwise;
        let arc_length = Distance::from_mm(PI / 2.0);
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_float_absolute_eq!(dest.get_x().to_mm(), -1.0, 0.000001);
        assert_float_absolute_eq!(dest.get_y().to_mm(), 1.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_length_counterclockwise_1() {
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let end = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(-1.0));
        let l = compute_arc_length(
            start,
            center,
            end,
            RotationDirection::CounterClockwise,
            false,
        );
        assert_float_absolute_eq!(l.to_mm(), PI * (3.0 / 2.0), 0.000001);
    }

    #[test]
    fn test_compute_arc_length_clockwise_1() {
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let end = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(-1.0));
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise, false);
        assert_float_absolute_eq!(l.to_mm(), PI * (1.0 / 2.0), 0.000001);
    }

    #[test]
    fn test_compute_arc_length_counterclockwise_2() {
        let start = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(-1.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let end = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let l = compute_arc_length(
            start,
            center,
            end,
            RotationDirection::CounterClockwise,
            false,
        );
        assert_float_absolute_eq!(l.to_mm(), PI * (1.0 / 2.0), 0.000001);
    }

    #[test]
    fn test_compute_arc_length_clockwise_2() {
        let start = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(-1.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let end = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise, false);
        assert_float_absolute_eq!(l.to_mm(), PI * (3.0 / 2.0), 0.000001);
    }

    #[test]
    fn test_compute_arc_length_full_circle_off() {
        let start = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(-1.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let end = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(-1.0));
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise, false);
        assert_float_absolute_eq!(l.to_mm(), 0.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_length_full_circle_on() {
        let start = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(-1.0));
        let center = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(0.0));
        let end = Vector2D::new(Distance::from_mm(-1.0), Distance::from_mm(-1.0));
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise, true);
        assert_float_absolute_eq!(l.to_mm(), 2.0 * PI, 0.000001);
    }

    #[test]
    fn test_approximate_arc(){
        let arc_length = Distance::from_mm(20.0);
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let center = Vector2D::new(Distance::from_mm(10.0), Distance::from_mm(10.0));
        let direction = RotationDirection::Clockwise;
        let unit_length = Distance::from_mm(1.0);
        let points = approximate_arc(start, center, arc_length, direction, unit_length);
        assert_eq!(points.len(), 21);
        assert_float_absolute_eq!(points.get(0).unwrap().get_x().to_mm(), 0.0, 0.00001);
        assert_float_absolute_eq!(points.get(0).unwrap().get_y().to_mm(), 0.0, 0.00001);
        assert_float_absolute_eq!(points.get(1).unwrap().get_x().to_mm(), -0.681527, 0.00001);
        assert_float_absolute_eq!(points.get(1).unwrap().get_y().to_mm(), 0.731507, 0.00001);
        assert_float_absolute_eq!(points.get(10).unwrap().get_x().to_mm(), -4.098815, 0.00001);
        assert_float_absolute_eq!(points.get(10).unwrap().get_y().to_mm(), 8.893923, 0.00001);
        assert_float_absolute_eq!(points.get(20).unwrap().get_x().to_mm(), -1.437096, 0.00001);
        assert_float_absolute_eq!(points.get(20).unwrap().get_y().to_mm(), 18.318222, 0.00001);
    }

    #[test]
    fn test_approximate_arc_2(){
        let start = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let end = Vector2D::new(Distance::from_mm(20.0), Distance::from_mm(20.0));
        let center = Vector2D::new(Distance::from_mm(10.0), Distance::from_mm(10.0));
        let direction = RotationDirection::Clockwise;
        let arc_length = compute_arc_length(start, center, end, direction, false);
        assert_float_absolute_eq!(arc_length.to_mm(), 44.428828, 0.00001);
        let unit_length = Distance::from_mm(1.0);
        let points = approximate_arc(start, center, arc_length, direction, unit_length);
        assert_eq!(points.len(), 45);
        assert_float_absolute_eq!(points.get(0).unwrap().get_x().to_mm(), 0.0, 0.00001);
        assert_float_absolute_eq!(points.get(0).unwrap().get_y().to_mm(), 0.0, 0.00001);
        assert_float_absolute_eq!(points.get(44).unwrap().get_x().to_mm(), 19.692222, 0.00001);
        assert_float_absolute_eq!(points.get(44).unwrap().get_y().to_mm(), 20.298583, 0.00001);
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
