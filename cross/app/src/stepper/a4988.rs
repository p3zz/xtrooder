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
    pub bounds: Option<(f64, f64)>,
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

impl From<SteppingMode> for u8 {
    fn from(value: SteppingMode) -> Self {
        match value {
            SteppingMode::FullStep => 1,
            SteppingMode::HalfStep => 1<<1,
            SteppingMode::QuarterStep => 1<<2,
            SteppingMode::EighthStep => 1<<3,
            SteppingMode::SixteenthStep => 1<<4,
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
    // a step is a single step in full-step mode. Every step performed in another stepping mode
    // will result in a fraction of a step
    steps: f64,
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
            steps: 0f64,
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
        let duty_ratio = self.step.get_duty(Channel::Ch1) as f64 / self.step.get_max_duty() as f64;
        let micros = (step_duration.as_micros() as f64 * duty_ratio / f64::from(u8::from(self.options.stepping_mode))) as u64;
        self.step_duration = Duration::from_micros(micros);
        let freq = hz(((1.0 / self.step_duration.as_micros() as f64) * 1_000_000.0) as u32);
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

    pub fn set_stepping_mode(&mut self, mode: SteppingMode){
        self.options.stepping_mode = mode;
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
    // here a single step matches with the current stepping mode. If the stepping mode is full-step,
    // the stepper will step by 1 step, and it will be recorded as 1 full-step
    // if the stepping mode is on half-step, the stepper will step by 1 step, and it will be recorded as 1/2 full-step
    #[cfg(not(test))]
    pub async fn move_for_steps(&mut self, steps: u64) -> Result<(), StepperError> {
        let duration = self.move_for_steps_inner(steps)?;
        self.step.enable(Channel::Ch1);
        self.step.enable(Channel::Ch2);
        self.step.enable(Channel::Ch3);
        self.step.enable(Channel::Ch4);

        Timer::after(duration).await;

        self.step.disable(Channel::Ch1);
        self.step.disable(Channel::Ch2);
        self.step.disable(Channel::Ch3);
        self.step.disable(Channel::Ch4);
        
        Ok(())
    }

    #[cfg(test)]
    pub fn move_for_steps(&mut self, steps: u64) -> Result<(), StepperError> {
        let _duration = self.move_for_steps_inner(steps)?;
        Ok(())
    }

    fn move_for_steps_inner(&mut self, steps: u64) -> Result<Duration, StepperError> {
        info!("executed");
        let s = match self.get_direction(){
            RotationDirection::Clockwise => steps as i64,
            RotationDirection::CounterClockwise => -(steps as i64),
        };

        // compute the steps as a fraction of a full step, using the stepping mode.
        // this way we can keep track of steps in different step modes
        let s = (s as f64)/ (f64::from(u8::from(self.options.stepping_mode)));
        let steps_next = self.steps + s;

        if let Some((min, max)) = self.options.bounds{
            if steps_next < min || steps_next > max{
                return Err(StepperError::MoveOutOfBounds);
            }
        }

        self.steps = steps_next;

        // step exactly the number of steps passed to the function
        let duration = Duration::from_micros(steps * self.step_duration.as_micros() as u64);

        info!(
            "Steps: {} Total duration: {} us Step duration: {} us Direction: {}",
            steps,
            duration.as_micros(),
            self.step_duration.as_micros(),
            u8::from(self.get_direction())
        );

        Ok(duration)
    }
    

    fn move_for_distance_inner(&mut self, distance: Distance) -> Result<u64, StepperError> {
        if self.attachment.is_none(){
            return Err(StepperError::MissingAttachment)
        }
        let attachment = self.attachment.unwrap();
        
        let steps_n = (distance.div(&attachment.distance_per_step).unwrap() as f32).floor() as i64;

        // the steps number is computed using distance_per_step that is the distance covered by the stepper
        // when running on full-step mode.
        // if the stepping mode is half-step or below, we need to adapt the number of steps to cover the correct
        // distance as well
        let steps_n = steps_n * i64::from(u8::from(self.options.stepping_mode));
        
        let (steps_n, direction) = if steps_n.is_positive(){
            (steps_n as u64, RotationDirection::Clockwise)
        }else{
            (-steps_n as u64, RotationDirection::CounterClockwise)
        };

        self.set_direction(direction);

        Ok(steps_n)
    }

    #[cfg(test)]
    pub fn move_for_distance(&mut self, distance: Distance) -> Result<(), StepperError> {
        let steps = self.move_for_distance_inner(distance)?;
        self.move_for_steps(steps)
    }

    #[cfg(not(test))]
    pub async fn move_for_distance(&mut self, distance: Distance) -> Result<(), StepperError> {
        let steps = self.move_for_distance_inner(distance)?;
        self.move_for_steps(steps).await
    }

    fn move_to_destination_inner(&mut self, destination: Distance) -> Result<Distance, StepperError> {
        let p = self.get_position()?;
        let distance = destination.sub(&p);
        Ok(distance)
        // self.move_for_distance(distance).await
    }

    #[cfg(test)]
    pub fn move_to_destination(&mut self, destination: Distance) -> Result<(), StepperError> {
        let distance = self.move_to_destination_inner(destination)?;
        self.move_for_distance(distance)
    }

    #[cfg(not(test))]
    pub async fn move_to_destination(&mut self, destination: Distance) -> Result<(), StepperError> {
        let distance = self.move_to_destination_inner(destination)?;
        self.move_for_distance(distance).await
    }


    pub fn get_position(&self) -> Result<Distance, StepperError> {
        match self.attachment{
            Some(a) => Ok(Distance::from_mm(self.steps * a.distance_per_step.to_mm())),
            None => Err(StepperError::MissingAttachment)
        }
    }

    pub fn get_steps(&self) -> f64 {
        self.steps
    }

    pub fn get_speed(&self) -> f64 {
        compute_revolutions_per_second(core::time::Duration::from_micros(self.step_duration.as_micros()), self.options.steps_per_revolution)
    }

    #[cfg(not(test))]
    pub async fn home(&mut self) -> Result<(), StepperError> {
        self.move_to_destination(Distance::from_mm(0.0)).await
    }

    #[cfg(test)]
    pub fn home(&mut self) -> Result<(), StepperError> {
        self.move_to_destination(Distance::from_mm(0.0))
    }

    pub fn reset(&mut self) {
        self.step_duration = Duration::from_secs(1);
        self.steps = 0f64;
    }
}

#[cfg(test)]
#[defmt_test::tests]
mod tests {
    use defmt_rtt as _;
    use embassy_stm32::{gpio::{Level, Output, OutputType, Speed as PinSpeed}, peripherals::TIM5, time::hz, timer::{simple_pwm::{PwmPin, SimplePwm}, Channel, CountingMode}, Peripherals};
    use math::{common::RotationDirection, distance::Distance};
    use panic_probe as _;
    use defmt::assert;

    use crate::stepper::a4988::{Stepper, SteppingMode};

    #[init]
    fn init() -> Stepper<'static, TIM5>{
        let p = embassy_stm32::init(embassy_stm32::Config::default());

        let step = SimplePwm::new(
            p.TIM5,
            Some(PwmPin::new_ch1(p.PA0, OutputType::PushPull)),
            None,
            None,
            None,
            hz(1),
            CountingMode::EdgeAlignedUp,
        );

        let dir = Output::new(p.PB0, Level::Low, PinSpeed::Low);

        Stepper::new(
            step,
            dir,
        )

    }

    #[test]
    fn test_stepper_move_clockwise(s: &mut Stepper<'static, TIM5>) {
        let steps = 20;
        s.reset();
        s.set_direction(RotationDirection::Clockwise);
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 20.0);
    }

    #[test]
    fn test_stepper_move_counterclockwise(s: &mut Stepper<'static, TIM5>) {
        let steps = 20;
        s.reset();
        s.set_direction(RotationDirection::CounterClockwise);
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -20.0);
    }

    #[test]
    fn test_stepper_move_microstepping_clockwise(s: &mut Stepper<'static, TIM5>) {
        let steps = 20;
        s.reset();
        s.set_stepping_mode(SteppingMode::HalfStep);
        s.set_direction(RotationDirection::Clockwise);
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 10.0);
    }

    #[test]
    fn test_stepper_move_microstepping_counterclockwise(s: &mut Stepper<'static, TIM5>) {
        let steps = 20;
        s.reset();
        s.set_stepping_mode(SteppingMode::HalfStep);
        s.set_direction(RotationDirection::CounterClockwise);
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -10.0);
    }

    #[test]
    fn always_passes() {
        assert!(true);
    }
    
    // #[test]
    // fn always_passes_3(s: &mut Stepper<'static, TIM5>) {
    //     assert!(true);
    // }
}