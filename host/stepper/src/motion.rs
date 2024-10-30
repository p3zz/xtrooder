use core::ops::AddAssign;
use core::time::Duration;

use futures::join;
use math::angle::{cos, sin};
use math::common::{
    abs, compute_arc_destination, compute_arc_length, floor, max, RotationDirection,
};
use math::measurements::{AngularVelocity, Distance, Speed};
use math::vector::{Vector2D, Vector3D};

use crate::stepper::{
    Attached, AttachmentMode, StatefulOutputPin, Stepper, StepperError, StepperInputPin, TimerTrait,
};

#[derive(Clone, Copy)]
pub enum Positioning {
    Relative,
    Absolute,
}

pub fn no_move<P: StatefulOutputPin>(
    stepper: &Stepper<P, Attached>,
    positioning: Positioning,
) -> Distance {
    match positioning {
        Positioning::Relative => Distance::from_millimeters(0.0),
        Positioning::Absolute => stepper.get_position(),
    }
}

// ---------------------------- LINEAR MOVE 1D ----------------------------

pub async fn linear_move_to<P: StatefulOutputPin, T: TimerTrait>(
    stepper: &mut Stepper<P, Attached>,
    dest: Distance,
    speed: Speed,
) -> Result<Duration, StepperError> {
    let s = Speed::from_meters_per_second(abs(speed.as_meters_per_second()));
    stepper.set_speed_from_attachment(s);
    stepper.move_to_destination::<T>(dest).await
}

// ---------------------------- LINEAR MOVE 2D ----------------------------

async fn linear_move_to_2d_raw<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    dest: Vector2D<Distance>,
    speed: Vector2D<Speed>,
) -> Result<Duration, StepperError> {
    match join!(
        linear_move_to::<P, T>(stepper_a, dest.get_x(), speed.get_x()),
        linear_move_to::<P, T>(stepper_b, dest.get_y(), speed.get_y()),
    ) {
        (Ok(da), Ok(db)) => {
            let max = da.max(db);
            Ok(max)
        }
        _ => Err(StepperError::MoveNotValid),
    }
}

fn linear_move_to_2d_inner<P: StatefulOutputPin>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    dest: Vector2D<Distance>,
    speed: Speed,
) -> Result<Vector2D<Speed>, StepperError> {
    let src = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let angle = (dest - src).get_angle();
    let speed_x = cos(angle) * speed;
    let speed_y = sin(angle) * speed;

    Ok(Vector2D::new(speed_x, speed_y))
}

pub async fn linear_move_to_2d<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    dest: Vector2D<Distance>,
    speed: Speed,
) -> Result<Duration, StepperError> {
    let speed = linear_move_to_2d_inner(stepper_a, stepper_b, dest, speed)?;
    linear_move_to_2d_raw::<P, T>(stepper_a, stepper_b, dest, speed).await
}

// ---------------------------- LINEAR MOVE 3D ----------------------------

pub async fn linear_move_3d<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    stepper_c: &mut Stepper<P, Attached>,
    dest: Vector3D<Distance>,
    speed: Speed,
    positioning: Positioning,
) -> Result<Duration, StepperError> {
    match positioning {
        Positioning::Relative => {
            linear_move_for_3d::<P, T>(stepper_a, stepper_b, stepper_c, dest, speed).await
        }
        Positioning::Absolute => {
            linear_move_to_3d::<P, T>(stepper_a, stepper_b, stepper_c, dest, speed).await
        }
    }
}

async fn linear_move_to_3d_raw<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    stepper_c: &mut Stepper<P, Attached>,
    dest: Vector3D<Distance>,
    speed: Vector3D<Speed>,
) -> Result<Duration, StepperError> {
    match join!(
        linear_move_to::<P, T>(stepper_a, dest.get_x(), speed.get_x()),
        linear_move_to::<P, T>(stepper_b, dest.get_y(), speed.get_y()),
        linear_move_to::<P, T>(stepper_c, dest.get_z(), speed.get_z()),
    ) {
        (Ok(da), Ok(db), Ok(dc)) => {
            let max = da.max(db).max(dc);
            Ok(max)
        }
        _ => Err(StepperError::MoveNotValid),
    }
}

pub fn linear_move_to_3d_inner<P: StatefulOutputPin>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    stepper_c: &mut Stepper<P, Attached>,
    dest: Vector3D<Distance>,
    speed: Speed,
) -> Result<Vector3D<Speed>, StepperError> {
    let src = Vector3D::new(
        stepper_a.get_position(),
        stepper_b.get_position(),
        stepper_c.get_position(),
    );
    let delta = dest - src;
    let xy_angle = Vector2D::new(delta.get_x(), delta.get_y()).get_angle();
    let xz_angle = Vector2D::new(delta.get_x(), delta.get_z()).get_angle();
    let speed_x = cos(xy_angle) * speed;
    let speed_y = sin(xy_angle) * speed;
    let speed_z = sin(xz_angle) * speed;

    Ok(Vector3D::new(speed_x, speed_y, speed_z))
}

pub async fn linear_move_to_3d<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    stepper_c: &mut Stepper<P, Attached>,
    dest: Vector3D<Distance>,
    speed: Speed,
) -> Result<Duration, StepperError> {
    let speed = linear_move_to_3d_inner::<P>(stepper_a, stepper_b, stepper_c, dest, speed)?;
    linear_move_to_3d_raw::<P, T>(stepper_a, stepper_b, stepper_c, dest, speed).await
}

pub async fn linear_move_for_3d<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    stepper_c: &mut Stepper<P, Attached>,
    distance: Vector3D<Distance>,
    speed: Speed,
) -> Result<Duration, StepperError> {
    let source = Vector3D::new(
        stepper_a.get_position(),
        stepper_b.get_position(),
        stepper_c.get_position(),
    );
    let dest = source + distance;
    linear_move_to_3d::<P, T>(stepper_a, stepper_b, stepper_c, dest, speed).await
}

pub async fn linear_move_3d_e<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    stepper_c: &mut Stepper<P, Attached>,
    stepper_e: &mut Stepper<P, Attached>,
    dest: Vector3D<Distance>,
    speed: Speed,
    e_dest: Distance,
    positioning: Positioning,
) -> Result<Duration, StepperError> {
    match positioning {
        Positioning::Relative => {
            linear_move_for_3d_e::<P, T>(
                stepper_a, stepper_b, stepper_c, stepper_e, dest, speed, e_dest,
            )
            .await
        }
        Positioning::Absolute => {
            linear_move_to_3d_e::<P, T>(
                stepper_a, stepper_b, stepper_c, stepper_e, dest, speed, e_dest,
            )
            .await
        }
    }
}

pub async fn linear_move_to_3d_e<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    stepper_c: &mut Stepper<P, Attached>,
    stepper_e: &mut Stepper<P, Attached>,
    dest: Vector3D<Distance>,
    speed: Speed,
    e_dest: Distance,
) -> Result<Duration, StepperError> {
    let src = Vector3D::new(
        stepper_a.get_position(),
        stepper_b.get_position(),
        stepper_c.get_position(),
    );
    let distance = dest - src;
    let time = distance.get_magnitude() / speed;

    let e_delta = e_dest - stepper_e.get_position();
    let e_speed = e_delta / time;

    match join!(
        linear_move_to_3d::<P, T>(stepper_a, stepper_b, stepper_c, dest, speed),
        linear_move_to::<P, T>(stepper_e, e_dest, e_speed)
    ) {
        (Ok(dabc), Ok(de)) => {
            let max = dabc.max(de);
            Ok(max)
        }
        _ => Err(StepperError::MoveNotValid),
    }
}

pub async fn linear_move_for_3d_e<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    stepper_c: &mut Stepper<P, Attached>,
    stepper_e: &mut Stepper<P, Attached>,
    distance: Vector3D<Distance>,
    speed: Speed,
    e_distance: Distance,
) -> Result<Duration, StepperError> {
    let src = Vector3D::new(
        stepper_a.get_position(),
        stepper_b.get_position(),
        stepper_c.get_position(),
    );
    let abc_destination = src + distance;
    let e_destination = stepper_e.get_position() + e_distance;

    linear_move_to_3d_e::<P, T>(
        stepper_a,
        stepper_b,
        stepper_c,
        stepper_e,
        abc_destination,
        speed,
        e_destination,
    )
    .await
}

// ---------------------------- ARC MOVE 2D ----------------------------

pub async fn arc_move_2d_arc_length<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    arc_length: Distance,
    center: Vector2D<Distance>,
    speed: Speed,
    direction: RotationDirection,
) -> Result<Duration, StepperError> {
    let arc_unit_length = Distance::from_millimeters(1.0);
    if arc_length < arc_unit_length {
        return Err(StepperError::MoveTooShort);
    }
    let source = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let arcs_n = floor(arc_length / arc_unit_length) as u64;
    let mut total_duration = Duration::ZERO;
    for n in 0..(arcs_n + 1) {
        let arc_length = arc_unit_length * n as f64;
        let arc_dst = compute_arc_destination(source, center, arc_length, direction);
        total_duration += linear_move_to_2d::<P, T>(stepper_a, stepper_b, arc_dst, speed).await?;
    }
    Ok(total_duration)
}

pub async fn arc_move_3d_e_center<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    stepper_c: &mut Stepper<P, Attached>,
    stepper_e: &mut Stepper<P, Attached>,
    dest: Vector3D<Distance>,
    center: Vector2D<Distance>,
    speed: Speed,
    direction: RotationDirection,
    e_dest: Distance,
    full_circle_enabled: bool,
) -> Result<Duration, StepperError> {
    // TODO compute the minimum arc unit possible using the distance_per_step of each stepper
    let xy_dest = Vector2D::new(dest.get_x(), dest.get_y());
    let xy_center = Vector2D::new(center.get_x(), center.get_y());
    let xy_src = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());

    let arc_length = compute_arc_length(xy_src, xy_center, xy_dest, direction, full_circle_enabled);

    let time = arc_length / speed;

    let z_delta = dest.get_z() - stepper_c.get_position();
    let z_speed = z_delta / time;

    let e_delta = e_dest - stepper_e.get_position();
    let e_speed = e_delta / time;

    match join!(
        arc_move_2d_arc_length::<P, T>(
            stepper_a, stepper_b, arc_length, xy_center, speed, direction
        ),
        linear_move_to::<P, T>(stepper_c, dest.get_z(), z_speed),
        linear_move_to::<P, T>(stepper_e, e_dest, e_speed)
    ) {
        (Ok(dab), Ok(dc), Ok(de)) => {
            let max = dab.max(dc).max(de);
            Ok(max)
        }
        _ => Err(StepperError::MoveNotValid),
    }
}

pub async fn arc_move_3d_e_radius<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    stepper_c: &mut Stepper<P, Attached>,
    stepper_e: &mut Stepper<P, Attached>,
    dest: Vector3D<Distance>,
    radius: Distance,
    speed: Speed,
    direction: RotationDirection,
    e_dest: Distance,
) -> Result<Duration, StepperError> {
    let source = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let angle = source.get_angle();
    let center_offset_x = radius * cos(angle);
    let center_offset_y = radius * sin(angle);
    let center = source + Vector2D::new(center_offset_x, center_offset_y);
    arc_move_3d_e_center::<P, T>(
        stepper_a, stepper_b, stepper_c, stepper_e, dest, center, speed, direction, e_dest, false,
    )
    .await
}

pub async fn arc_move_3d_e_offset_from_center<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P, Attached>,
    stepper_b: &mut Stepper<P, Attached>,
    stepper_c: &mut Stepper<P, Attached>,
    stepper_e: &mut Stepper<P, Attached>,
    dest: Vector3D<Distance>,
    offset: Vector2D<Distance>,
    speed: Speed,
    direction: RotationDirection,
    e_dest: Distance,
) -> Result<Duration, StepperError> {
    let source = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let center = source + offset;
    arc_move_3d_e_center::<P, T>(
        stepper_a, stepper_b, stepper_c, stepper_e, dest, center, speed, direction, e_dest, true,
    )
    .await
}

pub async fn auto_home<
    I: StepperInputPin,
    O: StatefulOutputPin,
    T: TimerTrait,
    M: AttachmentMode,
>(
    stepper: &mut Stepper<O, M>,
    trigger: &I,
) -> Result<Duration, StepperError> {
    // set the rotation direction to positive
    let direction = stepper.get_options().positive_direction;
    stepper.set_direction(direction);
    stepper.set_speed(AngularVelocity::from_rpm(60.0));

    // calibrate x
    while !trigger.is_high() {
        stepper.step().unwrap();
        T::after(stepper.get_step_duration()).await;
    }
    let bounds = stepper
        .get_options()
        .bounds
        .ok_or(StepperError::MoveOutOfBounds)?;
    // set the current steps to the positive bound so we can safely home performing the correct number of steps
    stepper.set_steps(bounds.1);
    stepper.home::<T>().await
}

// perform one calibration at a time
pub async fn auto_home_3d<
    I: StepperInputPin,
    O: StatefulOutputPin,
    T: TimerTrait,
    M: AttachmentMode,
>(
    stepper_a: &mut Stepper<O, M>,
    stepper_b: &mut Stepper<O, M>,
    stepper_c: &mut Stepper<O, M>,
    trigger_a: &I,
    trigger_b: &I,
    trigger_c: &I,
) -> Result<Duration, StepperError> {
    let mut duration = Duration::from_millis(0);
    duration.add_assign(auto_home::<I, O, T, M>(stepper_a, trigger_a).await?);
    duration.add_assign(auto_home::<I, O, T, M>(stepper_b, trigger_b).await?);
    duration.add_assign(auto_home::<I, O, T, M>(stepper_c, trigger_c).await?);
    Ok(duration)
}

pub async fn retract<O: StatefulOutputPin, T: TimerTrait>(
    e_stepper: &mut Stepper<O, Attached>,
    z_stepper: &mut Stepper<O, Attached>,
    e_speed: Speed,
    e_distance: Distance,
    z_distance: Distance,
) -> Result<Duration, StepperError> {
    let e_destination = e_stepper.get_position() - e_distance;
    let z_destination = z_stepper.get_position() + z_distance;
    let e_time = e_distance / e_speed;
    let z_speed = z_distance / e_time;

    match join!(
        linear_move_to::<O, T>(e_stepper, e_destination, e_speed),
        linear_move_to::<O, T>(z_stepper, z_destination, z_speed)
    ) {
        (Ok(da), Ok(db)) => {
            let duration = da.max(db);
            Ok(duration)
        }
        _ => Err(StepperError::MoveNotValid),
    }
}

#[cfg(test)]
mod tests {

    use math::{
        common::RotationDirection,
        measurements::{Distance, Speed},
        vector::{Vector2D, Vector3D},
    };

    use crate::stepper::{NotAttached, StepperAttachment, StepperOptions, SteppingMode};
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

    struct InputPinMock {
        state: bool,
    }

    impl InputPinMock {
        fn new() -> Self {
            Self { state: false }
        }

        fn set_high(&mut self) {
            self.state = true;
        }
        fn set_low(&mut self) {
            self.state = false;
        }
    }

    impl StepperInputPin for InputPinMock {
        fn is_high(&self) -> bool {
            self.state
        }
    }

    #[tokio::test]
    async fn test_linear_move_to_no_move() {
        let mut s = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let destination = Distance::from_millimeters(0.0);
        let speed = Speed::from_meters_per_second(0.01);
        let res =
            linear_move_to::<StatefulOutputPinMock, StepperTimer>(&mut s, destination, speed).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 0.0, epsilon=0.000001);
        assert_abs_diff_eq!(s.get_position().as_millimeters(), 0.0, epsilon=0.000001);
        assert_eq!(s.get_direction(), RotationDirection::Clockwise);
    }

    #[tokio::test]
    async fn test_linear_move_to() {
        let mut s = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let destination = Distance::from_millimeters(10.0);
        let speed = Speed::from_meters_per_second(0.01);
        let res =
            linear_move_to::<StatefulOutputPinMock, StepperTimer>(&mut s, destination, speed).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 10.0, epsilon=0.000001);
        assert_abs_diff_eq!(s.get_position().as_millimeters(), 10.0, epsilon=0.000001);
        assert_eq!(s.get_direction(), RotationDirection::Clockwise);
        assert_abs_diff_eq!(s.get_speed_from_attachment().as_meters_per_second(), 0.01, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_linear_move_to_negative_speed() {
        let mut s = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let destination = Distance::from_millimeters(-10.0);
        let speed = Speed::from_meters_per_second(0.01);
        let res =
            linear_move_to::<StatefulOutputPinMock, StepperTimer>(&mut s, destination, speed).await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), -10.0, epsilon=0.000001);
        assert_abs_diff_eq!(s.get_position().as_millimeters(), -10.0, epsilon=0.000001);
        assert_eq!(s.get_direction(), RotationDirection::CounterClockwise);
    }

    #[tokio::test]
    async fn test_linear_move_to_2d() {
        let mut s_x = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let mut s_y = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let destination = Vector2D::new(Distance::from_millimeters(-10.0), Distance::from_millimeters(-10.0));
        let speed = Speed::from_meters_per_second(-0.01);
        let res = linear_move_to_2d::<StatefulOutputPinMock, StepperTimer>(
            &mut s_x,
            &mut s_y,
            destination,
            speed,
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), -10.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_y.get_steps(), -10.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_x.get_position().as_millimeters(), -10.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_y.get_position().as_millimeters(), -10.0, epsilon=0.000001);
        assert_eq!(s_x.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::CounterClockwise);
        assert_abs_diff_eq!(
            s_x.get_speed_from_attachment().as_meters_per_second(),
            0.00707814269,
            epsilon=0.000001
        );
        assert_abs_diff_eq!(
            s_y.get_speed_from_attachment().as_meters_per_second(),
            0.00707814269,
            epsilon=0.000001
        );
    }

    #[tokio::test]
    async fn test_linear_move_to_2d_no_move() {
        let mut s_x = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let mut s_y = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let destination = Vector2D::new(Distance::from_millimeters(0.0), Distance::from_millimeters(0.0));
        let speed = Speed::from_meters_per_second(-0.01);
        let res = linear_move_to_2d::<StatefulOutputPinMock, StepperTimer>(
            &mut s_x,
            &mut s_y,
            destination,
            speed,
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), 0.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_y.get_steps(), 0.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_x.get_position().as_millimeters(), 0.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_y.get_position().as_millimeters(), 0.0, epsilon=0.000001);
        assert_eq!(s_x.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert_abs_diff_eq!(s_x.get_speed_from_attachment().as_meters_per_second(), 0.01, epsilon=0.000001);
        assert_abs_diff_eq!(s_y.get_speed_from_attachment().as_meters_per_second(), 0.0, epsilon=0.000001);
    }

    #[tokio::test]
    async fn test_linear_move_to_2d_2() {
        let mut s_x = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let mut s_y = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let destination = Vector2D::new(Distance::from_millimeters(-5.0), Distance::from_millimeters(5.0));
        let speed = Speed::from_meters_per_second(0.01);
        let res = linear_move_to_2d::<StatefulOutputPinMock, StepperTimer>(
            &mut s_x,
            &mut s_y,
            destination,
            speed,
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), -5.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_y.get_steps(), 5.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_x.get_position().as_millimeters(), -5.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_y.get_position().as_millimeters(), 5.0, epsilon=0.000001);
        assert_eq!(s_x.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert_abs_diff_eq!(
            s_x.get_speed_from_attachment().as_meters_per_second(),
            0.0070781426,
            epsilon=0.000001
        );
        assert_abs_diff_eq!(
            s_y.get_speed_from_attachment().as_meters_per_second(),
            0.0070781426,
            epsilon=0.000001
        );
    }

    #[tokio::test]
    async fn test_linear_move_to_2d_different_stepping_mode() {
        let mut s_x = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let mut s_y = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let destination = Vector2D::new(Distance::from_millimeters(-5.0), Distance::from_millimeters(5.0));
        let speed = Speed::from_meters_per_second(0.01);
        s_x.set_stepping_mode(SteppingMode::HalfStep);
        s_y.set_stepping_mode(SteppingMode::QuarterStep);
        let res = linear_move_to_2d::<StatefulOutputPinMock, StepperTimer>(
            &mut s_x,
            &mut s_y,
            destination,
            speed,
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), -5.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_y.get_steps(), 5.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_x.get_position().as_millimeters(), -5.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_y.get_position().as_millimeters(), 5.0, epsilon=0.000001);
        assert_eq!(s_x.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert_abs_diff_eq!(
            s_x.get_speed_from_attachment().as_meters_per_second(),
            0.00707814,
            epsilon=0.000001
        );
        assert_abs_diff_eq!(
            s_y.get_speed_from_attachment().as_meters_per_second(),
            0.00707814,
            epsilon=0.000001
        );
    }

    #[tokio::test]
    async fn test_linear_move_to_3d() {
        let mut s_x = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let mut s_y = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let mut s_z = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let destination = Vector3D::new(
            Distance::from_millimeters(-5.0),
            Distance::from_millimeters(5.0),
            Distance::from_millimeters(5.0),
        );
        let speed = Speed::from_meters_per_second(0.01);
        s_x.set_stepping_mode(SteppingMode::FullStep);
        s_y.set_stepping_mode(SteppingMode::FullStep);
        s_z.set_stepping_mode(SteppingMode::FullStep);
        let res = linear_move_to_3d::<StatefulOutputPinMock, StepperTimer>(
            &mut s_x,
            &mut s_y,
            &mut s_z,
            destination,
            speed,
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), -5.0);
        assert_abs_diff_eq!(s_y.get_steps(), 5.0);
        assert_abs_diff_eq!(s_z.get_steps(), 5.0);
        assert_abs_diff_eq!(s_x.get_position().as_millimeters(), -5.0);
        assert_abs_diff_eq!(s_y.get_position().as_millimeters(), 5.0);
        assert_abs_diff_eq!(s_z.get_position().as_millimeters(), 5.0);
        assert_eq!(s_x.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_z.get_direction(), RotationDirection::Clockwise);
        assert_abs_diff_eq!(
            s_x.get_speed_from_attachment().as_meters_per_second(),
            0.00707814,
            epsilon=0.000001
        );
        assert_abs_diff_eq!(
            s_y.get_speed_from_attachment().as_meters_per_second(),
            0.00707814,
            epsilon=0.000001
        );
        assert_abs_diff_eq!(
            s_z.get_speed_from_attachment().as_meters_per_second(),
            0.00707814,
            epsilon=0.000001
        );
    }

    #[tokio::test]
    async fn test_linear_move_to_3d_lower_distance_per_step() {
        let attachment = StepperAttachment {
            distance_per_step: Distance::from_millimeters(0.5),
        };

        let mut s_x = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            attachment,
        );
        let mut s_y = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            attachment,
        );
        let mut s_z = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            attachment,
        );
        let destination = Vector3D::new(
            Distance::from_millimeters(-5.0),
            Distance::from_millimeters(-2.0),
            Distance::from_millimeters(5.0),
        );
        let speed = Speed::from_meters_per_second(0.01);
        s_x.set_stepping_mode(SteppingMode::FullStep);
        s_y.set_stepping_mode(SteppingMode::FullStep);
        s_z.set_stepping_mode(SteppingMode::FullStep);
        let res = linear_move_to_3d::<StatefulOutputPinMock, StepperTimer>(
            &mut s_x,
            &mut s_y,
            &mut s_z,
            destination,
            speed,
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), -10.0);
        assert_abs_diff_eq!(s_y.get_steps(), -4.0);
        assert_abs_diff_eq!(s_z.get_steps(), 10.0);
        assert_abs_diff_eq!(s_x.get_position().as_millimeters(), -5.0);
        assert_abs_diff_eq!(s_y.get_position().as_millimeters(), -2.0);
        assert_abs_diff_eq!(s_z.get_position().as_millimeters(), 5.0);
        assert_eq!(s_x.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_z.get_direction(), RotationDirection::Clockwise);
        assert_abs_diff_eq!(
            s_x.get_speed_from_attachment().as_meters_per_second(),
            0.00928212,
            epsilon=0.000001
        );
        assert_abs_diff_eq!(
            s_y.get_speed_from_attachment().as_meters_per_second(),
            0.00372533,
            epsilon=0.000001
        );
        assert_abs_diff_eq!(
            s_z.get_speed_from_attachment().as_meters_per_second(),
            0.00707814,
            epsilon=0.000001
        );
    }

    #[tokio::test]
    async fn test_linear_move_to_3d_no_move() {
        let mut s_x = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let mut s_y = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let mut s_z = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let destination = Vector3D::new(
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
        );
        let speed = Speed::from_meters_per_second(0.01);
        s_x.set_stepping_mode(SteppingMode::FullStep);
        s_y.set_stepping_mode(SteppingMode::FullStep);
        s_z.set_stepping_mode(SteppingMode::FullStep);
        let res = linear_move_to_3d::<StatefulOutputPinMock, StepperTimer>(
            &mut s_x,
            &mut s_y,
            &mut s_z,
            destination,
            speed,
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), 0.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_y.get_steps(), 0.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_z.get_steps(), 0.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_x.get_position().as_millimeters(), 0.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_y.get_position().as_millimeters(), 0.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_z.get_position().as_millimeters(), 0.0, epsilon=0.000001);
        assert_eq!(s_x.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_z.get_direction(), RotationDirection::Clockwise);
    }

    #[tokio::test]
    async fn test_arc_move_2d_arc_length() {
        let mut s_x = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let mut s_y = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let arc_length = Distance::from_millimeters(20.0);
        let center = Vector2D::new(Distance::from_millimeters(10.0), Distance::from_millimeters(10.0));
        let speed = Speed::from_meters_per_second(0.01);
        let direction = RotationDirection::Clockwise;
        let res = arc_move_2d_arc_length::<StatefulOutputPinMock, StepperTimer>(
            &mut s_x, &mut s_y, arc_length, center, speed, direction,
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), -2.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_y.get_steps(), 18.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_x.get_position().as_millimeters(), -2.0, epsilon=0.000001);
        assert_abs_diff_eq!(s_y.get_position().as_millimeters(), 18.0, epsilon=0.000001);
        assert_eq!(s_x.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
    }

    #[tokio::test]
    async fn test_auto_home_failure() {
        let mut stepper = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
        );
        let mut trigger: InputPinMock = InputPinMock::new();
        trigger.set_high();

        let result = auto_home::<InputPinMock, StatefulOutputPinMock, StepperTimer, NotAttached>(
            &mut stepper,
            &trigger,
        )
        .await;
        assert!(result.is_err());
        assert_eq!(StepperError::MoveOutOfBounds, result.err().unwrap());
    }

    #[tokio::test]
    async fn test_auto_home_success() {
        let mut stepper = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions {
                steps_per_revolution: 100,
                stepping_mode: SteppingMode::FullStep,
                bounds: Some((-100.0, 100.0)),
                positive_direction: RotationDirection::Clockwise,
            },
        );
        let mut trigger: InputPinMock = InputPinMock::new();
        // simulate collision with the trigger switch
        trigger.set_high();

        let result = auto_home::<InputPinMock, StatefulOutputPinMock, StepperTimer, NotAttached>(
            &mut stepper,
            &trigger,
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(1, result.unwrap().as_secs());
    }
}
