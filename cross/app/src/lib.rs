#![no_std]
#![no_main]

pub mod hotend;
pub mod planner;
pub mod stepper;
pub mod utils;

#[cfg(test)]
#[defmt_test::tests]
mod tests {
    use defmt::assert;
    use defmt_rtt as _;
    use embassy_stm32::gpio::{Level, Output, Speed as PinSpeed};
    use math::{
        common::RotationDirection,
        distance::Distance,
        speed::Speed,
        vector::{Vector2D, Vector3D},
    };
    use panic_probe as _;

    use crate::{
        planner::motion,
        stepper::a4988::{Stepper, StepperAttachment, StepperOptions, SteppingMode},
    };

    #[init]
    fn init() -> (Stepper<'static>, Stepper<'static>, Stepper<'static>) {
        let p = embassy_stm32::init(embassy_stm32::Config::default());

        let step = Output::new(p.PA0, Level::Low, PinSpeed::Low);

        let dir = Output::new(p.PB0, Level::Low, PinSpeed::Low);

        let a_stepper = Stepper::new(step, dir, StepperOptions::default(), None);

        let step = Output::new(p.PA1, Level::Low, PinSpeed::Low);

        let dir = Output::new(p.PB1, Level::Low, PinSpeed::Low);

        let b_stepper = Stepper::new(step, dir, StepperOptions::default(), None);

        let step = Output::new(p.PA2, Level::Low, PinSpeed::Low);

        let dir = Output::new(p.PB2, Level::Low, PinSpeed::Low);

        let c_stepper = Stepper::new(step, dir, StepperOptions::default(), None);

        (a_stepper, b_stepper, c_stepper)
    }

    #[test]
    fn test_stepper_step(s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>)) {
        s.0.reset();
        s.0.set_direction(RotationDirection::Clockwise);
        let res = s.0.step();
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 1.0);
    }

    #[test]
    fn test_stepper_step_out_of_bounds(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        s.0.reset();
        let mut options = StepperOptions::default();
        options.bounds = Some((-1.0, 1.0));
        s.0.set_options(options);
        s.0.set_direction(RotationDirection::Clockwise);
        let res = s.0.step();
        let res = s.0.step();
        assert!(res.is_err());
        assert_eq!(s.0.get_steps(), 1.0);
    }

    #[test]
    fn test_stepper_move_clockwise(s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>)) {
        let steps = 20;
        s.0.reset();
        s.0.set_direction(RotationDirection::Clockwise);
        let res = s.0.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 20.0);
        assert_eq!(s.0.get_speed(), 0.005);
    }

    #[test]
    fn test_stepper_move_counterclockwise(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let steps = 20;
        s.0.reset();
        s.0.set_direction(RotationDirection::CounterClockwise);
        let res = s.0.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), -20.0);
    }

    #[test]
    fn test_stepper_move_microstepping_clockwise(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let steps = 20;
        s.0.reset();
        s.0.set_stepping_mode(SteppingMode::HalfStep);
        s.0.set_direction(RotationDirection::Clockwise);
        let res = s.0.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 10.0);
    }

    #[test]
    fn test_stepper_move_microstepping_counterclockwise(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let steps = 20;
        s.0.reset();
        s.0.set_stepping_mode(SteppingMode::HalfStep);
        s.0.set_direction(RotationDirection::CounterClockwise);
        let res = s.0.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), -10.0);
    }

    #[test]
    fn test_stepper_move_clockwise_positive_direction_clockwise(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let steps = 20;
        s.0.reset();
        s.0.set_stepping_mode(SteppingMode::FullStep);
        s.0.set_direction(RotationDirection::Clockwise);
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::Clockwise;
        s.0.set_options(options);
        let res = s.0.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 20.0);
    }

    #[test]
    fn test_stepper_move_clockwise_positive_direction_counterclockwise(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let steps = 20;
        s.0.reset();
        s.0.set_stepping_mode(SteppingMode::FullStep);
        s.0.set_direction(RotationDirection::Clockwise);
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::CounterClockwise;
        s.0.set_options(options);
        let res = s.0.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), -20.0);
    }

    #[test]
    fn test_stepper_move_counterclockwise_positive_direction_clockwise(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let steps = 20;
        s.0.reset();
        s.0.set_stepping_mode(SteppingMode::FullStep);
        s.0.set_direction(RotationDirection::CounterClockwise);
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::Clockwise;
        s.0.set_options(options);
        let res = s.0.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), -20.0);
    }

    #[test]
    fn test_stepper_move_counterclockwise_positive_direction_counterclockwise(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let steps = 20;
        s.0.reset();
        s.0.set_stepping_mode(SteppingMode::FullStep);
        s.0.set_direction(RotationDirection::CounterClockwise);
        let mut options = StepperOptions::default();
        options.positive_direction = RotationDirection::CounterClockwise;
        s.0.set_options(options);
        let res = s.0.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 20.0);
    }

    #[test]
    fn test_stepper_move_for_distance_no_attachment(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let distance = Distance::from_mm(20.0);
        s.0.reset();
        let res = s.0.move_for_distance(distance);
        assert!(res.is_err());
    }

    #[test]
    fn test_stepper_move_for_distance(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let distance = Distance::from_mm(10.0);
        s.0.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = s.0.move_for_distance(distance);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 10.0);
        assert!(s.0.get_position().is_ok());
        assert_eq!(s.0.get_position().unwrap().to_mm(), 10.0);
    }

    #[test]
    fn test_stepper_move_for_distance_space_wasted(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let distance = Distance::from_mm(10.5);
        s.0.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = s.0.move_for_distance(distance);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 10.0);
        assert!(s.0.get_position().is_ok());
        assert_eq!(s.0.get_position().unwrap().to_mm(), 10.0);
    }

    #[test]
    fn test_stepper_move_for_distance_lower_distance_per_step(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let distance = Distance::from_mm(10.5);
        s.0.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(0.5),
        });
        let res = s.0.move_for_distance(distance);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 21.0);
        assert!(s.0.get_position().is_ok());
        assert_eq!(s.0.get_position().unwrap().to_mm(), 10.5);
    }

    #[test]
    fn test_stepper_move_for_distance_negative(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let distance = Distance::from_mm(-10.5);
        s.0.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(0.5),
        });
        let res = s.0.move_for_distance(distance);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), -21.0);
        assert!(s.0.get_position().is_ok());
        assert_eq!(s.0.get_position().unwrap().to_mm(), -10.5);
    }

    #[test]
    fn test_stepper_move_for_distance_zero(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let distance = Distance::from_mm(0.0);
        s.0.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(0.5),
        });
        let res = s.0.move_for_distance(distance);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 0.0);
        assert!(s.0.get_position().is_ok());
        assert_eq!(s.0.get_position().unwrap().to_mm(), 0.0);
    }

    #[test]
    fn test_stepper_move_for_steps_outofbounds(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let steps = 10;
        s.0.reset();
        s.0.set_stepping_mode(SteppingMode::FullStep);
        s.0.set_direction(RotationDirection::CounterClockwise);
        let mut options = StepperOptions::default();
        options.bounds = Some((-10.0, 10.0));
        s.0.set_options(options);
        let res = s.0.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), -10.0);

        let steps = 15;
        let res = s.0.move_for_steps(steps);
        assert!(res.is_err());
        assert_eq!(s.0.get_steps(), -10.0);
    }

    #[test]
    fn test_stepper_home(s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>)) {
        let steps = 10;
        s.0.reset();
        s.0.set_stepping_mode(SteppingMode::FullStep);
        s.0.set_attachment(StepperAttachment::default());

        s.0.set_direction(RotationDirection::Clockwise);
        let res = s.0.move_for_steps(steps);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 10.0);

        let res = s.0.home();
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 0.0);
    }

    #[test]
    fn test_stepper_home_no_attachment(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        s.0.reset();
        s.0.set_stepping_mode(SteppingMode::FullStep);

        let res = s.0.home();
        assert!(res.is_err());
        assert_eq!(s.0.get_steps(), 0.0);
    }

    #[test]
    fn test_stepper_set_speed_positive(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        s.0.reset();
        s.0.set_stepping_mode(SteppingMode::FullStep);
        let res = s.0.set_speed(1.0);
        assert!(res.is_ok());
        assert_eq!(s.0.get_speed(), 0.9992006394884093);
    }

    #[test]
    fn test_stepper_set_speed_zero(s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>)) {
        s.0.reset();
        s.0.set_stepping_mode(SteppingMode::FullStep);
        let res = s.0.set_speed(0.0);
        assert!(res.is_ok());
    }

    #[test]
    fn test_stepper_set_speed_negative(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        s.0.reset();
        s.0.set_stepping_mode(SteppingMode::FullStep);
        let res = s.0.set_speed(-10.0);
        assert!(res.is_err());
    }

    #[test]
    fn test_stepper_set_speed_from_attachment_no_attachment(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        s.0.reset();
        let res =
            s.0.set_speed_from_attachment(Speed::from_mm_per_second(3.0));
        assert!(res.is_err());
    }

    #[test]
    fn test_stepper_set_speed_from_attachment_positive(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        s.0.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res =
            s.0.set_speed_from_attachment(Speed::from_mm_per_second(3.0));
        assert!(res.is_ok());
    }

    #[test]
    fn test_stepper_set_speed_from_attachment_negative(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        s.0.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res =
            s.0.set_speed_from_attachment(Speed::from_mm_per_second(-3.0));
        assert!(res.is_err());
    }

    #[test]
    fn test_stepper_set_speed_from_attachment_zero(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        s.0.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res =
            s.0.set_speed_from_attachment(Speed::from_mm_per_second(0.0));
        assert!(res.is_ok());
        assert_eq!(s.0.get_speed(), 0.0);
    }

    #[test]
    fn always_passes() {
        assert!(true);
    }

    #[test]
    fn test_linear_move_to_no_move(s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>)) {
        let destination = Distance::from_mm(0.0);
        let speed = Speed::from_mm_per_second(10.0);
        s.0.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = motion::linear_move_to(&mut s.0, destination, speed);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 0.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s.0.get_direction(), RotationDirection::CounterClockwise);
        assert!(s.0.get_speed_from_attachment().is_ok());
    }

    #[test]
    fn test_linear_move_to(s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>)) {
        let destination = Distance::from_mm(10.0);
        let speed = Speed::from_mm_per_second(10.0);
        s.0.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = motion::linear_move_to(&mut s.0, destination, speed);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 10.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), 10.0);
        assert_eq!(s.0.get_direction(), RotationDirection::Clockwise);
        assert!(s.0.get_speed_from_attachment().is_ok());
        assert_eq!(
            s.0.get_speed_from_attachment().unwrap().to_mm_per_second(),
            9.999400035997839
        );
    }

    #[test]
    fn test_linear_move_to_negative_speed(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let destination = Distance::from_mm(-10.0);
        let speed = Speed::from_mm_per_second(-10.0);
        s.0.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = motion::linear_move_to(&mut s.0, destination, speed);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), -10.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), -10.0);
        assert_eq!(s.0.get_direction(), RotationDirection::CounterClockwise);
    }

    #[test]
    fn test_linear_move_to_2d(s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>)) {
        let destination = Vector2D::new(Distance::from_mm(-10.0), Distance::from_mm(-10.0));
        let speed = Speed::from_mm_per_second(-10.0);
        s.0.reset();
        s.1.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s.1.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = motion::linear_move_to_2d(&mut s.0, &mut s.1, destination, speed);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), -10.0);
        assert_eq!(s.1.get_steps(), -10.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), -10.0);
        assert_eq!(s.1.get_position().unwrap().to_mm(), -10.0);
        assert_eq!(s.0.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s.1.get_direction(), RotationDirection::CounterClockwise);
        assert!(s.0.get_speed_from_attachment().is_ok());
        assert_eq!(
            s.0.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.07734118446382
        );
        assert_eq!(
            s.1.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.07734118446382
        );
    }

    #[test]
    fn test_linear_move_to_2d_no_move(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let destination = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let speed = Speed::from_mm_per_second(-10.0);
        s.0.reset();
        s.1.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s.1.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = motion::linear_move_to_2d(&mut s.0, &mut s.1, destination, speed);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 0.0);
        assert_eq!(s.1.get_steps(), 0.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s.1.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s.0.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s.1.get_direction(), RotationDirection::CounterClockwise);
        assert!(s.0.get_speed_from_attachment().is_ok());
        assert!(s.1.get_speed_from_attachment().is_ok());
        assert_eq!(
            s.0.get_speed_from_attachment().unwrap().to_mm_per_second(),
            9.999400035997839
        );
        assert_eq!(
            s.1.get_speed_from_attachment().unwrap().to_mm_per_second(),
            0.0
        );
    }

    #[test]
    fn test_linear_move_to_2d_2(s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>)) {
        let destination = Vector2D::new(Distance::from_mm(-5.0), Distance::from_mm(5.0));
        let speed = Speed::from_mm_per_second(10.0);
        s.0.reset();
        s.1.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s.1.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = motion::linear_move_to_2d(&mut s.0, &mut s.1, destination, speed);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), -5.0);
        assert_eq!(s.1.get_steps(), 5.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), -5.0);
        assert_eq!(s.1.get_position().unwrap().to_mm(), 5.0);
        assert_eq!(s.0.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s.1.get_direction(), RotationDirection::Clockwise);
        assert_eq!(
            s.0.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.07734118446382
        );
        assert_eq!(
            s.1.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.07734118446382
        );
    }

    #[test]
    fn test_linear_move_to_2d_different_stepping_mode(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let destination = Vector2D::new(Distance::from_mm(-5.0), Distance::from_mm(5.0));
        let speed = Speed::from_mm_per_second(10.0);
        s.0.reset();
        s.1.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s.1.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s.0.set_stepping_mode(SteppingMode::HalfStep);
        s.1.set_stepping_mode(SteppingMode::QuarterStep);
        let res = motion::linear_move_to_2d(&mut s.0, &mut s.1, destination, speed);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), -5.0);
        assert_eq!(s.1.get_steps(), 5.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), -5.0);
        assert_eq!(s.1.get_position().unwrap().to_mm(), 5.0);
        assert_eq!(s.0.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s.1.get_direction(), RotationDirection::Clockwise);
        assert_eq!(
            s.0.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.07734118446382
        );
        assert_eq!(
            s.1.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.074337134610486
        );
    }

    #[test]
    fn test_linear_move_to_3d(s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>)) {
        let destination = Vector3D::new(
            Distance::from_mm(-5.0),
            Distance::from_mm(5.0),
            Distance::from_mm(5.0),
        );
        let speed = Speed::from_mm_per_second(10.0);
        s.0.reset();
        s.1.reset();
        s.2.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s.1.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s.2.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s.0.set_stepping_mode(SteppingMode::FullStep);
        s.1.set_stepping_mode(SteppingMode::FullStep);
        s.2.set_stepping_mode(SteppingMode::FullStep);
        let res = motion::linear_move_to_3d(&mut s.0, &mut s.1, &mut s.2, destination, speed);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), -5.0);
        assert_eq!(s.1.get_steps(), 5.0);
        assert_eq!(s.2.get_steps(), 5.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), -5.0);
        assert_eq!(s.1.get_position().unwrap().to_mm(), 5.0);
        assert_eq!(s.2.get_position().unwrap().to_mm(), 5.0);
        assert_eq!(s.0.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s.1.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s.2.get_direction(), RotationDirection::Clockwise);
        assert_eq!(
            s.0.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.07734118446382
        );
        assert_eq!(
            s.1.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.07734118446382
        );
        assert_eq!(
            s.2.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.07734118446382
        );
    }

    #[test]
    fn test_linear_move_to_3d_lower_distance_per_step(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let destination = Vector3D::new(
            Distance::from_mm(-5.0),
            Distance::from_mm(-2.0),
            Distance::from_mm(5.0),
        );
        let speed = Speed::from_mm_per_second(10.0);
        s.0.reset();
        s.1.reset();
        s.2.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(0.5),
        });
        s.1.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(0.5),
        });
        s.2.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(0.5),
        });
        s.0.set_stepping_mode(SteppingMode::FullStep);
        s.1.set_stepping_mode(SteppingMode::FullStep);
        s.2.set_stepping_mode(SteppingMode::FullStep);
        let res = motion::linear_move_to_3d(&mut s.0, &mut s.1, &mut s.2, destination, speed);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), -10.0);
        assert_eq!(s.1.get_steps(), -4.0);
        assert_eq!(s.2.get_steps(), 10.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), -5.0);
        assert_eq!(s.1.get_position().unwrap().to_mm(), -2.0);
        assert_eq!(s.2.get_position().unwrap().to_mm(), 5.0);
        assert_eq!(s.0.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s.1.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s.2.get_direction(), RotationDirection::Clockwise);
        assert_eq!(
            s.0.get_speed_from_attachment().unwrap().to_mm_per_second(),
            9.277470590418229
        );
        assert_eq!(
            s.1.get_speed_from_attachment().unwrap().to_mm_per_second(),
            3.725338260714073
        );
        assert_eq!(
            s.2.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.07734118446382
        );
    }

    #[test]
    fn test_linear_move_to_3d_no_move(
        s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>),
    ) {
        let destination = Vector3D::new(
            Distance::from_mm(0.0),
            Distance::from_mm(0.0),
            Distance::from_mm(0.0),
        );
        let speed = Speed::from_mm_per_second(10.0);
        s.0.reset();
        s.1.reset();
        s.2.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s.1.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s.2.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s.0.set_stepping_mode(SteppingMode::FullStep);
        s.1.set_stepping_mode(SteppingMode::FullStep);
        s.2.set_stepping_mode(SteppingMode::FullStep);
        let res = motion::linear_move_to_3d(&mut s.0, &mut s.1, &mut s.2, destination, speed);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 0.0);
        assert_eq!(s.1.get_steps(), 0.0);
        assert_eq!(s.2.get_steps(), 0.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s.1.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s.2.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s.0.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s.1.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s.2.get_direction(), RotationDirection::CounterClockwise);
    }
}
