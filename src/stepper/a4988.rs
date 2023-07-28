#![no_std]

use core::f64::consts::PI;

use embassy_stm32::gpio::{Output, AnyPin};
use embassy_stm32::pwm::{CaptureCompare16bitInstance, Channel};
use embassy_stm32::pwm::simple_pwm::SimplePwm;
use embassy_stm32::time::mhz;
use embassy_time::{Timer, Duration};

pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}

impl Position{
    pub fn new(x: f64, y: f64, z: f64) -> Position{
        Position { x, y, z }
    }
}
pub struct Speed {
    // rps
    value: u64
}

impl Speed {
    // round per second
    pub fn from_rps(rps: u64) -> Speed{
        Speed{
            value: rps
        }
    }

    // mm per second
    pub fn from_mmps(mmps: f64, radius: Length) -> Speed{
        let perimeter = 2.0 * PI * radius.to_mm();
        Speed{
            value: (mmps/perimeter) as u64
        }
    }

    pub fn to_rps(&self) -> u64{
        self.value
    }

    pub fn to_mmps(&self, radius: Length) -> f64{
        let perimeter = 2.0 * PI * radius.to_mm();
        self.value as f64 * perimeter
    }
}

#[derive(Clone, Copy)]
pub struct Length{
    // mm
    value: f64
}

impl Length{
    pub fn from_mm(value: f64) -> Length{
        Length{
            value
        }
    }

    pub fn to_mm(&self) -> f64{
        return self.value;
    }
}

pub enum StepperDirection{
    Clockwise,
    CounterClockwise
}

pub struct Stepper<'s, 'd, S>{
    step: SimplePwm<'s, S>,
    dir: Output<'d, AnyPin>,
    steps_per_revolution: u64,
    step_delay: Duration,
    // mm
    distance_per_step: Length,
    // mm
    position: Length,
    direction: StepperDirection
}

impl <'s, 'd, S> Stepper <'s, 'd, S>
where S: CaptureCompare16bitInstance,
{
    pub fn new(step: SimplePwm<'s, S>, dir: Output<'d, AnyPin>, steps_per_revolution: u64, distance_per_step: Length) -> Stepper<'s, 'd, S>{
        Stepper{
            step,
            dir,
            steps_per_revolution,
            step_delay: compute_step_delay(Speed::from_rps(1), steps_per_revolution),
            distance_per_step,
            position: Length::from_mm(0.0),
            direction: StepperDirection::Clockwise
        }
    }

    pub fn set_speed(&mut self, speed: Speed) -> (){
        self.step_delay = compute_step_delay(speed, self.steps_per_revolution);
    }

    pub fn set_direction(&mut self, direction: StepperDirection) -> (){
        self.direction = direction;
        let _  = match self.direction {
            StepperDirection::Clockwise => self.dir.set_high(),
            StepperDirection::CounterClockwise => self.dir.set_low()
        };
    }

    // FIXME need a way to update the direction based on the sign of the distance
    pub async fn move_for(&mut self, distance: Length){
        let steps_n = (distance.to_mm() / self.distance_per_step.to_mm()) as u64;
        // for every step we need to wait step_delay at high then step_delay at low, so 2 step_delay per step
        let duration = Duration::from_micros(2 * self.step_delay.as_micros() * steps_n);
        self.step.enable(Channel::Ch1);
        // FIXME check the frequence, or something horrible will happen 
        self.step.set_freq(mhz(1 / (self.step_delay.as_micros() as u32)));
        Timer::after(duration).await;
        self.step.disable(Channel::Ch1);
        let distance = match self.direction{
            StepperDirection::Clockwise => self.distance_per_step.to_mm(),
            StepperDirection::CounterClockwise => -self.distance_per_step.to_mm()
        };
        self.position = Length::from_mm(self.position.to_mm() + distance);
    }

    // FIXME make distance always positive
    pub async fn move_to(&mut self, dst: Length){
        let delta = dst.to_mm() - self.position.to_mm();
        self.direction = if delta.is_sign_negative() {StepperDirection::CounterClockwise} else {StepperDirection::Clockwise}; 
        let distance = Length::from_mm(delta);
        self.move_for(distance).await;
    }

    pub fn get_position(&self) -> Length{
        self.position
    }

}

// get second per step from round per minute
// TODO check types
fn compute_step_delay(speed: Speed, steps_per_revolution: u64) -> Duration {
    let spr = 1.0 / speed.to_rps() as f64;
    let sps = spr/(steps_per_revolution as f64);
    let microsps = (sps * 1_000_000.0) as u64;
    return Duration::from_micros(microsps);
}

// get distance per step from pulley's radius
// used for X/Y axis
pub fn dps_from_radius(r: Length, steps_per_revolution: u64) -> Length {
    let p = 2.0 * r.to_mm() * PI;
    return Length::from_mm(p / (steps_per_revolution as f64));
}

// get distance per step from bar's pitch
// used for Z axis
pub fn dps_from_pitch(pitch: Length, steps_per_revolution: u64) -> Length {
    return Length::from_mm(pitch.to_mm() / (steps_per_revolution as f64));
}