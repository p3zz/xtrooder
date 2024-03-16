use defmt::{assert, assert_eq, println};

use crate::math::{common::abs, speed::Speed, distance::Distance};
use super::math::{compute_step_duration};

fn test_rps_from_mmps_1(){
    println!("Test - RPS from MMPS 1");
    let steps_per_revolution = 100_u64;
    let distance_per_step = Distance::from_mm(1.0);
    let speed = Speed::from_revolutions_per_second(1.0, steps_per_revolution, distance_per_step);
    assert!(speed.to_mm_per_second() == 0.01);
}

fn test_rps_from_mmps_2(){
    println!("Test - RPS from MMPS 2");
    let steps_per_revolution = 200_u64;
    let distance_per_step = Distance::from_mm(1.0);
    let speed = Speed::from_revolutions_per_second(1.0, steps_per_revolution, distance_per_step);
    assert!(speed.to_mm_per_second() == 0.005);
}

fn test_rps_from_mmps_3(){
    println!("Test - RPS from MMPS 3");
    let steps_per_revolution = 200_u64;
    let distance_per_step = Distance::from_mm(0.1);
    let speed = Speed::from_revolutions_per_second(100.0, steps_per_revolution, distance_per_step);
    assert_eq!(speed.to_mm_per_second(), 5.0);
}

fn test_compute_step_duration_1(){
    println!("Test - Compute step duration 1");
    let steps_per_revolution = 200_u64;
    let distance_per_step = Distance::from_mm(1.0);
    let speed = Speed::from_revolutions_per_second(1.0, steps_per_revolution, distance_per_step);
    let duration = compute_step_duration(steps_per_revolution, distance_per_step, speed);
    assert!(duration.is_some());
    assert!(abs(1_000_000.0 - duration.unwrap().as_micros() as f64) < 50.0);
}

fn test_compute_step_duration_2(){
    println!("Test - Compute step duration 2");
    let steps_per_revolution = 100_u64;
    let distance_per_step = Distance::from_mm(0.1);
    let speed = Speed::from_revolutions_per_second(10.0, steps_per_revolution, distance_per_step);
    let duration = compute_step_duration(steps_per_revolution, distance_per_step, speed);
    assert!(duration.is_some());
    println!("{}", duration.unwrap().as_micros());
    assert!(abs(10_000.0 - duration.unwrap().as_micros() as f64) < 50.0);
}

fn test_compute_step_duration_3(){
    println!("Test - Compute step duration 3");
    let steps_per_revolution = 100_u64;
    let distance_per_step = Distance::from_mm(0.1);
    let speed = Speed::from_revolutions_per_second(0.0, steps_per_revolution, distance_per_step);
    let duration = compute_step_duration(steps_per_revolution, distance_per_step, speed);
    assert!(duration.is_none());
}

fn test_compute_step_duration_4(){
    println!("Test - Compute step duration 4");
    let steps_per_revolution = 100_u64;
    let distance_per_step = Distance::from_mm(0.0);
    let speed = Speed::from_revolutions_per_second(1.0, steps_per_revolution, distance_per_step);
    let duration = compute_step_duration(steps_per_revolution, distance_per_step, speed);
    assert!(duration.is_none());
}

pub fn test(){
    test_rps_from_mmps_1();
    test_rps_from_mmps_2();
    test_rps_from_mmps_3();
    test_compute_step_duration_1();
    test_compute_step_duration_2();
    test_compute_step_duration_3();
    test_compute_step_duration_4();
}