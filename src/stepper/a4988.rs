#![allow(dead_code)]

use core::f64::consts::PI;

use embassy_stm32::gpio::{Output, AnyPin};
use embassy_stm32::pac::sai::vals::Freq;
use embassy_stm32::pwm::{CaptureCompare16bitInstance, Channel};
use embassy_stm32::pwm::simple_pwm::SimplePwm;
use embassy_stm32::time::hz;
use embassy_time::{Timer, Duration};
use micromath::F32Ext;
use crate::stepper::units::{Length, Position, Speed};

use defmt::*;
use {defmt_rtt as _, panic_probe as _};

pub enum StepperDirection{
    Clockwise,
    CounterClockwise
}

pub struct Stepper<'s, 'd, S>{
    step: SimplePwm<'s, S>,
    dir: Output<'d, AnyPin>,
    steps_per_revolution: u64,
    distance_per_step: Length,
    position: Position,
    direction: StepperDirection,
    step_duration: Duration,
}

impl <'s, 'd, S> Stepper <'s, 'd, S>
where S: CaptureCompare16bitInstance,
{
    pub fn new(step: SimplePwm<'s, S>, dir: Output<'d, AnyPin>, steps_per_revolution: u64, radius: Length) -> Stepper<'s, 'd, S>{
        let distance_per_step = dps_from_radius(radius, steps_per_revolution);
        Stepper{
            step,
            dir,
            steps_per_revolution,
            distance_per_step,
            position: Position::from_mm(0.0),
            direction: StepperDirection::Clockwise,
            step_duration: compute_step_duration(steps_per_revolution, distance_per_step, Speed::from_mmps(0.0).unwrap())
        }
    }

    /*
    update the speed an dcompute the frequency in which the pwm must run.
    pwm frequency: count of PWM interval periods per second
    PWM period: duration of one complete cycle or the total amount of active and inactive time combined
    */ 
    pub fn set_speed(&mut self, speed: Speed) -> (){
        self.step_duration = compute_step_duration(self.steps_per_revolution, self.distance_per_step, speed);
        let freq = hz(((1.0 / self.step_duration.as_micros() as f64) * 1_000_000.0) as u32);
        self.step.set_freq(freq);
    }

    pub fn set_direction(&mut self, direction: StepperDirection) -> (){
        self.direction = direction;
        match self.direction {
            StepperDirection::Clockwise => self.dir.set_high(),
            StepperDirection::CounterClockwise => self.dir.set_low()
        };
    }

    // the stepping is implemented through a pwm, where the duty is 50% (in order to have high/low pin for the same amount of time),
    // and the frequency is computed using the time for a step to be executed (step duration)
    pub async fn move_for(&mut self, distance: Length){
        if distance.to_mm() < self.distance_per_step.to_mm() {
            return;
        }
        // compute the number of steps we need to perform
        let steps_n = (distance.to_mm() / self.distance_per_step.to_mm()) as u64;

        // compute the duration of the move.
        let duration = Duration::from_micros(steps_n * self.step_duration.as_micros());
        
        // move
        self.step.enable(Channel::Ch1);
        Timer::after(duration).await;
        self.step.disable(Channel::Ch1);
        
        // update the position of the stepper
        let distance = match self.direction{
            StepperDirection::Clockwise => distance.to_mm(),
            StepperDirection::CounterClockwise => -distance.to_mm()
        };
        self.position = Position::from_mm(self.position.to_mm() + distance);
    }

    pub async fn move_to(&mut self, dst: Position){
        let delta = dst.to_mm() - self.position.to_mm();
        let direction = if delta.is_sign_negative() {StepperDirection::CounterClockwise} else {StepperDirection::Clockwise};
        self.set_direction(direction);
        let distance = Length::from_mm((delta as f32).abs() as f64);
        self.move_for(distance.unwrap()).await;
    }

    pub fn get_position(&self) -> Position{
        self.position
    }

    

}

// get distance per step from pulley's radius
// used for X/Y axis
fn dps_from_radius(r: Length, steps_per_revolution: u64) -> Length {
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
fn compute_step_duration(spr: u64, dps: Length, speed: Speed) -> Duration {
    // distance per revolution
    let distance_per_revolution = Length::from_mm(spr as f64 * dps.to_mm()).unwrap();
    let revolution_per_second = speed.to_mmps() / distance_per_revolution.to_mm();
    let second_per_revolution = 1.0 / revolution_per_second;
    let second_per_step = second_per_revolution / (spr as f64);
    let usecond_per_step = (second_per_step * 1_000_000.0) as u64;
    // we have to take into account also the time the stepper in inactive, so multiply the us per 2
    Duration::from_micros(usecond_per_step * 2)
}