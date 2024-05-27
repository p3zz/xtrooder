use core::time::Duration;
use math::common::compute_step_duration;
use math::common::{abs, compute_revolutions_per_second, floor, RotationDirection};
use math::computable::Computable;
use math::distance::Distance;
use math::speed::Speed;

pub trait TimerTrait{
    async fn after(duration: Duration);
}

pub trait StatefulOutputPin{
    fn set_high(&mut self);
    fn set_low(&mut self);
    fn is_high(&self) -> bool;
}

#[derive(Clone, Copy)]
pub struct StepperAttachment {
    pub distance_per_step: Distance,
}

impl Default for StepperAttachment {
    fn default() -> Self {
        Self {
            distance_per_step: Distance::from_mm(1.0),
        }
    }
}

#[derive(Clone, Copy)]
pub struct StepperOptions {
    pub steps_per_revolution: u64,
    pub stepping_mode: SteppingMode,
    pub bounds: Option<(f64, f64)>,
    pub positive_direction: RotationDirection,
}

impl Default for StepperOptions {
    fn default() -> Self {
        Self {
            steps_per_revolution: 200,
            stepping_mode: SteppingMode::FullStep,
            bounds: None,
            positive_direction: RotationDirection::Clockwise,
        }
    }
}

#[derive(Debug)]
pub enum StepperError {
    MoveTooShort,
    MoveOutOfBounds,
    MoveNotValid,
    MissingAttachment,
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
            SteppingMode::HalfStep => 1 << 1,
            SteppingMode::QuarterStep => 1 << 2,
            SteppingMode::EighthStep => 1 << 3,
            SteppingMode::SixteenthStep => 1 << 4,
        }
    }
}

pub struct Stepper<P: StatefulOutputPin> {
    // properties that won't change
    step: P,
    dir: P,
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

impl<P: StatefulOutputPin> Stepper<P> {
    pub fn new(
        step: P,
        dir: P,
        options: StepperOptions,
        attachment: Option<StepperAttachment>,
    ) -> Self {
        Self {
            step,
            dir,
            options,
            attachment: attachment,
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
            self.options.steps_per_revolution,
        ) {
            Ok(d) => d,
            Err(_) => return Err(StepperError::MoveNotValid),
        };
        let micros = (step_duration.as_micros() as f64
            / f64::from(u8::from(self.options.stepping_mode))) as u64;
        self.step_duration = Duration::from_micros(micros);
        Ok(())
    }

    pub fn set_speed_from_attachment(&mut self, speed: Speed) -> Result<(), StepperError> {
        if self.attachment.is_none() {
            return Err(StepperError::MissingAttachment);
        }
        let attachment = self.attachment.unwrap();
        let rps = speed.to_revolutions_per_second(
            self.options.steps_per_revolution,
            attachment.distance_per_step,
        );
        self.set_speed(rps)
    }

    // this option must be modifiable so that during the execution we can freely switch between different stepping modes for higher precision
    pub fn set_stepping_mode(&mut self, mode: SteppingMode) {
        self.options.stepping_mode = mode;
    }

    #[cfg(test)]
    pub fn set_attachment(&mut self, attachment: StepperAttachment) {
        self.attachment = Some(attachment);
    }

    #[cfg(test)]
    pub fn set_options(&mut self, options: StepperOptions) {
        self.options = options;
    }

    pub fn set_direction(&mut self, direction: RotationDirection) {
        match direction {
            RotationDirection::Clockwise => self.dir.set_high(),
            RotationDirection::CounterClockwise => self.dir.set_low(),
        };
    }

    pub fn get_direction(&mut self) -> RotationDirection {
        if self.dir.is_high() {
            RotationDirection::Clockwise
        } else {
            RotationDirection::CounterClockwise
        }
    }

    pub fn step(&mut self) -> Result<(), StepperError> {
        let mut step = 1.0 / f64::from(u8::from(self.options.stepping_mode));
        // if we are going counterclockwise but the positive direction is counterclockwise, the step is positive
        // if we are going clockwise but the positive direction is clockwise, the step is positive
        // if we are going counterclockwise but the positive direction is clockwise, the step is negative
        // if we are going clockwise but the positive direction is counterclockwise, the step is negative
        let dir = i8::from(self.options.positive_direction) * i8::from(self.get_direction());
        step *= f64::from(dir);
        let steps_next = self.steps + step;
        if let Some((min, max)) = self.options.bounds {
            if steps_next < min || steps_next > max {
                return Err(StepperError::MoveOutOfBounds);
            }
        }

        self.step.set_high();
        self.step.set_low();
        
        self.steps = steps_next;
        Ok(())
    }

    pub async fn move_for_steps<T: TimerTrait>(&mut self, steps: u64) -> Result<(), StepperError> {
        if steps == 0 {
            return Ok(());
        }
        if self.step_duration.as_micros() == 0 {
            return Err(StepperError::MoveNotValid);
        }
        for _ in 0..steps {
            self.step()?;
            T::after(self.step_duration).await;
        }
        Ok(())
    }

    fn move_for_distance_inner(&mut self, distance: Distance) -> Result<u64, StepperError> {
        if self.attachment.is_none() {
            return Err(StepperError::MissingAttachment);
        }
        let attachment = self.attachment.unwrap();

        let steps_n = abs(distance.to_mm()) / attachment.distance_per_step.to_mm();

        let steps_n = floor(steps_n) as u64;

        // the steps number is computed using distance_per_step that is the distance covered by the stepper
        // when running on full-step mode.
        // if the stepping mode is half-step or below, we need to adapt the number of steps to cover the correct
        // distance as well
        let steps_n = steps_n * u64::from(u8::from(self.options.stepping_mode));

        let direction = if distance.to_mm().is_sign_positive() {
            RotationDirection::Clockwise
        } else {
            RotationDirection::CounterClockwise
        };

        self.set_direction(direction);

        Ok(steps_n)
    }

    pub async fn move_for_distance<T: TimerTrait>(&mut self, distance: Distance) -> Result<(), StepperError> {
        let steps = self.move_for_distance_inner(distance)?;
        self.move_for_steps::<T>(steps).await
    }

    fn move_to_destination_inner(
        &mut self,
        destination: Distance,
    ) -> Result<Distance, StepperError> {
        let p = self.get_position()?;
        let distance = destination.sub(&p);
        Ok(distance)
    }

    pub async fn move_to_destination<T: TimerTrait>(&mut self, destination: Distance) -> Result<(), StepperError> {
        let distance = self.move_to_destination_inner(destination)?;
        self.move_for_distance::<T>(distance).await
    }

    pub fn get_position(&self) -> Result<Distance, StepperError> {
        let steps = self.get_steps();
        match self.attachment {
            Some(a) => Ok(Distance::from_mm(steps * a.distance_per_step.to_mm())),
            None => Err(StepperError::MissingAttachment),
        }
    }

    pub fn get_steps(&self) -> f64 {
        self.steps
    }

    pub fn get_speed(&self) -> f64 {
        compute_revolutions_per_second(
            self.step_duration,
            self.options.steps_per_revolution,
        )
    }

    pub fn get_speed_from_attachment(&self) -> Result<Speed, StepperError> {
        if let Some(attachment) = self.attachment {
            let rev_per_second = self.get_speed() / f64::from(u8::from(self.options.stepping_mode));
            return Ok(Speed::from_revolutions_per_second(
                rev_per_second,
                self.options.steps_per_revolution,
                attachment.distance_per_step,
            ));
        }
        Err(StepperError::MissingAttachment)
    }

    pub async fn home<T: TimerTrait>(&mut self) -> Result<(), StepperError> {
        self.move_to_destination::<T>(Distance::from_mm(0.0)).await
    }

    #[cfg(test)]
    fn reset(&mut self) {
        self.step_duration = Duration::from_secs(1);
        self.dir.set_low();
        self.steps = 0f64;
        self.options = StepperOptions::default();
        self.attachment = None;
    }
}

#[cfg(test)]
mod tests{
    use math::{common::RotationDirection, distance::Distance, speed::Speed};
    use tokio::time::sleep;

    use super::*;

    struct StatefulOutputPinMock{
        state: bool
    }
    
    impl StatefulOutputPinMock{
        pub fn new() -> Self {
            Self{
                state: false,
            }
        }
    }
    
    impl StatefulOutputPin for StatefulOutputPinMock{
        fn set_high(&mut self) {
            self.state = true;
        }
    
        fn set_low(&mut self) {
            self.state = false;
        }
    
        fn is_high(&self) -> bool {
            self.state
        }
    }

    struct StepperTimer{}

    impl TimerTrait for StepperTimer{
        async fn after(duration: core::time::Duration) {
            sleep(duration).await
        }
    }
    

    #[test]
    fn always_passes(){
        assert!(true);
    }

    #[test]
    fn test_stepper_step() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        s.set_direction(RotationDirection::Clockwise);
        let res = s.step();
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 1.0);
    }

    #[test]
    fn test_stepper_step_out_of_bounds() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let mut options = StepperOptions::default();
        options.bounds = Some((-1.0, 1.0));
        let mut s = Stepper::new(step, direction, options, None);
        s.set_direction(RotationDirection::Clockwise);
        let res = s.step();
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 1.0);
        let res = s.step();
        assert!(res.is_err());
        assert_eq!(s.get_steps(), 1.0);
    }

    #[tokio::test]
    async fn test_stepper_move_for_steps_fail() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        s.set_direction(RotationDirection::Clockwise);
        let res = s.set_speed(0.0);
        assert!(res.is_ok());
        let steps = 20;
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_stepper_move_for_steps_success() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        s.set_direction(RotationDirection::Clockwise);
        s.set_speed(1.0).unwrap();
        let steps = 20;
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 20.0);
        assert_eq!(s.get_speed(), 1.0);
    }

    #[tokio::test]
    async fn test_stepper_move_counterclockwise() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let steps = 20;
        s.set_direction(RotationDirection::CounterClockwise);
        s.set_speed(5.0).unwrap();
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -20.0);
    }

    #[tokio::test]
    async fn test_stepper_move_microstepping_clockwise() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let steps = 20;
        s.set_stepping_mode(SteppingMode::HalfStep);
        s.set_direction(RotationDirection::Clockwise);
        s.set_speed(5.0).unwrap();
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 10.0);
    }

    #[tokio::test]
    async fn test_stepper_move_microstepping_counterclockwise() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let steps = 20;
        s.set_stepping_mode(SteppingMode::HalfStep);
        s.set_direction(RotationDirection::CounterClockwise);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -10.0);
    }

    #[tokio::test]
    async fn test_stepper_move_clockwise_positive_direction_clockwise() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let steps = 20;
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::Clockwise);
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::Clockwise;
        s.set_options(options);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 20.0);
    }

    #[tokio::test]
    async fn test_stepper_move_clockwise_positive_direction_counterclockwise() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let steps = 20;
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::Clockwise);
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::CounterClockwise;
        s.set_options(options);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -20.0);
    }

    #[tokio::test]
    async fn test_stepper_move_counterclockwise_positive_direction_clockwise() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let steps = 20;
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::CounterClockwise);
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::Clockwise;
        s.set_options(options);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -20.0);
    }

    #[tokio::test]
    async fn test_stepper_move_counterclockwise_positive_direction_counterclockwise(){
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let steps = 20;
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::CounterClockwise);
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::CounterClockwise;
        s.set_options(options);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 20.0);
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance_no_attachment() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let distance = Distance::from_mm(20.0);
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let distance = Distance::from_mm(10.0);
        s.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 10.0);
        assert!(s.get_position().is_ok());
        assert_eq!(s.get_position().unwrap().to_mm(), 10.0);
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance_space_wasted() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let distance = Distance::from_mm(10.5);
        s.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 10.0);
        assert!(s.get_position().is_ok());
        assert_eq!(s.get_position().unwrap().to_mm(), 10.0);
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance_space_wasted_2() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let distance = Distance::from_mm(0.5);
        s.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 0.0);
        assert!(s.get_position().is_ok());
        assert_eq!(s.get_position().unwrap().to_mm(), 0.0);
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance_space_wasted_3() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let distance = Distance::from_mm(-0.5);
        s.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 0.0);
        assert!(s.get_position().is_ok());
        assert_eq!(s.get_position().unwrap().to_mm(), 0.0);
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance_lower_distance_per_step() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let distance = Distance::from_mm(10.5);
        s.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(0.5),
        });
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 21.0);
        assert!(s.get_position().is_ok());
        assert_eq!(s.get_position().unwrap().to_mm(), 10.5);
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance_negative() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let distance = Distance::from_mm(-10.5);
        s.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(0.5),
        });
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -21.0);
        assert!(s.get_position().is_ok());
        assert_eq!(s.get_position().unwrap().to_mm(), -10.5);
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance_zero() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let distance = Distance::from_mm(0.0);
        s.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(0.5),
        });
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 0.0);
        assert!(s.get_position().is_ok());
        assert_eq!(s.get_position().unwrap().to_mm(), 0.0);
    }

    #[tokio::test]
    async fn test_stepper_move_for_steps_outofbounds() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let steps = 10;
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::CounterClockwise);
        let mut options = StepperOptions::default();
        options.bounds = Some((-10.0, 10.0));
        s.set_options(options);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -10.0);

        let steps = 15;
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_err());
        assert_eq!(s.get_steps(), -10.0);
    }

    #[tokio::test]
    async fn test_stepper_home() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let steps = 10;
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_attachment(StepperAttachment::default());

        s.set_direction(RotationDirection::Clockwise);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 10.0);

        let res = s.home::<StepperTimer>().await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 0.0);
    }

    #[tokio::test]
    async fn test_stepper_home_no_attachment() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        s.set_stepping_mode(SteppingMode::FullStep);

        let res = s.home::<StepperTimer>().await;
        assert!(res.is_err());
        assert_eq!(s.get_steps(), 0.0);
    }

    #[test]
    fn test_stepper_set_speed_positive() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        s.set_stepping_mode(SteppingMode::FullStep);
        let res = s.set_speed(1.0);
        assert!(res.is_ok());
        assert_eq!(s.get_speed(), 1.0);
    }

    #[test]
    fn test_stepper_set_speed_zero() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        s.set_stepping_mode(SteppingMode::FullStep);
        let res = s.set_speed(0.0);
        assert!(res.is_ok());
    }

    #[test]
    fn test_stepper_set_speed_negative() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        s.set_stepping_mode(SteppingMode::FullStep);
        let res = s.set_speed(-10.0);
        assert!(res.is_err());
    }

    #[test]
    fn test_stepper_set_speed_from_attachment_no_attachment() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        let res = s.set_speed_from_attachment(Speed::from_mm_per_second(3.0));
        assert!(res.is_err());
    }

    #[test]
    fn test_stepper_set_speed_from_attachment_positive() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        s.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = s.set_speed_from_attachment(Speed::from_mm_per_second(3.0));
        assert!(res.is_ok());
    }

    #[test]
    fn test_stepper_set_speed_from_attachment_negative() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        s.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = s.set_speed_from_attachment(Speed::from_mm_per_second(-3.0));
        assert!(res.is_err());
    }

    #[test]
    fn test_stepper_set_speed_from_attachment_zero() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options, None);
        s.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = s.set_speed_from_attachment(Speed::from_mm_per_second(0.0));
        assert!(res.is_ok());
        assert_eq!(s.get_speed(), 0.0);
    }

}
