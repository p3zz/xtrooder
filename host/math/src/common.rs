use core::{f64::consts::PI, time::Duration};
use micromath::F32Ext;

use crate::{distance::Distance, speed::Speed};

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

#[cfg(test)]
mod tests {
    use crate::{
        common::{abs, compute_step_duration},
        distance::Distance,
        speed::Speed,
    };

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
