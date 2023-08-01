#[no_std]

use core::f64::consts::PI;

use embassy_stm32::gpio::{Output, AnyPin};
use embassy_stm32::pwm::{CaptureCompare16bitInstance, Channel};
use embassy_stm32::pwm::simple_pwm::SimplePwm;
use embassy_stm32::time::hz;
use embassy_time::{Timer, Duration};
use micromath::F32Ext;
use crate::stepper::motion::{Length, Position1D, Speed};

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
    speed: Speed,
    // mm
    position: Position1D,
    direction: StepperDirection
}

impl <'s, 'd, S> Stepper <'s, 'd, S>
where S: CaptureCompare16bitInstance,
{
    pub fn new(step: SimplePwm<'s, S>, dir: Output<'d, AnyPin>, speed: Speed, steps_per_revolution: u64, distance_per_step: Length) -> Stepper<'s, 'd, S>{
        Stepper{
            step,
            dir,
            steps_per_revolution,
            speed,
            distance_per_step,
            position: Position1D::from_mm(0.0),
            direction: StepperDirection::Clockwise
        }
    }

    pub fn set_speed(&mut self, speed: Speed) -> (){
        self.speed = speed;
    }

    pub fn set_direction(&mut self, direction: StepperDirection) -> (){
        self.direction = direction;
        match self.direction {
            StepperDirection::Clockwise => self.dir.set_high(),
            StepperDirection::CounterClockwise => self.dir.set_low()
        };
    }

    // the stepping is implemented through a pwm, where the duty is 50% (in order to have high/low pin for the same amount of time),
    // and the frequency is computed using the time for a step to be executed (step delay)
    pub async fn move_for(&mut self, distance: Length){
        let step_delay = self.compute_step_delay();
        let steps_n = (distance.to_mm() / self.distance_per_step.to_mm()) as u64;
        // for every step we need to wait step_delay at high then step_delay at low, so 2 step_delay per step
        let duration = Duration::from_micros(2 * step_delay.as_micros() * steps_n);
        info!("Duration: {}", duration.as_micros());
        self.step.enable(Channel::Ch1);
        let freq = hz(((1.0 / step_delay.as_micros() as f64) * 1_000_000.0) as u32);
        self.step.set_freq(freq);
        Timer::after(duration).await;
        self.step.disable(Channel::Ch1);
        let distance = match self.direction{
            StepperDirection::Clockwise => self.distance_per_step.to_mm(),
            StepperDirection::CounterClockwise => -self.distance_per_step.to_mm()
        };
        self.position = Position1D::from_mm(self.position.to_mm() + distance);
    }

    pub async fn move_to(&mut self, dst: Position1D){
        let delta = dst.to_mm() - self.position.to_mm();
        let direction = if delta.is_sign_negative() {StepperDirection::CounterClockwise} else {StepperDirection::Clockwise};
        self.set_direction(direction);
        let distance = Length::from_mm((delta as f32).abs() as f64);
        self.move_for(distance).await;
    }

    pub fn get_position(&self) -> Position1D{
        self.position
    }

    // TODO try to simplify this algorithm
    fn compute_step_delay(&self) -> Duration {
        let round_length = Length::from_mm(self.steps_per_revolution as f64 * self.distance_per_step.to_mm());
        let round_per_second = self.speed.to_mmps() / round_length.to_mm();
        let second_per_round = 1.0 / round_per_second;
        let second_per_step = second_per_round / (self.steps_per_revolution as f64);
        let microsps = (second_per_step * 1_000_000.0) as u64;
        Duration::from_micros(microsps)
    }

}

// get distance per step from pulley's radius
// used for X/Y axis
pub fn dps_from_radius(r: Length, steps_per_revolution: u64) -> Length {
    let p = 2.0 * r.to_mm() * PI;
    Length::from_mm(p / (steps_per_revolution as f64))
}

// get distance per step from bar's pitch
// used for Z axis
pub fn dps_from_pitch(pitch: Length, steps_per_revolution: u64) -> Length {
    Length::from_mm(pitch.to_mm() / (steps_per_revolution as f64))
}