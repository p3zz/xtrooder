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
    pub positive_direction: RotationDirection,
}

impl Default for StepperOptions{
    fn default() -> Self {
        Self { steps_per_revolution: 200, stepping_mode: SteppingMode::FullStep, bounds: None, positive_direction: RotationDirection::Clockwise }
    }
}

#[derive(Debug)]
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

pub struct Stepper<'s> {
    // properties that won't change
    step: Output<'s>,
    dir: Output<'s>,
    options: StepperOptions,
    attachment: Option<StepperAttachment>,
    // properties that have to be computed and kept updated during the execution
    // we need to keep the set speed because we can't get the frequency from the pwm pin to compute the speed
    step_duration: Duration,
    // a step is a single step in full-step mode. Every step performed in another stepping mode
    // will result in a fraction of a step
    // steps are positive when the stepper moves in clockwise order
    steps: f64,
}

impl<'s> Stepper<'s>
{
    pub fn new(
        step: Output<'s>,
        dir: Output<'s>,
        options: StepperOptions,
        attachment: Option<StepperAttachment>
    ) -> Self {
        
        Self {
            step,
            dir,
            options,
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
    pub fn set_speed(&mut self, revolutions_per_second: f64) -> Result<(), StepperError> {
        let step_duration = match compute_step_duration(
            revolutions_per_second,
            self.options.steps_per_revolution
        ){
            Ok(d) => d,
            Err(_) => return Err(StepperError::MoveNotValid),
        };
        let micros = (step_duration.as_micros() as f64 / f64::from(u8::from(self.options.stepping_mode))) as u64;
        self.step_duration = Duration::from_micros(micros);
        Ok(())
    }

    pub fn set_speed_from_attachment(&mut self, speed: Speed) -> Result<(), StepperError> {
        if self.attachment.is_none(){
            return Err(StepperError::MissingAttachment);
        }
        let attachment = self.attachment.unwrap();
        let rps = speed.to_revolutions_per_second(self.options.steps_per_revolution, attachment.distance_per_step);
        self.set_speed(rps)
    }

    // this option must be modifiable so that during the execution we can freely switch between different stepping modes for higher precision
    pub fn set_stepping_mode(&mut self, mode: SteppingMode){
        self.options.stepping_mode = mode;
    }

    #[cfg(test)]
    pub fn set_attachment(&mut self, attachment: StepperAttachment){
        self.attachment = Some(attachment);
    }

    #[cfg(test)]
    pub fn set_options(&mut self, options: StepperOptions){
        self.options = options;
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

    pub fn step_inner(&mut self) -> Result<(), StepperError>{
        let mut step = 1.0 / f64::from(u8::from(self.options.stepping_mode));
        // if we are going counterclockwise but the positive direction is counterclockwise, the step is positive
        // if we are going clockwise but the positive direction is clockwise, the step is positive
        // if we are going counterclockwise but the positive direction is clockwise, the step is negative
        // if we are going clockwise but the positive direction is counterclockwise, the step is negative
        let dir = i8::from(self.options.positive_direction) * i8::from(self.get_direction());
        step *= f64::from(dir);
        let steps_next = self.steps + step;
        if let Some((min, max)) = self.options.bounds{
            if steps_next < min || steps_next > max{
                return Err(StepperError::MoveOutOfBounds);
            }
        }
        self.steps = steps_next;
        Ok(())
    }

    #[cfg(not(test))]
    pub async fn step(&mut self) -> Result<(), StepperError> {
        self.step_inner()?;
        self.step.set_high();
        self.step.set_low();
        Timer::after(self.step_duration).await;
        Ok(())
    }

    #[cfg(test)]
    pub fn step(&mut self) -> Result<(), StepperError> {
        self.step_inner()
    }

    #[cfg(not(test))]
    pub async fn move_for_steps(&mut self, steps: u64) -> Result<(), StepperError> {
        info!("Steps: {}, Step duration: {} us", steps, self.step_duration.as_micros());
        for _ in 0..steps{
            self.step().await?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn move_for_steps(&mut self, steps: u64) -> Result<(), StepperError> {
        for _ in 0..steps{
            self.step()?;
        }
        Ok(())
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
        let steps = self.get_steps();
        match self.attachment{
            Some(a) => Ok(Distance::from_mm(steps * a.distance_per_step.to_mm())),
            None => Err(StepperError::MissingAttachment)
        }
    }

    pub fn get_steps(&self) -> f64 {
        self.steps
    }

    pub fn get_speed(&self) -> f64 {
        compute_revolutions_per_second(core::time::Duration::from_micros(self.step_duration.as_micros()), self.options.steps_per_revolution)
    }

    pub fn get_speed_from_attachment(&self) -> Result<Speed, StepperError>{
        if let Some(attachment) = self.attachment{
            let rev_per_second = self.get_speed() / f64::from(u8::from(self.options.stepping_mode));
            return Ok(Speed::from_revolutions_per_second(rev_per_second, self.options.steps_per_revolution, attachment.distance_per_step));
        }
        Err(StepperError::MissingAttachment)
    }

    #[cfg(not(test))]
    pub async fn home(&mut self) -> Result<(), StepperError> {
        self.move_to_destination(Distance::from_mm(0.0)).await
    }

    #[cfg(test)]
    pub fn home(&mut self) -> Result<(), StepperError> {
        self.move_to_destination(Distance::from_mm(0.0))
    }

    #[cfg(test)]
    pub fn reset(&mut self) {
        self.step_duration = Duration::from_secs(1);
        self.steps = 0f64;
        self.options = StepperOptions::default();
        self.attachment = None;
    }
}
