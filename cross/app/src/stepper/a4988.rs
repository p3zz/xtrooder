use defmt::info;
use embassy_stm32::gpio::Output;
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::SimplePwm;
use embassy_stm32::timer::{CaptureCompare16bitInstance, Channel};
use embassy_time::{Duration, Timer};
use math::common::{abs, compute_revolutions_per_second, RotationDirection};
use math::computable::Computable;
use math::distance::Distance;
use math::speed::Speed;
use micromath::F32Ext;

use math::common::compute_step_duration;

#[derive(Clone, Copy)]
pub struct StepperAttachment{
    pub distance_per_step: Distance,
}

impl Default for StepperAttachment{
    fn default() -> Self {
        Self { distance_per_step: Distance::from_mm(1.0) }
    }
}

#[derive(Clone, Copy)]
pub struct StepperOptions{
    pub steps_per_revolution: u64,
    pub stepping_mode: SteppingMode,
    pub bounds: Option<(i64, i64)>,
    pub positive_position: RotationDirection,
}

impl Default for StepperOptions{
    fn default() -> Self {
        Self { steps_per_revolution: 200, stepping_mode: SteppingMode::FullStep, bounds: None, positive_position: RotationDirection::Clockwise }
    }
}

pub enum StepperError {
    MoveTooShort,
    MoveOutOfBounds,
    MoveNotValid,
    MissingAttachment
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
    dir: Output<'s>,
    options: StepperOptions,
    attachment: Option<StepperAttachment>,
    // properties that have to be computed and kept updated during the execution
    // we need to keep the set speed because we can't get the frequency from the pwm pin to compute the speed
    step_duration: Duration,
    steps: i64,
}

impl<'s, S> Stepper<'s, S>
where
    S: CaptureCompare16bitInstance,
{
    pub fn new(
        mut step: SimplePwm<'s, S>,
        dir: Output<'s>,
    ) -> Stepper<'s, S> {
        step.set_duty(Channel::Ch1, step.get_max_duty() / 2);
        step.set_duty(Channel::Ch2, step.get_max_duty() / 2);
        step.set_duty(Channel::Ch3, step.get_max_duty() / 2);
        step.set_duty(Channel::Ch4, step.get_max_duty() / 2);
        
        Stepper {
            step,
            dir,
            options: StepperOptions::default(),
            attachment: None,
            step_duration: Duration::from_secs(1),
            steps: 0,
        }
    }

    /*
    update the speed an dcompute the frequency in which the pwm must run.
    pwm frequency: count of PWM interval periods per second
    PWM period: duration of one complete cycle or the total amount of active and inactive time combined
    */
    pub fn set_speed(&mut self, revolutions_per_second: f64) {
        let step_duration = compute_step_duration(
            revolutions_per_second,
            self.options.steps_per_revolution
        );
        let step_duration = step_duration.expect("Invalid step duration");
        self.step_duration = Duration::from_micros(step_duration.as_micros() as u64); 
        let freq = hz(((1.0 / step_duration.as_micros() as f64) * 1_000_000.0) as u32);
        self.step.set_frequency(freq);
    }

    pub fn set_speed_from_attachment(&mut self, speed: Speed) -> Result<(), StepperError> {
        if self.attachment.is_none(){
            return Err(StepperError::MissingAttachment);
        }
        let attachment = self.attachment.unwrap();
        let rps = speed.to_revolutions_per_second(self.options.steps_per_revolution, attachment.distance_per_step);
        self.set_speed(rps);
        Ok(())
    }

    pub fn set_direction(&mut self, direction: RotationDirection){
        match direction{
            RotationDirection::Clockwise => self.dir.set_high(),
            RotationDirection::CounterClockwise => self.dir.set_low(),
        };
    }

    pub fn get_direction(&self) -> RotationDirection {
        if self.dir.is_set_high(){
            RotationDirection::Clockwise
        }else{
            RotationDirection::CounterClockwise
        }
    }

    // the stepping is implemented through a pwm,
    // and the frequency is computed using the time for a step to be executed (step duration)
    pub async fn move_for_steps(&mut self, steps: u64) -> Result<(), StepperError> {
        let s = match self.get_direction(){
            RotationDirection::Clockwise => steps as i64,
            RotationDirection::CounterClockwise => -(steps as i64),
        };
        let steps_next = self.steps + s;

        if let Some((min, max)) = self.options.bounds{
            if steps_next < min || steps_next > max{
                return Err(StepperError::MoveOutOfBounds);
            }
        }
        let duration = Duration::from_micros(steps * self.step_duration.as_micros() as u64);

        info!(
            "Steps: {} Total duration: {} us Step duration: {} us Direction: {}",
            steps,
            duration.as_micros(),
            self.step_duration.as_micros(),
            u8::from(self.get_direction())
        );
        
        self.step.enable(Channel::Ch1);
        self.step.enable(Channel::Ch2);
        self.step.enable(Channel::Ch3);
        self.step.enable(Channel::Ch4);

        Timer::after(duration).await;

        self.step.disable(Channel::Ch1);
        self.step.disable(Channel::Ch2);
        self.step.disable(Channel::Ch3);
        self.step.disable(Channel::Ch4);

        self.steps = steps_next;

        Ok(())
    }

    pub async fn move_for_distance(&mut self, distance: Distance) -> Result<(), StepperError> {
        if self.attachment.is_none(){
            return Err(StepperError::MissingAttachment)
        }
        let attachment = self.attachment.unwrap();
        
        let steps_n = (distance.div(&attachment.distance_per_step).unwrap() as f32).floor() as i64;
        
        let direction = if steps_n.is_positive(){
            RotationDirection::Clockwise
        }else{
            RotationDirection::CounterClockwise
        };

        self.set_direction(direction);

        let steps_n = if steps_n.is_negative(){
            -steps_n as u64
        }else{
            steps_n as u64
        };

        self.move_for_steps(steps_n).await
    }

    pub async fn move_to_destination(&mut self, destination: Distance) -> Result<(), StepperError> {
        let p = self.get_position()?;
        let delta = destination.sub(&p);
        self.move_for_distance(delta).await
    }

    pub fn get_position(&self) -> Result<Distance, StepperError> {
        match self.attachment{
            Some(a) => Ok(Distance::from_mm(self.steps as f64 * a.distance_per_step.to_mm())),
            None => Err(StepperError::MissingAttachment)
        }
    }

    pub fn get_speed(&self) -> f64 {
        compute_revolutions_per_second(core::time::Duration::from_micros(self.step_duration.as_micros()), self.options.steps_per_revolution)
    }

    pub async fn home(&mut self) -> Result<(), StepperError> {
        self.move_to_destination(Distance::from_mm(0.0)).await
    }

    pub fn reset(&mut self) {
        self.step_duration = Duration::from_secs(1);
        self.steps = 0;
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