#![no_std]
#![no_main]

pub mod hotend;
pub mod planner;
pub mod stepper;
pub mod utils;

#[cfg(test)]
#[defmt_test::tests]
mod tests {
    use defmt_rtt as _;
    use embassy_stm32::gpio::{Level, Output, Speed as PinSpeed};
    use math::{common::RotationDirection, distance::Distance};
    use panic_probe as _;
    use defmt::assert;

    use crate::stepper::a4988::{Stepper, StepperAttachment, StepperOptions, SteppingMode};

    #[init]
    fn init() -> Stepper<'static>{
        let p = embassy_stm32::init(embassy_stm32::Config::default());

        let step = Output::new(p.PA0, Level::Low, PinSpeed::Low);

        let dir = Output::new(p.PB0, Level::Low, PinSpeed::Low);

        Stepper::new(
            step,
            dir,
            StepperOptions::default(),
            None
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
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::Clockwise;
        s.set_options(options);
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
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::CounterClockwise;
        s.set_options(options);
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
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::Clockwise;
        s.set_options(options);
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
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::CounterClockwise;
        s.set_options(options);
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
        let mut options = StepperOptions::default();
        options.bounds = Some((-10.0, 10.0));
        s.set_options(options);
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

    // #[test]
    // fn test_stepper_move_clockwise(steppers: &mut (Stepper<'static>, Stepper<'static>)) {
    //     let destination = Distance::from_mm(10.0);
    //     let speed = Speed::from_mm_per_second(10.0);
    //     steppers.0.reset();
    //     let res = motion::linear_move_to(&mut steppers.0, destination, speed);
    //     assert!(res.is_ok());
    //     assert_eq!(steppers.0.get_steps(), 20.0);
    //     assert_eq!(steppers.0.get_position().unwrap().to_mm(), 10.0);
    // }
    
}