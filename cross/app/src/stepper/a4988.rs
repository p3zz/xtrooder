#![allow(dead_code)]

use math::common::abs;
use math::computable::Computable;
use math::distance::Distance;
use math::speed::Speed;
use embassy_stm32::gpio::Output;
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::SimplePwm;
use embassy_stm32::timer::{CaptureCompare16bitInstance, Channel};
use embassy_time::{Duration, Timer};

use math::common::compute_step_duration;
use {defmt_rtt as _, panic_probe as _};

#[derive(Clone, Copy)]
pub enum StepperDirection {
    Clockwise,
    CounterClockwise,
}

pub struct Stepper<'s, S> {
    // properties that won't change
    step: SimplePwm<'s, S>,
    step_ch: Channel,
    dir: Output<'s>,
    steps_per_revolution: u64,
    distance_per_step: Distance,
    bounds: (Distance, Distance),

    // properties that have to be computed and kept updated during the execution
    speed: Speed,
    position: Distance,
    direction: StepperDirection,
}

impl<'s, S> Stepper<'s, S>
where
    S: CaptureCompare16bitInstance,
{
    pub fn new(
        mut step: SimplePwm<'s, S>,
        step_ch: Channel,
        dir: Output<'s>,
        steps_per_revolution: u64,
        distance_per_step: Distance,
    ) -> Stepper<'s, S> {
        // the duty is 50% (in order to have high/low pin for the same amount of time)
        // TODO do we really need to set the duty to 50%?
        step.set_duty(step_ch, step.get_max_duty() / 2);
        Stepper {
            step,
            step_ch,
            dir,
            steps_per_revolution,
            distance_per_step,
            speed: Speed::from_mm_per_second(0.0),
            position: Distance::from_mm(0.0),
            direction: StepperDirection::Clockwise,
            bounds: (Distance::from_mm(-10000.0), Distance::from_mm(10000.0)),
        }
    }

    /*
    update the speed an dcompute the frequency in which the pwm must run.
    pwm frequency: count of PWM interval periods per second
    PWM period: duration of one complete cycle or the total amount of active and inactive time combined
    */
    pub fn set_speed(&mut self, speed: Speed) -> () {
        self.speed = speed;
    }

    // the stepping is implemented through a pwm,
    // and the frequency is computed using the time for a step to be executed (step duration)
    pub async fn move_for(&mut self, distance: Distance) -> Result<(), ()> {
        if abs(distance.to_mm()) < self.distance_per_step.to_mm(){
            return Err(());
        }

        let position_next = self.position.add(&distance);
        if position_next.to_mm() < self.bounds.0.to_mm()
            || position_next.to_mm() > self.bounds.1.to_mm()
        {
            return Err(());
        }

        let step_duration = compute_step_duration(
            self.steps_per_revolution,
            self.distance_per_step,
            self.speed,
        );
        let step_duration = step_duration.expect("Invalid step duration");
        let freq = hz(((1.0 / step_duration.as_micros() as f64) * 1_000_000.0) as u32);
        self.step.set_frequency(freq);

        
        self.direction = if distance.to_mm().is_sign_positive() {
            StepperDirection::Clockwise
        } else {
            StepperDirection::CounterClockwise
        };

        match self.direction {
            StepperDirection::Clockwise => self.dir.set_high(),
            StepperDirection::CounterClockwise => self.dir.set_low(),
        };

        let steps_n = (abs(distance.to_mm()) / self.distance_per_step.to_mm()) as u64;

        // compute the duration of the move.
        let duration = Duration::from_micros(steps_n * step_duration.as_micros() as u64);

        // move
        self.step.enable(self.step_ch);
        Timer::after(duration).await;
        self.step.disable(self.step_ch);

        self.position = position_next;

        Ok(())
    }

    pub async fn move_to(&mut self, dst: Distance) {
        let delta = dst.sub(&self.position);
        self.move_for(delta).await;
    }

    pub fn get_position(&self) -> Distance {
        self.position
    }

    pub fn get_direction(&self) -> StepperDirection {
        self.direction
    }

    pub fn get_speed(&self) -> Speed {
        self.speed
    }

    pub async fn home(&mut self) {
        self.move_to(Distance::from_mm(0.0)).await;
    }

    pub fn reset(&mut self) -> () {
        self.speed = Speed::from_mm_per_second(0.0);
        self.position = Distance::from_mm(0.0);
        self.direction = StepperDirection::Clockwise;
    }
}
