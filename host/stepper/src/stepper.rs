use core::marker::PhantomData;
use core::time::Duration;
use math::common::{angular_velocity_from_speed, angular_velocity_from_steps, compute_step_duration, speed_from_angular_velocity};
use math::common::{abs, floor, RotationDirection};
use math::measurements::{AngularVelocity, Distance, Speed};

pub trait TimerTrait {
    async fn after(duration: Duration);
}

pub trait StatefulOutputPin {
    fn set_high(&mut self);
    fn set_low(&mut self);
    fn is_high(&self) -> bool;
}

pub trait StepperInputPin {
    fn is_high(&self) -> bool;
}

#[derive(Clone, Copy)]
pub struct StepperAttachment {
    pub distance_per_step: Distance,
}

impl Default for StepperAttachment {
    fn default() -> Self {
        Self {
            distance_per_step: Distance::from_millimeters(1.0),
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

#[derive(Debug, PartialEq)]
pub enum StepperError {
    MoveTooShort,
    MoveOutOfBounds,
    MoveNotValid,
    MissingAttachment,
    NotSupported,
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

pub struct NotAttached {}
pub struct Attached {}

pub trait AttachmentMode {}

impl AttachmentMode for Attached {}
impl AttachmentMode for NotAttached {}

pub struct Stepper<P: StatefulOutputPin, M: AttachmentMode> {
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
    // used to keep the attachment mode
    _attachment_mode: PhantomData<M>,
}

impl<P: StatefulOutputPin, M: AttachmentMode> Stepper<P, M> {
    fn new_inner(
        step: P,
        dir: P,
        attachment: Option<StepperAttachment>,
        options: StepperOptions,
    ) -> Self {
        Self {
            step,
            dir,
            options,
            attachment,
            step_duration: Duration::from_secs(1),
            steps: 0f64,
            _attachment_mode: PhantomData,
        }
    }

    /*
    update the speed an dcompute the frequency in which the pwm must run.
    pwm frequency: count of PWM interval periods per second
    PWM period: duration of one complete cycle or the total amount of active and inactive time combined
    */
    pub fn set_speed(&mut self, angular_velocity: AngularVelocity) {
        let step_duration = compute_step_duration(
            angular_velocity,
            self.options.steps_per_revolution,
        );
        let micros = (step_duration.as_micros() as f64 / f64::from(u8::from(self.options.stepping_mode))) as u64;
        self.step_duration = Duration::from_micros(micros);
    }

    // this option must be modifiable so that during the execution we can freely switch between different stepping modes for higher precision
    pub fn set_stepping_mode(&mut self, mode: SteppingMode) {
        self.options.stepping_mode = mode;
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

    pub async fn move_for_steps<T: TimerTrait>(
        &mut self,
        steps: u64,
    ) -> Result<Duration, StepperError> {
        if steps == 0 || self.step_duration.is_zero() {
            return Ok(Duration::ZERO);
        }

        let mut total_duration = Duration::ZERO;
        for _ in 0..steps {
            self.step()?;
            T::after(self.step_duration).await;
            total_duration += self.step_duration;
        }
        Ok(total_duration)
    }

    pub fn get_steps(&self) -> f64 {
        self.steps
    }

    pub fn set_steps(&mut self, steps: f64) {
        self.steps = steps;
    }

    pub fn get_speed(&self) -> AngularVelocity {
        angular_velocity_from_steps(self.step_duration, self.options.steps_per_revolution)
    }

    pub fn get_options(&self) -> StepperOptions {
        self.options
    }

    pub fn get_step_duration(&self) -> Duration {
        self.step_duration
    }

    // if the steps are negative and the positive direction is clockwise, we need to go clockwise
    // if the steps are negative and the positive direction is counter-clockwise, we need to go counter-clockwise
    // if the steps are positive and the positive direction is clockwise, we need to go counter-clockwise
    // if the steps are positive and the positive direction is counter-clockwise, we need to go clockwise
    pub async fn home<T: TimerTrait>(&mut self) -> Result<Duration, StepperError> {
        let sign = self.steps * f64::from(i8::from(self.options.positive_direction));
        let direction = if sign.is_sign_positive() {
            RotationDirection::CounterClockwise
        } else {
            RotationDirection::Clockwise
        };
        self.set_direction(direction);
        // we need to get the total number of effective steps we have already performed, so we can
        // come back to the origin (0). steps member is normalized in full-steps, so we need to multiply
        // it by the stepping mode we're in
        let steps = (abs(self.steps) * f64::from(u8::from(self.options.stepping_mode))) as u64;
        self.move_for_steps::<T>(steps).await
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

impl<P: StatefulOutputPin> Stepper<P, NotAttached> {
    pub fn new(step: P, dir: P, options: StepperOptions) -> Self {
        Self::new_inner(step, dir, None, options)
    }
}

impl<P: StatefulOutputPin> Stepper<P, Attached> {
    pub fn new_with_attachment(
        step: P,
        dir: P,
        options: StepperOptions,
        attachment: StepperAttachment,
    ) -> Self {
        Self::new_inner(step, dir, Some(attachment), options)
    }

    pub fn set_speed_from_attachment(&mut self, speed: Speed) {
        let attachment = self.attachment.unwrap();
        let angular_velocity = angular_velocity_from_speed(speed, self.options.steps_per_revolution, attachment.distance_per_step);
        self.set_speed(angular_velocity);
    }

    fn move_for_distance_inner(&mut self, distance: Distance) -> u64 {
        let attachment = self.attachment.unwrap();

        let steps_n = abs(distance / attachment.distance_per_step);

        let steps_n = floor(steps_n) as u64;

        // the steps number is computed using distance_per_step that is the distance covered by the stepper
        // when running on full-step mode.
        // if the stepping mode is half-step or below, we need to adapt the number of steps to cover the correct
        // distance as well
        let steps_n = steps_n * u64::from(u8::from(self.options.stepping_mode));

        let direction = if distance.as_millimeters().is_sign_positive() {
            RotationDirection::Clockwise
        } else {
            RotationDirection::CounterClockwise
        };

        self.set_direction(direction);

        steps_n
    }

    pub async fn move_for_distance<T: TimerTrait>(
        &mut self,
        distance: Distance,
    ) -> Result<Duration, StepperError> {
        let steps = self.move_for_distance_inner(distance);
        self.move_for_steps::<T>(steps).await
    }

    fn move_to_destination_inner(
        &mut self,
        destination: Distance,
    ) -> Distance {
        let p = self.get_position();
        
        destination - p
    }

    pub async fn move_to_destination<T: TimerTrait>(
        &mut self,
        destination: Distance,
    ) -> Result<Duration, StepperError> {
        let distance = self.move_to_destination_inner(destination);
        self.move_for_distance::<T>(distance).await
    }

    pub fn get_position(&self) -> Distance {
        let attachment = self.attachment.unwrap();
        let steps = self.get_steps();
        steps * attachment.distance_per_step
    }

    pub fn get_speed_from_attachment(&self) -> Speed {
        let attachment = self.attachment.unwrap();
        let rev_per_second = self.get_speed() / f64::from(u8::from(self.options.stepping_mode));
        speed_from_angular_velocity(rev_per_second, self.options.steps_per_revolution, attachment.distance_per_step)
    }
}

#[cfg(test)]
mod tests {
    use math::{common::RotationDirection, measurements::{Distance, Speed}};
    use tokio::time::sleep;
    use approx::assert_abs_diff_eq;

    use super::*;

    struct StatefulOutputPinMock {
        state: bool,
    }

    impl StatefulOutputPinMock {
        pub fn new() -> Self {
            Self { state: false }
        }
    }

    impl StatefulOutputPin for StatefulOutputPinMock {
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

    struct StepperTimer {}

    impl TimerTrait for StepperTimer {
        async fn after(duration: core::time::Duration) {
            sleep(duration).await
        }
    }

    #[test]
    fn always_passes() {
        assert!(true);
    }

    #[test]
    fn test_stepper_step() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        s.set_direction(RotationDirection::Clockwise);
        let res = s.step();
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 1.0, epsilon = 0.000001);
    }

    #[test]
    fn test_stepper_step_out_of_bounds() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let mut options = StepperOptions::default();
        options.bounds = Some((-1.0, 1.0));
        let mut s = Stepper::new(step, direction, options);
        s.set_direction(RotationDirection::Clockwise);
        let res = s.step();
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 1.0, epsilon = 0.000001);
        let res = s.step();
        assert!(res.is_err());
        assert_abs_diff_eq!(s.get_steps(), 1.0, epsilon = 0.000001);
    }

    #[tokio::test]
    async fn test_stepper_move_for_steps_fail() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        s.set_direction(RotationDirection::Clockwise);
        let angular_velocity = AngularVelocity::from_rpm(0.0);
        s.set_speed(angular_velocity);
        let steps = 20;
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 0.0, epsilon = 0.000001)
    }

    #[tokio::test]
    async fn test_stepper_move_for_steps_success() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        s.set_direction(RotationDirection::Clockwise);
        let angular_velocity = AngularVelocity::from_rpm(60.0);
        s.set_speed(angular_velocity);
        let steps = 20;
        let m = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(m.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 20.0, epsilon = 0.000001);
        assert_eq!(s.get_speed(), angular_velocity);
        assert_eq!(
            m.unwrap().as_micros(),
            Duration::from_millis(100).as_micros()
        );
    }

    #[tokio::test]
    async fn test_stepper_move_counterclockwise() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        let steps = 20;
        s.set_direction(RotationDirection::CounterClockwise);
        let angular_velocity = AngularVelocity::from_rpm(300.0);
        s.set_speed(angular_velocity);
        let m = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(m.is_ok());
        assert_abs_diff_eq!(s.get_steps(), -20.0, epsilon = 0.000001);
        assert_eq!(
            m.unwrap().as_micros(),
            Duration::from_millis(20).as_micros()
        );
    }

    #[tokio::test]
    async fn test_stepper_move_microstepping_clockwise() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        let steps = 20;
        let angular_velocity = AngularVelocity::from_rpm(300.0);
        s.set_stepping_mode(SteppingMode::HalfStep);
        s.set_direction(RotationDirection::Clockwise);
        s.set_speed(angular_velocity);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 10.0, epsilon = 0.000001);
    }

    #[tokio::test]
    async fn test_stepper_move_microstepping_counterclockwise() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        let steps = 20;
        s.set_stepping_mode(SteppingMode::HalfStep);
        s.set_direction(RotationDirection::CounterClockwise);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), -10.0, epsilon = 0.000001);
    }

    #[tokio::test]
    async fn test_stepper_move_clockwise_positive_direction_clockwise() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        let steps = 20;
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::Clockwise);
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::Clockwise;
        s.set_options(options);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 20.0, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_stepper_move_clockwise_positive_direction_counterclockwise() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        let steps = 20;
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::Clockwise);
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::CounterClockwise;
        s.set_options(options);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), -20.0, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_stepper_move_counterclockwise_positive_direction_clockwise() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        let steps = 20;
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::CounterClockwise);
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::Clockwise;
        s.set_options(options);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), -20.0, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_stepper_move_counterclockwise_positive_direction_counterclockwise() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        let steps = 20;
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::CounterClockwise);
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::CounterClockwise;
        s.set_options(options);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 20.0, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s =
            Stepper::new_with_attachment(step, direction, options, StepperAttachment::default());
        let distance = Distance::from_millimeters(10.0);
        let m = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(m.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 10.0, epsilon=0.000001);
        assert_abs_diff_eq!(s.get_position().as_millimeters(), 10.0, epsilon=0.000001);
        assert_eq!(m.unwrap().as_micros(), Duration::from_secs(10).as_micros());
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance_space_wasted() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s =
            Stepper::new_with_attachment(step, direction, options, StepperAttachment::default());
        let distance = Distance::from_millimeters(10.5);
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 10.0, epsilon=0.000001);
        assert_abs_diff_eq!(s.get_position().as_millimeters(), 10.0, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance_space_wasted_2() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s =
            Stepper::new_with_attachment(step, direction, options, StepperAttachment::default());
        let distance = Distance::from_millimeters(0.5);
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 0.0, epsilon=0.000001);
        assert_abs_diff_eq!(s.get_position().as_millimeters(), 0.0, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance_space_wasted_3() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s =
            Stepper::new_with_attachment(step, direction, options, StepperAttachment::default());
        let distance = Distance::from_millimeters(-0.5);
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 0.0, epsilon=0.000001);
        assert_abs_diff_eq!(s.get_position().as_millimeters(), 0.0, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance_lower_distance_per_step() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new_with_attachment(
            step,
            direction,
            options,
            StepperAttachment {
                distance_per_step: Distance::from_millimeters(0.5),
            },
        );
        let distance = Distance::from_millimeters(10.5);
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 21.0, epsilon=0.000001);
        assert_abs_diff_eq!(s.get_position().as_millimeters(), 10.5, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance_negative() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new_with_attachment(
            step,
            direction,
            options,
            StepperAttachment {
                distance_per_step: Distance::from_millimeters(0.5),
            },
        );
        let distance = Distance::from_millimeters(-10.5);
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), -21.0, epsilon=0.000001);
        assert_abs_diff_eq!(s.get_position().as_millimeters(), -10.5, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_stepper_move_for_distance_zero() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new_with_attachment(
            step,
            direction,
            options,
            StepperAttachment {
                distance_per_step: Distance::from_millimeters(0.5),
            },
        );
        let distance = Distance::from_millimeters(0.0);
        let res = s.move_for_distance::<StepperTimer>(distance).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 0.0, epsilon=0.000001);
        assert_abs_diff_eq!(s.get_position().as_millimeters(), 0.0, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_stepper_move_for_steps_outofbounds() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        let steps = 10;
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::CounterClockwise);
        let mut options = StepperOptions::default();
        options.bounds = Some((-10.0, 10.0));
        s.set_options(options);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), -10.0, epsilon=0.000001);

        let steps = 15;
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_err());
        assert_abs_diff_eq!(s.get_steps(), -10.0, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_stepper_home() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s =
            Stepper::new_with_attachment(step, direction, options, StepperAttachment::default());
        let steps = 10;
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_direction(RotationDirection::Clockwise);
        let res = s.move_for_steps::<StepperTimer>(steps).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 10.0);

        let res = s.home::<StepperTimer>().await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 0.0, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_stepper_home_no_attachment() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        s.set_stepping_mode(SteppingMode::FullStep);

        let res = s.home::<StepperTimer>().await;
        assert!(res.is_ok());
    }

    #[test]
    fn test_stepper_set_speed_positive() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        let angular_velocity = AngularVelocity::from_rpm(60.0);
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_speed(angular_velocity);
        assert_eq!(s.get_speed(), angular_velocity);
    }

    #[test]
    fn test_stepper_set_speed_zero() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        let angular_velocity = AngularVelocity::from_rpm(0.0);
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_speed(angular_velocity);
        assert_eq!(s.get_speed(), angular_velocity);
    }

    #[test]
    fn test_stepper_set_speed_negative() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let mut s = Stepper::new(step, direction, options);
        let angular_velocity = AngularVelocity::from_rpm(-600.0);
        s.set_stepping_mode(SteppingMode::FullStep);
        s.set_speed(angular_velocity);
        assert_eq!(s.get_speed(), AngularVelocity::from_rpm(0.0));
    }

    #[test]
    fn test_stepper_set_speed_from_attachment_positive() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let speed = Speed::from_meters_per_second(0.003);
        let mut s =
            Stepper::new_with_attachment(step, direction, options, StepperAttachment::default());
        s.set_speed_from_attachment(speed);
        assert_abs_diff_eq!(s.get_speed_from_attachment().as_metres_per_second(), speed.as_meters_per_second(), epsilon=0.000001);
    }

    #[test]
    fn test_stepper_set_speed_from_attachment_negative() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let speed = Speed::from_meters_per_second(-3.0);
        let mut s =
            Stepper::new_with_attachment(step, direction, options, StepperAttachment::default());
        s.set_speed_from_attachment(speed);
        assert_abs_diff_eq!(s.get_speed_from_attachment().as_metres_per_second(), 0.0, epsilon=0.000001);
    }

    #[test]
    fn test_stepper_set_speed_from_attachment_zero() {
        let step = StatefulOutputPinMock::new();
        let direction = StatefulOutputPinMock::new();
        let options = StepperOptions::default();
        let speed = Speed::from_meters_per_second(-3.0);
        let mut s =
            Stepper::new_with_attachment(step, direction, options, StepperAttachment::default());
        s.set_speed_from_attachment(speed);
        assert_abs_diff_eq!(s.get_speed_from_attachment().as_metres_per_second(), 0.0, epsilon=0.000001);
    }
}
