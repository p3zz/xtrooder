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
    // the option positive_direction changes only the sign of the steps retrieved by get_steps and get_position
    // if the positive_direction is counterclockwise and steps = 100, the value returned by get_steps will be -100
    // if the positive_direction is clockwise and steps = 100, the value returned by get_steps will be 100
    steps: f64,
}

impl<'s> Stepper<'s>
{
    pub fn new(
        step: Output<'s>,
        dir: Output<'s>,
        options: StepperOptions
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
        self.set_speed(rps)?;
        Ok(())
    }

    // this option must be modifiable so that during the execution we can freely switch between different stepping modes for higher precision
    pub fn set_stepping_mode(&mut self, mode: SteppingMode){
        self.options.stepping_mode = mode;
    }

    pub fn set_attachment(&mut self, attachment: StepperAttachment){
        self.attachment = Some(attachment);
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

    pub async fn step(&mut self){
        self.step.set_high();
        self.step.set_low();
        Timer::after(self.step_duration).await
    }


    #[cfg(not(test))]
    pub async fn move_for_steps(&mut self, steps: u64) -> Result<(), StepperError> {
        let steps_next = self.check_move_valid(steps)?;
        info!("Steps: {}, Step duration: {} us", steps, self.step_duration.as_micros());
        for _ in 0..steps{
            self.step().await;
        }
        self.steps = steps_next;
        Ok(())
    }

    #[cfg(test)]
    pub fn move_for_steps(&mut self, steps: u64) -> Result<(), StepperError> {
        let steps_next = self.check_move_valid(steps)?;
        self.steps = steps_next;
        Ok(())
    }

    fn check_move_valid(&mut self, steps: u64) -> Result<f64, StepperError> {
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

        return Ok(steps_next);
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
        match self.options.positive_direction{
            RotationDirection::Clockwise => self.steps,
            RotationDirection::CounterClockwise => -self.steps,
        }
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

    fn reset(&mut self) {
        self.step_duration = Duration::from_secs(1);
        self.steps = 0f64;
        self.options = StepperOptions::default();
        self.attachment = None;
    }
}

#[cfg(test)]
#[defmt_test::tests]
mod tests {
    use defmt_rtt as _;
    use embassy_stm32::gpio::{Level, Output, Speed as PinSpeed};
    use math::{common::RotationDirection, distance::Distance};
    use panic_probe as _;
    use defmt::assert;

    use super::{Stepper, StepperAttachment, StepperOptions, SteppingMode};

    #[init]
    fn init() -> Stepper<'static>{
        let p = embassy_stm32::init(embassy_stm32::Config::default());

        let step = Output::new(p.PA0, Level::Low, PinSpeed::Low);

        let dir = Output::new(p.PB0, Level::Low, PinSpeed::Low);

        Stepper::new(
            step,
            dir,
            StepperOptions::default()
        )

    }

    #[test]
    fn test_stepper_move_clockwise(s: &mut Stepper<'static>) {
        let steps = 20;
        s.reset();
        s.set_direction(RotationDirection::Clockwise);
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 20.0);
        assert_eq!(s.get_speed(), 0.005);
    }

    #[test]
    fn test_stepper_move_counterclockwise(s: &mut Stepper<'static>) {
        let steps = 20;
        s.reset();
        s.set_direction(RotationDirection::CounterClockwise);
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -20.0);
    }

    #[test]
    fn test_stepper_move_microstepping_clockwise(s: &mut Stepper<'static>) {
        let steps = 20;
        s.reset();
        s.set_stepping_mode(SteppingMode::HalfStep);
        s.set_direction(RotationDirection::Clockwise);
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 10.0);
    }

    #[test]
    fn test_stepper_move_microstepping_counterclockwise(s: &mut Stepper<'static>) {
        let steps = 20;
        s.reset();
        s.set_stepping_mode(SteppingMode::HalfStep);
        s.set_direction(RotationDirection::CounterClockwise);
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -10.0);
    }

    #[test]
    fn test_stepper_move_clockwise_positive_direction_clockwise(s: &mut Stepper<'static>) {
        let steps = 20;
        s.reset();
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::Clockwise);
        // positive direction cannot be modified from api. This is only for test
        s.options.positive_direction = RotationDirection::Clockwise;
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 20.0);
    }

    #[test]
    fn test_stepper_move_clockwise_positive_direction_counterclockwise(s: &mut Stepper<'static>) {
        let steps = 20;
        s.reset();
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::Clockwise);
        // positive direction cannot be modified from api. This is only for test
        s.options.positive_direction = RotationDirection::CounterClockwise;
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -20.0);
    }

    #[test]
    fn test_stepper_move_counterclockwise_positive_direction_clockwise(s: &mut Stepper<'static>) {
        let steps = 20;
        s.reset();
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::CounterClockwise);
        // positive direction cannot be modified from api. This is only for test
        s.options.positive_direction = RotationDirection::Clockwise;
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -20.0);
    }

    #[test]
    fn test_stepper_move_counterclockwise_positive_direction_counterclockwise(s: &mut Stepper<'static>) {
        let steps = 20;
        s.reset();
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::CounterClockwise);
        // positive direction cannot be modified from api. This is only for test
        s.options.positive_direction = RotationDirection::CounterClockwise;
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 20.0);
    }

    #[test]
    fn test_stepper_move_for_distance_no_attachment(s: &mut Stepper<'static>) {
        let distance = Distance::from_mm(20.0);
        s.reset();
        let res = s.move_for_distance(distance);
        assert!(res.is_err());
    }

    #[test]
    fn test_stepper_move_for_distance(s: &mut Stepper<'static>) {
        let distance = Distance::from_mm(10.0);
        s.reset();
        s.set_attachment(StepperAttachment { distance_per_step: Distance::from_mm(1.0) });
        let res = s.move_for_distance(distance);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 10.0);
        assert!(s.get_position().is_ok());
        assert_eq!(s.get_position().unwrap().to_mm(), 10.0);
    }

    #[test]
    fn test_stepper_move_for_distance_space_wasted(s: &mut Stepper<'static>) {
        let distance = Distance::from_mm(10.5);
        s.reset();
        s.set_attachment(StepperAttachment { distance_per_step: Distance::from_mm(1.0) });
        let res = s.move_for_distance(distance);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 10.0);
        assert!(s.get_position().is_ok());
        assert_eq!(s.get_position().unwrap().to_mm(), 10.0);
    }

    #[test]
    fn test_stepper_move_for_distance_lower_distance_per_step(s: &mut Stepper<'static>) {
        let distance = Distance::from_mm(10.5);
        s.reset();
        s.set_attachment(StepperAttachment { distance_per_step: Distance::from_mm(0.5) });
        let res = s.move_for_distance(distance);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 21.0);
        assert!(s.get_position().is_ok());
        assert_eq!(s.get_position().unwrap().to_mm(), 10.5);
    }

    #[test]
    fn test_stepper_move_for_steps_outofbounds(s: &mut Stepper<'static>) {
        let steps = 10;
        s.reset();
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::CounterClockwise);
        s.options.bounds = Some((-10.0, 10.0));
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -10.0);

        let steps = 15;
        let res = s.move_for_steps(steps);
        assert!(res.is_err());
        assert_eq!(s.get_steps(), -10.0);
    }

    #[test]
    fn test_stepper_home(s: &mut Stepper<'static>) {
        let steps = 10;
        s.reset();
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_attachment(StepperAttachment::default());

        s.set_direction(RotationDirection::Clockwise);
        let res = s.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 10.0);

        let res = s.home();
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 0.0);
    }

    #[test]
    fn test_stepper_home_no_attachment(s: &mut Stepper<'static>) {
        s.reset();
        s.set_stepping_mode(SteppingMode::FullStep);

        let res = s.home();
        assert!(res.is_err());
        assert_eq!(s.get_steps(), 0.0);
    }

    #[test]
    fn test_stepper_set_speed_positive(s: &mut Stepper<'static>) {
        s.reset();
        s.set_stepping_mode(SteppingMode::FullStep);
        let res = s.set_speed(1.0);
        assert!(res.is_ok());
        assert_eq!(s.get_speed(), 0.9992006394884093);
    }

    #[test]
    fn test_stepper_set_speed_zero(s: &mut Stepper<'static>) {
        s.reset();
        s.set_stepping_mode(SteppingMode::FullStep);
        let res = s.set_speed(0.0);
        assert!(res.is_err());
    }

    #[test]
    fn test_stepper_set_speed_negative(s: &mut Stepper<'static>) {
        s.reset();
        s.set_stepping_mode(SteppingMode::FullStep);
        let res = s.set_speed(-10.0);
        assert!(res.is_err());
    }

    #[test]
    fn always_passes() {
        assert!(true);
    }
    
}