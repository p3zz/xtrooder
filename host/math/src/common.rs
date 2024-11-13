use core::{f64::consts::PI, time::Duration};
use measurements::{AngularVelocity, Distance, Speed};
use micromath::F32Ext;

use crate::{
    angle::{asin, cos, sin, Angle},
    vector::Vector2D,
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

impl From<&str> for RotationDirection {
    fn from(value: &str) -> Self {
        match value {
            "clockwise" => RotationDirection::Clockwise,
            "counterclockwise" => RotationDirection::CounterClockwise,
            _ => panic!("Invalid rotation direction"),
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
    Some(Distance::from_millimeters(
        p / (steps_per_revolution as f64),
    ))
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
pub fn compute_step_duration(rpm: AngularVelocity, steps_per_revolution: u64) -> Duration {
    if steps_per_revolution == 0 {
        return Duration::ZERO;
    }
    let rpm = rpm.as_rpm().max(0f64);
    if rpm == 0.0 {
        return Duration::ZERO;
    }
    let second_per_revolution = 1.0 / (rpm / 60.0);
    let second_per_step = second_per_revolution / (steps_per_revolution as f64);
    Duration::from_secs_f64(second_per_step)
}

pub fn angular_velocity_from_speed(
    speed: Speed,
    steps_per_revolution: u64,
    distance_per_step: Distance,
) -> AngularVelocity {
    let distance_per_revolution = steps_per_revolution as f64 * distance_per_step;
    if distance_per_revolution.as_millimeters() == 0f64 {
        return AngularVelocity::from_rpm(0.0);
    }
    AngularVelocity::from_rpm(
        speed.as_meters_per_second() / distance_per_revolution.as_meters() * 60.0,
    )
}

pub fn angular_velocity_from_steps(
    step_duration: Duration,
    steps_per_revolution: u64,
) -> AngularVelocity {
    let second_per_step = step_duration.as_secs_f64();
    let second_per_revolution = second_per_step * steps_per_revolution as f64;
    if second_per_revolution == 0.0 {
        return AngularVelocity::from_rpm(0.0);
    }
    AngularVelocity::from_rpm((1.0 / second_per_revolution) * 60.0)
}

pub fn speed_from_angular_velocity(
    angular_velocity: AngularVelocity,
    steps_per_revolution: u64,
    distance_per_step: Distance,
) -> Speed {
    let distance_per_revolution = steps_per_revolution as f64 * distance_per_step;
    Speed::from_meters_per_second(
        distance_per_revolution.as_meters() * angular_velocity.as_rpm() / 60.0,
    )
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
    let mut th: f64 =
        2.0 * asin(chord_length.as_millimeters() / (2.0 * radius.as_millimeters())).as_radians();

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

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use core::{f64::consts::PI, time::Duration};

    use crate::{
        common::{
            angular_velocity_from_steps, compute_arc_length, compute_step_duration,
            speed_from_angular_velocity, RotationDirection,
        },
        vector::Vector2D,
    };
    use measurements::{AngularVelocity, Distance, Speed};

    use super::{angular_velocity_from_speed, compute_arc_destination};

    #[test]
    fn test_speed_from_angular_velocity() {
        let steps_per_revolution = 100_u64;
        let distance_per_step = Distance::from_millimeters(1.0);
        let angular_velocity = AngularVelocity::from_rpm(60.0);
        let speed =
            speed_from_angular_velocity(angular_velocity, steps_per_revolution, distance_per_step);
        assert_abs_diff_eq!(speed.as_meters_per_second(), 0.1, epsilon = 0.000001);
    }

    #[test]
    fn test_speed_from_angular_velocity_2() {
        let steps_per_revolution = 200_u64;
        let distance_per_step = Distance::from_millimeters(1.0);
        let angular_velocity = AngularVelocity::from_rpm(60.0);
        let speed =
            speed_from_angular_velocity(angular_velocity, steps_per_revolution, distance_per_step);
        assert_abs_diff_eq!(speed.as_meters_per_second(), 0.2, epsilon = 0.000001);
    }

    #[test]
    fn test_speed_from_angular_velocity_3() {
        let steps_per_revolution = 200_u64;
        let distance_per_step = Distance::from_millimeters(0.1);
        let angular_velocity = AngularVelocity::from_rpm(6000.0);
        let speed =
            speed_from_angular_velocity(angular_velocity, steps_per_revolution, distance_per_step);
        assert_abs_diff_eq!(speed.as_meters_per_second(), 2.0, epsilon = 0.000001);
    }

    #[test]
    fn test_angular_velocity_from_speed() {
        let steps_per_revolution = 100_u64;
        let distance_per_step = Distance::from_millimeters(1.0);
        let speed = Speed::from_meters_per_second(0.1);
        let angular_velocity =
            angular_velocity_from_speed(speed, steps_per_revolution, distance_per_step);
        assert_abs_diff_eq!(angular_velocity.as_rpm(), 60.0, epsilon = 0.000001);
    }

    #[test]
    fn test_angular_velocity_from_speed_2() {
        let steps_per_revolution = 200_u64;
        let distance_per_step = Distance::from_millimeters(1.0);
        let speed = Speed::from_meters_per_second(0.2);
        let angular_velocity =
            angular_velocity_from_speed(speed, steps_per_revolution, distance_per_step);
        assert_abs_diff_eq!(angular_velocity.as_rpm(), 60.0, epsilon = 0.000001);
    }

    #[test]
    fn test_angular_velocity_from_speed_3() {
        let steps_per_revolution = 200_u64;
        let distance_per_step = Distance::from_millimeters(0.1);
        let speed = Speed::from_meters_per_second(2.0);
        let speed = angular_velocity_from_speed(speed, steps_per_revolution, distance_per_step);
        assert_abs_diff_eq!(speed.as_rpm(), 6000.0, epsilon = 0.000001);
    }

    #[test]
    fn test_compute_step_duration_valid() {
        let steps_per_revolution = 200_u64;
        let revolutions_per_second = AngularVelocity::from_rpm(60.0);
        let duration = compute_step_duration(revolutions_per_second, steps_per_revolution);
        assert_eq!(duration.as_micros(), 5000);
    }

    #[test]
    fn test_compute_step_duration_zero() {
        let steps_per_revolution = 200_u64;
        let revolutions_per_second = AngularVelocity::from_rpm(0.0);
        let duration = compute_step_duration(revolutions_per_second, steps_per_revolution);
        assert!(duration.is_zero());
    }

    #[test]
    fn test_compute_step_duration_negative() {
        let steps_per_revolution = 200_u64;
        let revolutions_per_second = AngularVelocity::from_rpm(-120.0);
        let duration = compute_step_duration(revolutions_per_second, steps_per_revolution);
        assert!(duration.is_zero());
    }

    #[test]
    fn test_angular_velocity_from_steps() {
        let steps_per_revolution = 200_u64;
        let step_duration = Duration::from_micros(5000);
        let angular_velocity = angular_velocity_from_steps(step_duration, steps_per_revolution);
        assert_abs_diff_eq!(angular_velocity.as_rpm(), 60.0, epsilon = 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_clockwise_1() {
        let start = Vector2D::new(
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
        );
        let center = Vector2D::new(
            Distance::from_millimeters(1.0),
            Distance::from_millimeters(0.0),
        );
        let arc_length = Distance::from_millimeters(PI / 2.0);
        let direction = RotationDirection::Clockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_abs_diff_eq!(dest.get_x().as_millimeters(), 1.0, epsilon = 0.000001);
        assert_abs_diff_eq!(dest.get_y().as_millimeters(), 1.0, epsilon = 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_clockwise_2() {
        let start = Vector2D::new(
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
        );
        let center = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(0.0),
        );
        let direction = RotationDirection::Clockwise;
        let arc_length = Distance::from_millimeters(PI / 2.0);
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_abs_diff_eq!(dest.get_x().as_millimeters(), -1.0, epsilon = 0.000001);
        assert_abs_diff_eq!(dest.get_y().as_millimeters(), -1.0, epsilon = 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_clockwise_3() {
        let start = Vector2D::new(
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
        );
        let center = Vector2D::new(
            Distance::from_millimeters(2.0),
            Distance::from_millimeters(2.0),
        );
        let arc_length = Distance::from_millimeters(PI / 2.0);
        let direction = RotationDirection::Clockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_abs_diff_eq!(
            dest.get_x().as_millimeters(),
            -0.7539200,
            epsilon = 0.000001
        );
        assert_abs_diff_eq!(
            dest.get_y().as_millimeters(),
            1.35507822,
            epsilon = 0.000001
        );
    }

    #[test]
    fn test_compute_arc_destination_counterclockwise_4() {
        let start = Vector2D::new(
            Distance::from_millimeters(2.0),
            Distance::from_millimeters(-6.0),
        );
        let center = Vector2D::new(
            Distance::from_millimeters(3.0),
            Distance::from_millimeters(-2.0),
        );
        let arc_length = Distance::from_millimeters(PI / 2.0);
        let direction = RotationDirection::CounterClockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_abs_diff_eq!(dest.get_x().as_millimeters(), 3.5589966, epsilon = 0.000001);
        assert_abs_diff_eq!(
            dest.get_y().as_millimeters(),
            -6.0850364,
            epsilon = 0.000001
        );
    }

    #[test]
    fn test_compute_arc_destination_counterclockwise_1() {
        let start = Vector2D::new(
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
        );
        let center = Vector2D::new(
            Distance::from_millimeters(1.0),
            Distance::from_millimeters(0.0),
        );
        let arc_length = Distance::from_millimeters(PI / 2.0);
        let direction = RotationDirection::CounterClockwise;
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_abs_diff_eq!(dest.get_x().as_millimeters(), 1.0, epsilon = 0.000001);
        assert_abs_diff_eq!(dest.get_y().as_millimeters(), -1.0, epsilon = 0.000001);
    }

    #[test]
    fn test_compute_arc_destination_counterclockwise_2() {
        let start = Vector2D::new(
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
        );
        let center = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(0.0),
        );
        let direction = RotationDirection::CounterClockwise;
        let arc_length = Distance::from_millimeters(PI / 2.0);
        let dest = compute_arc_destination(start, center, arc_length, direction);
        assert_abs_diff_eq!(dest.get_x().as_millimeters(), -1.0, epsilon = 0.000001);
        assert_abs_diff_eq!(dest.get_y().as_millimeters(), 1.0, epsilon = 0.000001);
    }

    #[test]
    fn test_compute_arc_length_counterclockwise_1() {
        let start = Vector2D::new(
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
        );
        let center = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(0.0),
        );
        let end = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(-1.0),
        );
        let l = compute_arc_length(
            start,
            center,
            end,
            RotationDirection::CounterClockwise,
            false,
        );
        assert_abs_diff_eq!(l.as_millimeters(), PI * (3.0 / 2.0), epsilon = 0.000001);
    }

    #[test]
    fn test_compute_arc_length_clockwise_1() {
        let start = Vector2D::new(
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
        );
        let center = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(0.0),
        );
        let end = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(-1.0),
        );
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise, false);
        assert_abs_diff_eq!(l.as_millimeters(), PI * (1.0 / 2.0), epsilon = 0.000001);
    }

    #[test]
    fn test_compute_arc_length_counterclockwise_2() {
        let start = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(-1.0),
        );
        let center = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(0.0),
        );
        let end = Vector2D::new(
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
        );
        let l = compute_arc_length(
            start,
            center,
            end,
            RotationDirection::CounterClockwise,
            false,
        );
        assert_abs_diff_eq!(l.as_millimeters(), PI * (1.0 / 2.0), epsilon = 0.000001);
    }

    #[test]
    fn test_compute_arc_length_clockwise_2() {
        let start = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(-1.0),
        );
        let center = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(0.0),
        );
        let end = Vector2D::new(
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
        );
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise, false);
        assert_abs_diff_eq!(l.as_millimeters(), PI * (3.0 / 2.0), epsilon = 0.000001);
    }

    #[test]
    fn test_compute_arc_length_full_circle_off() {
        let start = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(-1.0),
        );
        let center = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(0.0),
        );
        let end = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(-1.0),
        );
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise, false);
        assert_abs_diff_eq!(l.as_millimeters(), 0.0, epsilon = 0.000001);
    }

    #[test]
    fn test_compute_arc_length_full_circle_on() {
        let start = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(-1.0),
        );
        let center = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(0.0),
        );
        let end = Vector2D::new(
            Distance::from_millimeters(-1.0),
            Distance::from_millimeters(-1.0),
        );
        let l = compute_arc_length(start, center, end, RotationDirection::Clockwise, true);
        assert_abs_diff_eq!(l.as_millimeters(), 2.0 * PI, epsilon = 0.000001);
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
