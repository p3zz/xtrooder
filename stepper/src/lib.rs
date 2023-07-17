#![no_std]

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;
use embedded_time::duration::*;
use nb::block;

pub enum StepperDirection{
    Clockwise,
    CounterClockwise
}

pub struct Stepper<O, T>{
    step: O,
    dir: O,
    steps_per_revolution: u32,
    timer: T,
    step_delay: Microseconds,
    // mm
    distance_per_step: f32,
    // mm
    position: f32,
    direction: StepperDirection
}

impl <O, T> Stepper<O, T>
where O: OutputPin, T: CountDown<Time = Microseconds>,
{
    pub fn new(step: O, dir: O, steps_per_revolution: u32, timer: T, distance_per_step: f32) -> Stepper<O,T>{
        Stepper{
            step,
            dir,
            steps_per_revolution,
            timer,
            step_delay: sps_from_rpm(1, steps_per_revolution),
            distance_per_step,
            position: 0.0,
            direction: StepperDirection::Clockwise
        }
    }

    pub fn set_speed(&mut self, speed: u32) -> (){
        self.step_delay = sps_from_rpm(speed, self.steps_per_revolution);
    }

    pub fn set_direction(&mut self, direction: StepperDirection) -> (){
        self.direction = direction;
        let _  = match self.direction {
            StepperDirection::Clockwise => self.dir.set_high(),
            StepperDirection::CounterClockwise => self.dir.set_low()
        };
    }

    pub fn step(&mut self) -> (){
        let _ = self.step.set_high();
        self.timer.start(self.step_delay);
        block!(self.timer.wait()).unwrap();
        let _ = self.step.set_low();
        self.position += match self.direction{
            StepperDirection::Clockwise => self.distance_per_step,
            StepperDirection::CounterClockwise => -self.distance_per_step,
        };
    }

    pub fn move_for(&mut self, distance: f32) -> (){
        let steps = (distance / self.distance_per_step) as u32;
        for _ in 0..steps{
            self.step();
        }
    }
}

// get second per step from round per minute
fn sps_from_rpm(rpm: u32, steps_per_revolution: u32) -> Microseconds<u32> {
    let rps = (rpm / 60) as f32;
    let spr = 1.0 / rps;
    let sps = spr/(steps_per_revolution as f32);
    let microsps = (sps * 1_000_000.0) as u32;
    return Microseconds(microsps);
}

// get distance per step from pulley's radius
// used for X/Y axis
pub fn dps_from_radius(r: f32, steps_per_revolution: u32) -> f32 {
    let p = 2.0 * r * 3.14159;
    return p / (steps_per_revolution as f32);
}

// get distance per step from bar's pitch
// used for Z axis
pub fn dps_from_pitch(pitch: f32, steps_per_revolution: u32) -> f32 {
    return pitch / (steps_per_revolution as f32);
}