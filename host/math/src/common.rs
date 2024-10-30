use core::{f64::consts::PI, time::Duration};
use heapless::Vec;
use micromath::F32Ext;
use measurements::{Distance, AngularVelocity};

use crate::{
    angle::{asin, cos, sin, Angle}, vector::Vector2D,
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

pub fn max<T: PartialEq + PartialOrd>(other: &[T]) -> Option<&T> {
    let mut max = other.first()?;
    for e in other {
        if *e > *max {
            max = e;
        }
    }
    Some(max)
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
    if r.as_millimeters() == 0f64 || steps_per_revolution == 0 {
        return None;
    }
    let p = 2.0 * r.as_millimeters() * PI;
    Some(Distance::from_millimeters(p / (steps_per_revolution as f64)))
}

// get distance per step from bar's pitch
// used for Z axis
pub fn dps_from_pitch(pitch: Distance, steps_per_revolution: u64) -> Option<Distance> {
    if pitch.as_millimeters() == 0f64 || steps_per_revolution == 0 {
        return None;
    }
    Some(Distance::from_millimeters(
        pitch.as_millimeters() / (steps_per_revolution as f64),
    ))
}

// compute the step duration, known as the delay between two successive steps
// the step duration is comprehensive of the HIGH period and the LOW period to perform a single step
// spr -> step per revolution
// dps -> distance per step
// speed -> mm/s
pub fn compute_step_duration(
    rpm: AngularVelocity,
    steps_per_revolution: u64,
) -> Result<Duration, ()> {
    if steps_per_revolution == 0 {
        return Err(());
    }
    let rpm = rpm.as_rpm().max(0f64);
    if rpm == 0.0 {
        return Ok(Duration::ZERO);
    }
    let second_per_revolution = 1.0 / (rpm / 60.0);
    let second_per_step = second_per_revolution / (steps_per_revolution as f64);
    Ok(Duration::from_secs_f64(second_per_step))
}

pub fn compute_rpm(step_duration: Duration, steps_per_revolution: u64) -> AngularVelocity {
    let second_per_step = step_duration.as_secs_f64();
    let second_per_revolution = second_per_step * steps_per_revolution as f64;
    if second_per_revolution == 0.0 {
        return AngularVelocity::from_rpm(0.0);
    }
    AngularVelocity::from_rpm((1.0 / second_per_revolution) * 60.0)
}

pub fn from_revolutions_per_second(
    value: f64,
    steps_per_revolution: u64,
    distance_per_step: Distance,
) -> Self {
    let distance_per_revolution =
        Distance::from_mm(steps_per_revolution as f64 * distance_per_step.to_mm());
    Self {
        value: distance_per_revolution.to_mm() * value,
    }
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
    let radius = (end - center).get_magnitude();
    if radius.as_millimeters() == 0f64 {
        return Distance::from_millimeters(0.0);
    }

    let chord_length = (end - start).get_magnitude();
    let mut th: f64 = 2.0 * asin(chord_length.as_millimeters() / (2.0 * radius.as_millimeters())).as_radians();

    if start_angle.as_radians() < end_angle.as_radians()
        && direction == RotationDirection::Clockwise
        || start_angle.as_radians() > end_angle.as_radians()
            && direction == RotationDirection::CounterClockwise
    {
        th = 2.0 * PI - th;
    }

    if th == 0f64 && full_circle_enabled {
        th = 2.0 * PI;
    }

    Distance::from_millimeters(radius.as_millimeters() * th)
}

pub fn compute_arc_destination(
    start: Vector2D<Distance>,
    center: Vector2D<Distance>,
    arc_length: Distance,
    direction: RotationDirection,
) -> Vector2D<Distance> {
    let delta = start - center;
    let radius = delta.get_magnitude();

    if radius.as_millimeters() == 0.0 || arc_length.as_millimeters() == 0.0 {
        return start;
    }

    let l = match direction {
        RotationDirection::Clockwise => Distance::from_millimeters(-arc_length.as_millimeters()),
        RotationDirection::CounterClockwise => arc_length,
    };

    let angle = Angle::from_radians(l.as_millimeters() / radius.as_millimeters());

    let x = center.get_x().as_millimeters() + (delta.get_x().as_millimeters() * cos(angle))
        - (delta.get_y().as_millimeters() * sin(angle));
    let y = center.get_y().as_millimeters()
        + (delta.get_x().as_millimeters() * sin(angle))
        + (delta.get_y().as_millimeters() * cos(angle));
    let x = Distance::from_millimeters(x);
    let y = Distance::from_millimeters(y);
    Vector2D::new(x, y)
}

pub fn approximate_arc(
    source: Vector2D<Distance>,
    center: Vector2D<Distance>,
    arc_length: Distance,
    direction: RotationDirection,
    unit_length: Distance,
) -> Vec<Vector2D<Distance>, 1024> {
    let mut points: Vec<Vector2D<Distance>, 1024> = Vec::new();
    let arcs_n = ((arc_length/ unit_length) as f32).floor() as u64;
    for i in 0..(arcs_n + 1) {
        let arc_length_curr = Distance::from_millimeters(unit_length.as_millimeters() * i as f64);
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
            compute_arc_length, compute_rpm, compute_step_duration,
            RotationDirection,
        },
        vector::Vector2D,
    };
    use measurements::{Distance, AngularVelocity};

    use super::{approximate_arc, compute_arc_destination};

    #[test]
    fn test_rps_from_millimetersps_1() {
        let steps_per_revolution = 100_u64;
        let distance_per_step = Distance::from_millimeters(1.0);
        let speed =
            AngularVelocity::from_revolutions_per_second(1.0, steps_per_revolution, distance_per_step);
        assert_eq!(speed.as_millimeters_per_second(), 100.0);
    }

    #[test]
    fn test_rps_from_millimetersps_2() {
        let steps_per_revolution = 200_u64;
        let distance_per_step = Distance::from_millimeters(1.0);
        let speed =
            AngularVelocity::from_revolutions_per_second(1.0, steps_per_revolution, distance_per_step);
        assert_eq!(speed.as_millimeters_per_second(), 200.0);
    }

    #[test]
    fn test_rps_from_millimetersps_3() {
        let steps_per_revolution = 200_u64;
        let distance_per_step = Distance::from_millimeters(0.1);
        let speed =
            AngularVelocity::from_revolutions_per_second(100.0, steps_per_revolution, distance_per_step);
        assert_eq!(speed.as_millimeters_per_second(), 2000.0);
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
        let start = Vector2D::new(Distance::from_millimeters(0.0), Distance::from_millimeters(0.0));
        let center = Vector2D::new(Distance::from_millimeters(1.0), Distance::from_millimeters(0.0));
        let arc_length = Distance::from_millimeters(PI / 2.0);
        let direction = RotationDirection::Clockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_float_absolute_eq!(dest.get_x().as_millimeters(), 1.0, 0.000001);
        assert_float_absolute_eq!(dest.get_y().as_millimeters(), 1.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_clockwise_2() {
        let start = Vector2D::new(Distance::from_millimeters(0.0), Distance::from_millimeters(0.0));
        let center = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(0.0));
        let direction = RotationDirection::Clockwise;
        let arc_length = Distance::from_millimeters(PI / 2.0);
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_float_absolute_eq!(dest.get_x().as_millimeters(), -1.0, 0.000001);
        assert_float_absolute_eq!(dest.get_y().as_millimeters(), -1.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_clockwise_3() {
        let start = Vector2D::new(Distance::from_millimeters(0.0), Distance::from_millimeters(0.0));
        let center = Vector2D::new(Distance::from_millimeters(2.0), Distance::from_millimeters(2.0));
        let arc_length = Distance::from_millimeters(PI / 2.0);
        let direction = RotationDirection::Clockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_float_absolute_eq!(dest.get_x().as_millimeters(), -0.7539200, 0.000001);
        assert_float_absolute_eq!(dest.get_y().as_millimeters(), 1.35507822, 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_counterclockwise_4() {
        let start = Vector2D::new(Distance::from_millimeters(2.0), Distance::from_millimeters(-6.0));
        let center = Vector2D::new(Distance::from_millimeters(3.0), Distance::from_millimeters(-2.0));
        let arc_length = Distance::from_millimeters(PI / 2.0);
        let direction = RotationDirection::CounterClockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_float_absolute_eq!(dest.get_x().as_millimeters(), 3.5589966, 0.000001);
        assert_float_absolute_eq!(dest.get_y().as_millimeters(), -6.0850364, 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_counterclockwise_1() {
        let start = Vector2D::new(Distance::from_millimeters(0.0), Distance::from_millimeters(0.0));
        let center = Vector2D::new(Distance::from_millimeters(1.0), Distance::from_millimeters(0.0));
        let arc_length = Distance::from_millimeters(PI / 2.0);
        let direction = RotationDirection::CounterClockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_float_absolute_eq!(dest.get_x().as_millimeters(), 1.0, 0.000001);
        assert_float_absolute_eq!(dest.get_y().as_millimeters(), -1.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_counterclockwise_2() {
        let start = Vector2D::new(Distance::from_millimeters(0.0), Distance::from_millimeters(0.0));
        let center = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(0.0));
        let direction = RotationDirection::CounterClockwise;
        let arc_length = Distance::from_millimeters(PI / 2.0);
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_float_absolute_eq!(dest.get_x().as_millimeters(), -1.0, 0.000001);
        assert_float_absolute_eq!(dest.get_y().as_millimeters(), 1.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_length_counterclockwise_1() {
        let start = Vector2D::new(Distance::from_millimeters(0.0), Distance::from_millimeters(0.0));
        let center = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(0.0));
        let end = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(-1.0));
        let l = compute_arc_length(
            start,
            center,
            end,
            RotationDirection::CounterClockwise,
            false,
        );
        assert_float_absolute_eq!(l.as_millimeters(), PI * (3.0 / 2.0), 0.000001);
    }

    #[test]
    fn test_compute_arc_length_clockwise_1() {
        let start = Vector2D::new(Distance::from_millimeters(0.0), Distance::from_millimeters(0.0));
        let center = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(0.0));
        let end = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(-1.0));
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise, false);
        assert_float_absolute_eq!(l.as_millimeters(), PI * (1.0 / 2.0), 0.000001);
    }

    #[test]
    fn test_compute_arc_length_counterclockwise_2() {
        let start = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(-1.0));
        let center = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(0.0));
        let end = Vector2D::new(Distance::from_millimeters(0.0), Distance::from_millimeters(0.0));
        let l = compute_arc_length(
            start,
            center,
            end,
            RotationDirection::CounterClockwise,
            false,
        );
        assert_float_absolute_eq!(l.as_millimeters(), PI * (1.0 / 2.0), 0.000001);
    }

    #[test]
    fn test_compute_arc_length_clockwise_2() {
        let start = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(-1.0));
        let center = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(0.0));
        let end = Vector2D::new(Distance::from_millimeters(0.0), Distance::from_millimeters(0.0));
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise, false);
        assert_float_absolute_eq!(l.as_millimeters(), PI * (3.0 / 2.0), 0.000001);
    }

    #[test]
    fn test_compute_arc_length_full_circle_off() {
        let start = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(-1.0));
        let center = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(0.0));
        let end = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(-1.0));
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise, false);
        assert_float_absolute_eq!(l.as_millimeters(), 0.0, 0.000001);
    }

    #[test]
    fn test_compute_arc_length_full_circle_on() {
        let start = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(-1.0));
        let center = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(0.0));
        let end = Vector2D::new(Distance::from_millimeters(-1.0), Distance::from_millimeters(-1.0));
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise, true);
        assert_float_absolute_eq!(l.as_millimeters(), 2.0 * PI, 0.000001);
    }

    #[test]
    fn test_approximate_arc() {
        let arc_length = Distance::from_millimeters(20.0);
        let start = Vector2D::new(Distance::from_millimeters(0.0), Distance::from_millimeters(0.0));
        let center = Vector2D::new(Distance::from_millimeters(10.0), Distance::from_millimeters(10.0));
        let direction = RotationDirection::Clockwise;
        let unit_length = Distance::from_millimeters(1.0);
        let points = approximate_arc(start, center, arc_length, direction, unit_length);
        assert_eq!(points.len(), 21);
        assert_float_absolute_eq!(points.first().unwrap().get_x().as_millimeters(), 0.0, 0.00001);
        assert_float_absolute_eq!(points.first().unwrap().get_y().as_millimeters(), 0.0, 0.00001);
        assert_float_absolute_eq!(points.get(1).unwrap().get_x().as_millimeters(), -0.681527, 0.00001);
        assert_float_absolute_eq!(points.get(1).unwrap().get_y().as_millimeters(), 0.731507, 0.00001);
        assert_float_absolute_eq!(points.get(10).unwrap().get_x().as_millimeters(), -4.098815, 0.00001);
        assert_float_absolute_eq!(points.get(10).unwrap().get_y().as_millimeters(), 8.893923, 0.00001);
        assert_float_absolute_eq!(points.get(20).unwrap().get_x().as_millimeters(), -1.437096, 0.00001);
        assert_float_absolute_eq!(points.get(20).unwrap().get_y().as_millimeters(), 18.318222, 0.00001);
    }

    #[test]
    fn test_approximate_arc_2() {
        let start = Vector2D::new(Distance::from_millimeters(0.0), Distance::from_millimeters(0.0));
        let end = Vector2D::new(Distance::from_millimeters(20.0), Distance::from_millimeters(20.0));
        let center = Vector2D::new(Distance::from_millimeters(10.0), Distance::from_millimeters(10.0));
        let direction = RotationDirection::Clockwise;
        let arc_length = compute_arc_length(start, center, end, direction, false);
        assert_float_absolute_eq!(arc_length.as_millimeters(), 44.428828, 0.00001);
        let unit_length = Distance::from_millimeters(1.0);
        let points = approximate_arc(start, center, arc_length, direction, unit_length);
        assert_eq!(points.len(), 45);
        assert_float_absolute_eq!(points.first().unwrap().get_x().as_millimeters(), 0.0, 0.00001);
        assert_float_absolute_eq!(points.first().unwrap().get_y().as_millimeters(), 0.0, 0.00001);
        assert_float_absolute_eq!(points.get(44).unwrap().get_x().as_millimeters(), 19.692222, 0.00001);
        assert_float_absolute_eq!(points.get(44).unwrap().get_y().as_millimeters(), 20.298583, 0.00001);
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
