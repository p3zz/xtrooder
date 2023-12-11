#![allow(dead_code)]

use crate::math::vector::Vector;
use embassy_stm32::gpio::{AnyPin, Output};
use embassy_stm32::pwm::simple_pwm::SimplePwm;
use embassy_stm32::pwm::{CaptureCompare16bitInstance, Channel};
use embassy_stm32::time::hz;
use embassy_time::{Duration, Timer};
use micromath::F32Ext;

use super::math::compute_step_duration;
use {defmt_rtt as _, panic_probe as _};

pub enum StepperDirection {
    Clockwise,
    CounterClockwise,
}

pub struct Stepper<'s, S> {
    // properties that won't change
    step: SimplePwm<'s, S>,
    step_ch: Channel,
    dir: Output<'s, AnyPin>,
    steps_per_revolution: u64,
    distance_per_step: Vector,

    // properties that have to be computed and kept updated during the execution
    position: Vector,
    direction: StepperDirection,
    step_duration: Duration,
}

impl<'s, S> Stepper<'s, S>
where
    S: CaptureCompare16bitInstance,
{
    pub fn new(
        mut step: SimplePwm<'s, S>,
        step_ch: Channel,
        dir: Output<'s, AnyPin>,
        steps_per_revolution: u64,
        distance_per_step: Vector,
    ) -> Stepper<'s, S> {
        step.set_duty(step_ch, step.get_max_duty() / 2);
        Stepper {
            step,
            step_ch,
            dir,
            steps_per_revolution,
            distance_per_step,
            position: Vector::from_mm(0.0),
            direction: StepperDirection::Clockwise,
            step_duration: compute_step_duration(
                steps_per_revolution,
                distance_per_step,
                Vector::from_mm(0.0)
            ),
        }
    }

    /*
    update the speed an dcompute the frequency in which the pwm must run.
    pwm frequency: count of PWM interval periods per second
    PWM period: duration of one complete cycle or the total amount of active and inactive time combined
    */
    pub fn set_speed(&mut self, speed: Vector) -> () {
        self.step_duration =
            compute_step_duration(self.steps_per_revolution, self.distance_per_step, speed);
        let freq = hz(((1.0 / self.step_duration.as_micros() as f64) * 1_000_000.0) as u32);
        self.step.set_freq(freq);
    }

    pub fn set_direction(&mut self, direction: StepperDirection) -> () {
        self.direction = direction;
        match self.direction {
            StepperDirection::Clockwise => self.dir.set_high(),
            StepperDirection::CounterClockwise => self.dir.set_low(),
        };
    }

    // the stepping is implemented through a pwm, where the duty is 50% (in order to have high/low pin for the same amount of time),
    // and the frequency is computed using the time for a step to be executed (step duration)
    async fn move_for(&mut self, distance: Vector) {
        if distance.to_mm() < self.distance_per_step.to_mm() {
            return;
        }
        // compute the number of steps we need to perform
        let steps_n = (distance.to_mm() / self.distance_per_step.to_mm()) as u64;

        // compute the duration of the move.
        let duration = Duration::from_micros(steps_n * self.step_duration.as_micros());

        // move
        self.step.enable(self.step_ch);
        Timer::after(duration).await;
        self.step.disable(self.step_ch);

        // update the position of the stepper
        let distance = match self.direction {
            StepperDirection::Clockwise => distance.to_mm(),
            StepperDirection::CounterClockwise => -distance.to_mm(),
        };
        self.position = Vector::from_mm(self.position.to_mm() + distance);
    }

    pub async fn move_to(&mut self, dst: Vector) {
        let delta = dst.to_mm() - self.position.to_mm();
        let direction = if delta.is_sign_negative() {
            StepperDirection::CounterClockwise
        } else {
            StepperDirection::Clockwise
        };
        self.set_direction(direction);
        let distance = Vector::from_mm((delta as f32).abs() as f64);
        self.move_for(distance).await;
    }

    pub fn get_position(&self) -> Vector {
        self.position
    }

    pub fn reset(&mut self) -> () {
        self.position = Vector::from_mm(0.0);
        self.direction = StepperDirection::Clockwise;
        self.step_duration = compute_step_duration(
            self.steps_per_revolution,
            self.distance_per_step,
            Vector::from_mm(0.0)
        );
    }
}
