use defmt::{assert, assert_eq, println};

use crate::math::{vector::Vector, common::abs};
use super::math::compute_step_duration;

fn test_compute_step_duration_1(){
    println!("Test - Compute step duration 1");
    let step_per_revolution = 200;
    let distance_per_step = Vector::from_mm(1.0);
    let speed = Vector::from_mm(1.0);
    let duration = compute_step_duration(step_per_revolution, distance_per_step, speed);
    assert!(duration.is_some());
    assert!(abs(1_000_000.0 - duration.unwrap().as_micros() as f64) < 50.0);
}

fn test_compute_step_duration_2(){
    println!("Test - Compute step duration 2");
    let step_per_revolution = 100;
    let distance_per_step = Vector::from_mm(0.1);
    let speed = Vector::from_mm(10.0);
    let duration = compute_step_duration(step_per_revolution, distance_per_step, speed);
    assert!(duration.is_some());
    println!("{}", duration.unwrap().as_micros());
    assert!(abs(10_000.0 - duration.unwrap().as_micros() as f64) < 50.0);
}

fn test_compute_step_duration_3(){
    println!("Test - Compute step duration 3");
    let step_per_revolution = 100;
    let distance_per_step = Vector::from_mm(0.1);
    let speed = Vector::from_mm(0.0);
    let duration = compute_step_duration(step_per_revolution, distance_per_step, speed);
    assert!(duration.is_none());
}

fn test_compute_step_duration_4(){
    println!("Test - Compute step duration 4");
    let step_per_revolution = 100;
    let distance_per_step = Vector::from_mm(0.0);
    let speed = Vector::from_mm(1.0);
    let duration = compute_step_duration(step_per_revolution, distance_per_step, speed);
    assert!(duration.is_none());
}

pub fn test(){
    test_compute_step_duration_1();
    test_compute_step_duration_2();
    test_compute_step_duration_3();
    test_compute_step_duration_4();
}