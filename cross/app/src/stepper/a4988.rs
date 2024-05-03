use defmt::info;
use embassy_stm32::gpio::Output;
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::SimplePwm;
use embassy_stm32::timer::{CaptureCompare16bitInstance, Channel};
use embassy_time::{Duration, Timer};
use math::common::{abs, RotationDirection};
use math::computable::Computable;
use math::distance::Distance;
use math::speed::Speed;

use math::common::compute_step_duration;

pub enum StepperError {
    MoveTooShort,
    MoveOutOfBounds,
    MoveNotValid,
}

#[derive(Clone, Copy)]
pub enum SteppingMode {
    FullStep,
    HalfStep,
    QuarterStep,
    EighthStep,
    SixteenthStep,
}

impl From<SteppingMode> for u64 {
    fn from(value: SteppingMode) -> Self {
        match value {
            SteppingMode::FullStep => 1,
            SteppingMode::HalfStep => 2,
            SteppingMode::QuarterStep => 4,
            SteppingMode::EighthStep => 8,
            SteppingMode::SixteenthStep => 16,
        }
    }
}

pub struct Stepper<'s, S> {
    // properties that won't change
    step: SimplePwm<'s, S>,
    step_ch: Channel,
    dir: Output<'s>,
    steps_per_revolution: u64,
    distance_per_step: Distance,
    bounds: (Distance, Distance),
    stepping_mode: SteppingMode,
    positive_heading: RotationDirection,
    // properties that have to be computed and kept updated during the execution
    speed: Speed,
    position: Distance,
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
        stepping_mode: SteppingMode,
    ) -> Stepper<'s, S> {
        // the duty is 50% (in order to have high/low pin for the same amount of time)
        // TODO do we really need to set the duty to 50%?
        step.set_duty(step_ch, (step.get_max_duty() as f64 * 0.5) as u16);
        Stepper {
            step,
            step_ch,
            dir,
            steps_per_revolution,
            distance_per_step: Distance::from_mm(
                distance_per_step.to_mm() / (u64::from(stepping_mode) as f64),
            ),
            speed: Speed::from_mm_per_second(0.0),
            position: Distance::from_mm(0.0),
            bounds: (Distance::from_mm(-12_000.0), Distance::from_mm(12_000.0)),
            stepping_mode,
            positive_heading: RotationDirection::Clockwise,
        }
    }

    // select how the stepper has to move (clockwise or counter-clockwise) in order to
    // perform a positive move. Use this if the stepper is mounted so that a positive move
    // is done with a counter-clockwise rotation
    pub fn set_positive_heading(&mut self, direction: RotationDirection) {
        self.positive_heading = direction;
    }

    /*
    update the speed an dcompute the frequency in which the pwm must run.
    pwm frequency: count of PWM interval periods per second
    PWM period: duration of one complete cycle or the total amount of active and inactive time combined
    */
    pub fn set_speed(&mut self, speed: Speed) {
        self.speed = speed;
    }

    // the stepping is implemented through a pwm,
    // and the frequency is computed using the time for a step to be executed (step duration)
    pub async fn move_for(&mut self, distance: Distance) -> Result<(), StepperError> {
        if abs(distance.to_mm()) < self.distance_per_step.to_mm() {
            return Err(StepperError::MoveTooShort);
        }

        let position_next = self.position.add(&distance);
        if position_next.to_mm() < self.bounds.0.to_mm()
            || position_next.to_mm() > self.bounds.1.to_mm()
        {
            return Err(StepperError::MoveOutOfBounds);
        }

        let step_duration = compute_step_duration(
            self.steps_per_revolution * u64::from(self.stepping_mode),
            self.distance_per_step,
            self.speed,
        );
        let step_duration = step_duration.expect("Invalid step duration");
        let freq = hz(((1.0 / step_duration.as_micros() as f64) * 1_000_000.0) as u32);
        self.step.set_frequency(freq);

        let distance = match self.positive_heading {
            RotationDirection::Clockwise => distance.to_mm(),
            RotationDirection::CounterClockwise => -distance.to_mm(),
        };

        if distance.is_sign_positive() {
            self.dir.set_high()
        } else {
            self.dir.set_low()
        };

        let steps_n = (abs(distance) / self.distance_per_step.to_mm()) as u64;

        // compute the duration of the move.
        let duration = Duration::from_micros(steps_n * step_duration.as_micros() as u64);

        info!(
            "Steps: {} Total duration: {} Step duration: {} Direction: {}",
            steps_n,
            duration.as_micros(),
            step_duration.as_micros(),
            u8::from(self.get_direction())
        );
        // move
        self.step.enable(self.step_ch);
        Timer::after(duration).await;
        self.step.disable(self.step_ch);

        self.position = position_next;

        Ok(())
    }

    pub async fn move_to(&mut self, dst: Distance) -> Result<(), StepperError> {
        let delta = dst.sub(&self.position);
        self.move_for(delta).await
    }

    pub fn get_position(&self) -> Distance {
        self.position
    }

    pub fn get_direction(&self) -> RotationDirection {
        if self.dir.is_set_high(){
            RotationDirection::Clockwise
        }else{
            RotationDirection::CounterClockwise
        }
    }

    pub fn get_speed(&self) -> Speed {
        self.speed
    }

    pub async fn home(&mut self) -> Result<(), StepperError> {
        self.move_to(Distance::from_mm(0.0)).await
    }

    pub fn reset(&mut self) {
        self.speed = Speed::from_mm_per_second(0.0);
        self.position = Distance::from_mm(0.0);
    }
}

#[cfg(test)]
#[defmt_test::tests]
mod tests {
    use defmt_rtt as _;
    // use embassy_stm32::{gpio::{Level, Output, OutputType, Speed as PinSpeed}, peripherals::TIM5, time::hz, timer::{simple_pwm::{PwmPin, SimplePwm}, Channel, CountingMode}, Peripherals};
    // use math::distance::Distance;
    use panic_probe as _;
    // use defmt::assert;

    // use crate::stepper::a4988::{Stepper, SteppingMode};

    // #[init]
    // fn init() -> Stepper<'static, TIM5>{
    //     let p = embassy_stm32::init(embassy_stm32::Config::default());

    //     let step = SimplePwm::new(
    //         p.TIM5,
    //         Some(PwmPin::new_ch1(p.PA0, OutputType::PushPull)),
    //         None,
    //         None,
    //         None,
    //         hz(1),
    //         CountingMode::EdgeAlignedUp,
    //     );

    //     let dir = Output::new(p.PB0, Level::Low, PinSpeed::Low);

    //     Stepper::new(
    //         step,
    //         Channel::Ch1,
    //         dir,
    //         200,
    //         Distance::from_mm(0.15f64),
    //         SteppingMode::FullStep,
    //     )

    // }

    // #[test]
    // fn always_passes(s: &mut Stepper<'static, TIM5>) {
    //     let dst = Distance::from_mm(20.0);
    //     s.move_to(dst);
    // }

    #[test]
    fn always_passes() {
        assert!(true);
    }
    
    // #[test]
    // fn always_passes_3(s: &mut Stepper<'static, TIM5>) {
    //     assert!(true);
    // }
}