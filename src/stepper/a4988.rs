#![no_std]

use core::f64::consts::PI;

use embassy_stm32::gpio::{Output, AnyPin};
use embassy_stm32::pwm::{CaptureCompare16bitInstance, Channel};
use embassy_stm32::pwm::simple_pwm::SimplePwm;
use embassy_stm32::time::hz;
use embassy_time::{Timer, Duration};

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

    pub fn to_rps(self) -> u64{
        self.value
    }

    pub fn to_mmps(self, radius: Length) -> f64{
        let perimeter = 2.0 * PI * radius.to_mm();
        self.value as f64 * perimeter
    }
}

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

    pub fn to_mm(self) -> f64{
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
    // position: Length,
    // direction: StepperDirection
}

impl <'s, 'd, S> Stepper <'s, 'd, S>
where S: CaptureCompare16bitInstance,
{
    pub fn new(step: SimplePwm<'s, S>, dir: Output<'d, AnyPin>, steps_per_revolution: u64, distance_per_step: Length) -> Stepper<'s, 'd, S>{
        Stepper{
            step,
            dir,
            steps_per_revolution,
            step_delay: sps_from_rpm(1, steps_per_revolution),
            distance_per_step,
            // position: Length::from_mm(0.0),
            // direction: StepperDirection::Clockwise
        }
    }

    pub fn set_speed(&mut self, speed: u64) -> (){
        self.step_delay = sps_from_rpm(speed, self.steps_per_revolution);
    }

    pub fn set_direction(&mut self, direction: StepperDirection) -> (){
        // self.direction = direction;
        let _  = match direction {
            StepperDirection::Clockwise => self.dir.set_high(),
            StepperDirection::CounterClockwise => self.dir.set_low()
        };
    }

    pub async fn step(&mut self) -> (){
        self.step.enable(Channel::Ch1);
        self.step.set_freq(hz(10));
        Timer::after(Duration::from_millis(500)).await;
        self.step.disable(Channel::Ch1);
        // let distance = match self.direction{
            // StepperDirection::Clockwise => self.distance_per_step.to_mm(),
            // StepperDirection::CounterClockwise => -self.distance_per_step.to_mm()
        // };
        // self.position = Length::from_mm(self.position.to_mm() + distance);
    }

    // pub fn move_for(&mut self, distance: Length) -> (){
    //     let steps = (distance.to_mm() / self.distance_per_step.to_mm()) as u32;
    //     for _ in 0..steps{
    //         self.step();
    //     }
    // }
}

// get second per step from round per minute
// TODO check types
fn sps_from_rpm(rpm: u64, steps_per_revolution: u64) -> Duration {
    let rps = rpm as f64 / 60.0;
    let spr = 1.0 / rps;
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