#![no_std]

use embassy_stm32::pwm::{CaptureCompare16bitInstance, Channel};
use embassy_stm32::pwm::simple_pwm::SimplePwm;
use embassy_stm32::time::hz;
use embassy_time::{Timer, Duration};

pub struct Length{
    // mm
    value: f32
}

impl Length{
    pub fn from_mm(value: f32) -> Length{
        Length{
            value
        }
    }

    pub fn to_mm(&self) -> f32{
        return self.value;
    }
}

pub enum StepperDirection{
    Clockwise,
    CounterClockwise
}

pub struct Stepper<'d, T>{
    step: SimplePwm<'d, T>,
    // dir: D,
    // steps_per_revolution: u32,
    // timer: T,
    // step_delay: MicroSeconds,
    // mm
    // distance_per_step: Length,
    // mm
    // position: Length,
    // direction: StepperDirection
}

impl <'d, T> Stepper<'d, T>
where T: CaptureCompare16bitInstance,
{
    pub fn new(step: SimplePwm<'d, T>) -> Stepper<T>{
        Stepper{
            step,
            // dir,
            // steps_per_revolution,
            // timer,
            // step_delay: sps_from_rpm(1, steps_per_revolution),
            // distance_per_step,
            // position: Length::from_mm(0.0),
            // direction: StepperDirection::Clockwise
        }
    }

    // pub fn set_speed(&mut self, speed: u32) -> (){
    //     self.step_delay = sps_from_rpm(speed, self.steps_per_revolution);
    // }

    // pub fn set_direction(&mut self, direction: StepperDirection) -> (){
    //     self.direction = direction;
    //     let _  = match self.direction {
    //         StepperDirection::Clockwise => self.dir.set_high(),
    //         StepperDirection::CounterClockwise => self.dir.set_low()
    //     };
    // }

    pub async fn step(&mut self) -> (){
        self.step.enable(Channel::Ch1);
        self.step.set_freq(hz(10));
        Timer::after(Duration::from_millis(500)).await;
        self.step.disable(Channel::Ch1);
        // let _ = self.step.set_high();
        // self.timer.start(self.step_delay);
        // block!(self.timer.wait()).unwrap();
        // let _ = self.step.set_low();
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
// fn sps_from_rpm(rpm: u32, steps_per_revolution: u32) -> MicroSeconds {
//     let rps = (rpm / 60) as f32;
//     let spr = 1.0 / rps;
//     let sps = spr/(steps_per_revolution as f32);
//     let microsps = (sps * 1_000_000.0) as u32;
//     return MicroSeconds(microsps);
// }

// get distance per step from pulley's radius
// used for X/Y axis
pub fn dps_from_radius(r: Length, steps_per_revolution: u32) -> Length {
    let p = 2.0 * r.to_mm() * 3.14159;
    return Length::from_mm(p / (steps_per_revolution as f32));
}

// get distance per step from bar's pitch
// used for Z axis
pub fn dps_from_pitch(pitch: Length, steps_per_revolution: u32) -> Length {
    return Length::from_mm(pitch.to_mm() / (steps_per_revolution as f32));
}