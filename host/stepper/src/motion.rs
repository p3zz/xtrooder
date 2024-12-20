use core::time::Duration;

use futures::future::select;
use futures::{join, pin_mut};
use math::common::{abs, compute_arc_destination, compute_arc_length, RotationDirection};
use math::measurements::{AngularVelocity, Distance, Speed};
use math::vector::{Vector2D, Vector3D};

use crate::stepper::{Attached, AttachmentMode, Stepper, StepperError};

use common::{ExtiInputPinBase, OutputPinBase, TimerBase};

#[derive(Clone, Copy, PartialEq)]
pub enum Positioning {
    Relative,
    Absolute,
}

impl From<&str> for Positioning {
    fn from(value: &str) -> Self {
        match value {
            "relative" => Positioning::Relative,
            "absolute" => Positioning::Absolute,
            _ => panic!("Invalid positioning"),
        }
    }
}

pub fn no_move<P: OutputPinBase>(
    stepper: &Stepper<P, Attached>,
    positioning: Positioning,
) -> Distance {
    match positioning {
        Positioning::Relative => Distance::from_millimeters(0.0),
        Positioning::Absolute => stepper.get_position(),
    }
}

// ---------------------------- LINEAR MOVE 1D ----------------------------

pub async fn linear_move_to<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    stepper: &mut Stepper<P, Attached>,
    dest: Distance,
    speed: Speed,
    endstop: &mut Option<I>,
) -> Result<Duration, StepperError> {
    let s = Speed::from_meters_per_second(abs(speed.as_meters_per_second()));
    stepper.set_speed_from_attachment(s);
    let f1 = stepper.move_to_destination::<T>(dest);
    if let Some(endstop) = endstop {
        let f2 = endstop.wait_for_high();
        pin_mut!(f1, f2);
        match select(f1, f2).await {
            futures::future::Either::Left(r) => r.0,
            futures::future::Either::Right(_) => Err(StepperError::EndstopHit),
        }
    } else {
        f1.await
    }
}

// ---------------------------- LINEAR MOVE 2D ----------------------------

async fn linear_move_to_2d_raw<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    steppers: (&mut Stepper<P, Attached>, &mut Stepper<P, Attached>),
    dest: Vector2D<Distance>,
    speed: Vector2D<Speed>,
    endstops: (&mut Option<I>, &mut Option<I>),
) -> Result<Duration, StepperError> {
    match join!(
        linear_move_to::<P, T, I>(steppers.0, dest.get_x(), speed.get_x(), endstops.0),
        linear_move_to::<P, T, I>(steppers.1, dest.get_y(), speed.get_y(), endstops.1),
    ) {
        (Ok(da), Ok(db)) => {
            let max = da.max(db);
            Ok(max)
        }
        _ => Err(StepperError::MoveNotValid),
    }
}

fn linear_move_to_2d_inner<P: OutputPinBase>(
    steppers: (&mut Stepper<P, Attached>, &mut Stepper<P, Attached>),
    dest: Vector2D<Distance>,
    speed: Speed,
) -> Result<Vector2D<Speed>, StepperError> {
    let src = Vector2D::new(steppers.0.get_position(), steppers.1.get_position());
    let delta = (dest - src).normalize();
    // TODO what happens if the normalize vector is zero?
    let speed_x = delta.get_x() * speed;
    let speed_y = delta.get_y() * speed;

    Ok(Vector2D::new(speed_x, speed_y))
}

pub async fn linear_move_to_2d<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    steppers: (&mut Stepper<P, Attached>, &mut Stepper<P, Attached>),
    dest: Vector2D<Distance>,
    speed: Speed,
    endstops: (&mut Option<I>, &mut Option<I>),
) -> Result<Duration, StepperError> {
    let speed = linear_move_to_2d_inner((steppers.0, steppers.1), dest, speed)?;
    linear_move_to_2d_raw::<P, T, I>((steppers.0, steppers.1), dest, speed, endstops).await
}

// ---------------------------- LINEAR MOVE 3D ----------------------------

pub async fn linear_move_3d<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    steppers: (
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
    ),
    dest: Vector3D<Distance>,
    speed: Speed,
    positioning: Positioning,
    endstops: (&mut Option<I>, &mut Option<I>, &mut Option<I>),
) -> Result<Duration, StepperError> {
    match positioning {
        Positioning::Relative => {
            linear_move_for_3d::<P, T, I>(steppers, dest, speed, endstops).await
        }
        Positioning::Absolute => {
            linear_move_to_3d::<P, T, I>(steppers, dest, speed, endstops).await
        }
    }
}

async fn linear_move_to_3d_raw<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    steppers: (
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
    ),
    dest: Vector3D<Distance>,
    speed: Vector3D<Speed>,
    endstops: (&mut Option<I>, &mut Option<I>, &mut Option<I>),
) -> Result<Duration, StepperError> {
    match join!(
        linear_move_to::<P, T, I>(steppers.0, dest.get_x(), speed.get_x(), endstops.0),
        linear_move_to::<P, T, I>(steppers.1, dest.get_y(), speed.get_y(), endstops.1),
        linear_move_to::<P, T, I>(steppers.2, dest.get_z(), speed.get_z(), endstops.2),
    ) {
        (Ok(da), Ok(db), Ok(dc)) => {
            let max = da.max(db).max(dc);
            Ok(max)
        }
        _ => Err(StepperError::MoveNotValid),
    }
}

pub fn linear_move_to_3d_inner<P: OutputPinBase>(
    steppers: (
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
    ),
    dest: Vector3D<Distance>,
    speed: Speed,
) -> Result<Vector3D<Speed>, StepperError> {
    let src = Vector3D::new(
        steppers.0.get_position(),
        steppers.1.get_position(),
        steppers.2.get_position(),
    );
    let delta = (dest - src).normalize();
    // TODO what happens if the normalize vector is zero?
    let speed_x = delta.get_x() * speed;
    let speed_y = delta.get_y() * speed;
    let speed_z = delta.get_z() * speed;

    Ok(Vector3D::new(speed_x, speed_y, speed_z))
}

pub async fn linear_move_to_3d<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    steppers: (
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
    ),
    dest: Vector3D<Distance>,
    speed: Speed,
    endstops: (&mut Option<I>, &mut Option<I>, &mut Option<I>),
) -> Result<Duration, StepperError> {
    let speed = linear_move_to_3d_inner::<P>((steppers.0, steppers.1, steppers.2), dest, speed)?;
    linear_move_to_3d_raw::<P, T, I>((steppers.0, steppers.1, steppers.2), dest, speed, endstops)
        .await
}

pub async fn linear_move_for_3d<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    steppers: (
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
    ),
    distance: Vector3D<Distance>,
    speed: Speed,
    endstops: (&mut Option<I>, &mut Option<I>, &mut Option<I>),
) -> Result<Duration, StepperError> {
    let source = Vector3D::new(
        steppers.0.get_position(),
        steppers.1.get_position(),
        steppers.2.get_position(),
    );
    let dest = source + distance;
    linear_move_to_3d::<P, T, I>(steppers, dest, speed, endstops).await
}

pub async fn linear_move_3d_e<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    steppers: (
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
    ),
    dest: Vector3D<Distance>,
    speed: Speed,
    e_dest: Distance,
    positioning: Positioning,
    e_positioning: Positioning,
    endstops: (
        &mut Option<I>,
        &mut Option<I>,
        &mut Option<I>,
        &mut Option<I>,
    ),
) -> Result<Duration, StepperError> {
    match positioning {
        Positioning::Relative => {
            let e_dest = match e_positioning{
                Positioning::Relative => e_dest,
                Positioning::Absolute => e_dest - steppers.3.get_position(),
            };
            linear_move_for_3d_e::<P, T, I>(steppers, dest, speed, e_dest, endstops).await
        }
        Positioning::Absolute => {
            let e_dest = match e_positioning{
                Positioning::Absolute => e_dest,
                Positioning::Relative => e_dest + steppers.3.get_position(),
            };
            linear_move_to_3d_e::<P, T, I>(steppers, dest, speed, e_dest, endstops).await
        }
    }
}

pub async fn linear_move_to_3d_e<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    steppers: (
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
    ),
    dest: Vector3D<Distance>,
    speed: Speed,
    e_dest: Distance,
    endstops: (
        &mut Option<I>,
        &mut Option<I>,
        &mut Option<I>,
        &mut Option<I>,
    ),
) -> Result<Duration, StepperError> {
    let start = Vector3D::new(steppers.0.get_position(), steppers.1.get_position(), steppers.2.get_position());
    let distance = (dest - start).get_magnitude();
    // TODO check threshold value
    let e_speed = if distance.as_millimeters() < 1e-6 {
        speed
    }else{
        let duration = distance / speed;
        let e_speed = (e_dest - steppers.3.get_position()) / duration;
        e_speed
    };
    match join!(
        linear_move_to_3d::<P, T, I>(
            (steppers.0, steppers.1, steppers.2),
            dest,
            speed,
            (endstops.0, endstops.1, endstops.2)
        ),
        linear_move_to::<P, T, I>(steppers.3, e_dest, e_speed, endstops.3)
    ) {
        (Ok(dabc), Ok(de)) => {
            let max = dabc.max(de);
            Ok(max)
        }
        _ => Err(StepperError::MoveNotValid),
    }
}

pub async fn linear_move_for_3d_e<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    steppers: (
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
    ),
    distance: Vector3D<Distance>,
    speed: Speed,
    e_distance: Distance,
    endstops: (
        &mut Option<I>,
        &mut Option<I>,
        &mut Option<I>,
        &mut Option<I>,
    ),
) -> Result<Duration, StepperError> {
    let src = Vector3D::new(
        steppers.0.get_position(),
        steppers.1.get_position(),
        steppers.2.get_position(),
    );
    let abc_destination = src + distance;
    let e_destination = steppers.3.get_position() + e_distance;

    linear_move_to_3d_e::<P, T, I>(steppers, abc_destination, speed, e_destination, endstops).await
}

// ---------------------------- ARC MOVE 2D ----------------------------

pub async fn arc_move_2d_arc_length<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    steppers: (&mut Stepper<P, Attached>, &mut Stepper<P, Attached>),
    arc_length: Distance,
    center: Vector2D<Distance>,
    speed: Speed,
    direction: RotationDirection,
    arc_unit_length: Distance,
    endstops: (&mut Option<I>, &mut Option<I>),
) -> Result<Duration, StepperError> {
    if arc_length < arc_unit_length {
        return Err(StepperError::MoveTooShort);
    }
    let source = Vector2D::new(steppers.0.get_position(), steppers.1.get_position());
    let arcs_n = (arc_length / arc_unit_length) as u64;
    let mut total_duration = Duration::ZERO;
    for n in 0..(arcs_n + 1) {
        let arc_length = arc_unit_length * n as f64;
        let arc_dst = compute_arc_destination(source, center, arc_length, direction);
        total_duration += linear_move_to_2d::<P, T, I>(
            (steppers.0, steppers.1),
            arc_dst,
            speed,
            (endstops.0, endstops.1),
        )
        .await?;
    }
    Ok(total_duration)
}

pub async fn arc_move_3d_e_center<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    steppers: (
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
    ),
    dest: Vector3D<Distance>,
    center: Vector2D<Distance>,
    speed: Speed,
    direction: RotationDirection,
    e_dest: Distance,
    full_circle_enabled: bool,
    arc_unit_length: Distance,
    endstops: (
        &mut Option<I>,
        &mut Option<I>,
        &mut Option<I>,
        &mut Option<I>,
    ),
) -> Result<Duration, StepperError> {
    // TODO compute the minimum arc unit possible using the distance_per_step of each stepper
    let xy_dest = Vector2D::new(dest.get_x(), dest.get_y());
    let xy_center = Vector2D::new(center.get_x(), center.get_y());
    let xy_src = Vector2D::new(steppers.0.get_position(), steppers.1.get_position());

    let arc_length = compute_arc_length(xy_src, xy_center, xy_dest, direction, full_circle_enabled);

    let time = arc_length / speed;

    let z_delta = dest.get_z() - steppers.2.get_position();
    let z_speed = z_delta / time;

    let e_delta = e_dest - steppers.3.get_position();
    let e_speed = e_delta / time;

    match join!(
        arc_move_2d_arc_length::<P, T, I>(
            (steppers.0, steppers.1),
            arc_length,
            xy_center,
            speed,
            direction,
            arc_unit_length,
            (endstops.0, endstops.1)
        ),
        linear_move_to::<P, T, I>(steppers.2, dest.get_z(), z_speed, endstops.2),
        linear_move_to::<P, T, I>(steppers.3, e_dest, e_speed, endstops.3)
    ) {
        (Ok(dab), Ok(dc), Ok(de)) => {
            let max = dab.max(dc).max(de);
            Ok(max)
        }
        _ => Err(StepperError::MoveNotValid),
    }
}

pub async fn arc_move_3d_e_radius<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    steppers: (
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
    ),
    dest: Vector3D<Distance>,
    radius: Distance,
    speed: Speed,
    direction: RotationDirection,
    e_dest: Distance,
    arc_unit_length: Distance,
    endstops: (
        &mut Option<I>,
        &mut Option<I>,
        &mut Option<I>,
        &mut Option<I>,
    ),
) -> Result<Duration, StepperError> {
    let source = Vector2D::new(steppers.0.get_position(), steppers.1.get_position());
    let norm = source.normalize();
    // TODO what happens if the normalize vector is zero?
    let center_offset_x = radius * norm.get_x();
    let center_offset_y = radius * norm.get_y();
    let center = source + Vector2D::new(center_offset_x, center_offset_y);
    arc_move_3d_e_center::<P, T, I>(
        steppers,
        dest,
        center,
        speed,
        direction,
        e_dest,
        false,
        arc_unit_length,
        endstops,
    )
    .await
}

pub async fn arc_move_3d_e_offset_from_center<
    P: OutputPinBase,
    T: TimerBase,
    I: ExtiInputPinBase,
>(
    steppers: (
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
        &mut Stepper<P, Attached>,
    ),
    dest: Vector3D<Distance>,
    offset: Vector2D<Distance>,
    speed: Speed,
    direction: RotationDirection,
    e_dest: Distance,
    arc_unit_length: Distance,
    endstops: (
        &mut Option<I>,
        &mut Option<I>,
        &mut Option<I>,
        &mut Option<I>,
    ),
) -> Result<Duration, StepperError> {
    let source = Vector2D::new(steppers.0.get_position(), steppers.1.get_position());
    let center = source + offset;
    arc_move_3d_e_center::<P, T, I>(
        steppers,
        dest,
        center,
        speed,
        direction,
        e_dest,
        true,
        arc_unit_length,
        endstops,
    )
    .await
}

pub async fn calibrate<I: ExtiInputPinBase, O: OutputPinBase, T: TimerBase>(
    stepper: &mut Stepper<O, Attached>,
    trigger: &I,
) -> Result<Duration, StepperError> {
    // set the rotation direction to positive
    let mut duration = Duration::ZERO;
    let direction = RotationDirection::from(-i8::from(stepper.get_options().positive_direction));
    stepper.set_direction(direction);
    stepper.set_speed(AngularVelocity::from_rpm(90.0));
    let step_duration = stepper.get_step_duration();
    // calibrate x
    while !trigger.is_high() {
        stepper.step_unchecked();
        T::after(step_duration).await;
        duration += step_duration;
    }
    let bounds = stepper
        .get_options()
        .bounds
        .ok_or(StepperError::MoveNotValid)?;
    stepper.set_position(bounds.0);
    Ok(duration)
}

pub async fn auto_home<I: ExtiInputPinBase, O: OutputPinBase, T: TimerBase>(
    stepper: &mut Stepper<O, Attached>,
    trigger: &I,
) -> Result<Duration, StepperError> {
    // set the rotation direction to positive
    let mut duration = Duration::ZERO;
    let direction = stepper.get_options().positive_direction;
    stepper.set_direction(direction);
    stepper.set_speed(AngularVelocity::from_rpm(60.0));
    let step_duration = stepper.get_step_duration();
    // calibrate x
    while !trigger.is_high() {
        stepper.step_unchecked();
        T::after(step_duration).await;
        duration += step_duration;
    }
    let bounds = stepper
        .get_options()
        .bounds
        .ok_or(StepperError::MoveNotValid)?;
    // set the current steps to the positive bound so we can safely home performing the correct number of steps
    stepper.set_position(bounds.1);
    duration += stepper.home::<T>().await?;
    Ok(duration)
}

pub async fn retract<O: OutputPinBase, T: TimerBase, I: ExtiInputPinBase>(
    steppers: (&mut Stepper<O, Attached>, &mut Stepper<O, Attached>),
    e_speed: Speed,
    e_distance: Distance,
    z_distance: Distance,
    endstops: (&mut Option<I>, &mut Option<I>),
) -> Result<Duration, StepperError> {
    let e_destination = steppers.1.get_position() - e_distance;
    let z_destination = steppers.0.get_position() + z_distance;
    let e_time = e_distance / e_speed;
    let z_speed = z_distance / e_time;

    match join!(
        linear_move_to::<_, T, _>(steppers.1, e_destination, e_speed, endstops.1),
        linear_move_to::<_, T, _>(steppers.0, z_destination, z_speed, endstops.0)
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
    use approx::assert_abs_diff_eq;
    use tokio::time::sleep;

    use super::*;

    struct StatefulOutputPinMock {
        state: bool,
    }

    impl StatefulOutputPinMock {
        pub fn new() -> Self {
            Self { state: false }
        }
    }

    impl OutputPinBase for StatefulOutputPinMock {
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

    impl TimerBase for StepperTimer {
        fn after(duration: Duration) -> impl core::future::Future<Output = ()> {
            sleep(duration)
        }
    }

    struct InputPinMock {
        state: bool,
        delay: Duration,
    }

    impl InputPinMock {
        fn new(delay: Duration) -> Self {
            Self {
                state: false,
                delay,
            }
        }

        fn set_high(&mut self) {
            self.state = true;
        }
    }

    impl ExtiInputPinBase for InputPinMock {
        fn is_high(&self) -> bool {
            self.state
        }

        fn wait_for_high(&mut self) -> impl core::future::Future<Output = ()> {
            sleep(self.delay)
        }

        fn wait_for_low(&mut self) -> impl core::future::Future<Output = ()> {
            sleep(self.delay)
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
        let mut endstop = None;
        let res = linear_move_to::<StatefulOutputPinMock, StepperTimer, InputPinMock>(
            &mut s,
            destination,
            speed,
            &mut endstop,
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 0.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s.get_position().as_millimeters(), 0.0, epsilon = 0.000001);
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
        let mut endstop = None;
        let res = linear_move_to::<StatefulOutputPinMock, StepperTimer, InputPinMock>(
            &mut s,
            destination,
            speed,
            &mut endstop,
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), 10.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s.get_position().as_millimeters(), 10.0, epsilon = 0.000001);
        assert_eq!(s.get_direction(), RotationDirection::Clockwise);
        assert_abs_diff_eq!(
            s.get_speed_from_attachment().as_meters_per_second(),
            0.01,
            epsilon = 0.000001
        );
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
        let mut endstop = None;
        let res = linear_move_to::<StatefulOutputPinMock, StepperTimer, InputPinMock>(
            &mut s,
            destination,
            speed,
            &mut endstop,
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s.get_steps(), -10.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s.get_position().as_millimeters(), -10.0, epsilon = 0.000001);
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
        let destination = Vector2D::new(
            Distance::from_millimeters(-10.0),
            Distance::from_millimeters(-10.0),
        );
        let mut endstop_x = None;
        let mut endstop_y = None;
        let speed = Speed::from_meters_per_second(-0.01);
        let res = linear_move_to_2d::<StatefulOutputPinMock, StepperTimer, InputPinMock>(
            (&mut s_x, &mut s_y),
            destination,
            speed,
            (&mut endstop_x, &mut endstop_y),
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), -10.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s_y.get_steps(), -10.0, epsilon = 0.000001);
        assert_abs_diff_eq!(
            s_x.get_position().as_millimeters(),
            -10.0,
            epsilon = 0.000001
        );
        assert_abs_diff_eq!(
            s_y.get_position().as_millimeters(),
            -10.0,
            epsilon = 0.000001
        );
        assert_eq!(s_x.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::CounterClockwise);
        assert_abs_diff_eq!(
            0.00703610931,
            s_x.get_speed_from_attachment().as_meters_per_second(),
            epsilon = 0.00001
        );
        assert_abs_diff_eq!(
            0.00703610931,
            s_y.get_speed_from_attachment().as_meters_per_second(),
            epsilon = 0.00001
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
        let mut endstop_x = None;
        let mut endstop_y = None;
        let destination = Vector2D::new(
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
        );
        let speed = Speed::from_meters_per_second(-0.01);
        let res = linear_move_to_2d::<StatefulOutputPinMock, StepperTimer, InputPinMock>(
            (&mut s_x, &mut s_y),
            destination,
            speed,
            (&mut endstop_x, &mut endstop_y),
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), 0.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s_y.get_steps(), 0.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s_x.get_position().as_millimeters(), 0.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s_y.get_position().as_millimeters(), 0.0, epsilon = 0.000001);
        assert_eq!(s_x.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert_abs_diff_eq!(
            s_x.get_speed_from_attachment().as_meters_per_second(),
            0.0,
            epsilon = 0.000001
        );
        assert_abs_diff_eq!(
            s_y.get_speed_from_attachment().as_meters_per_second(),
            0.0,
            epsilon = 0.000001
        );
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
        let mut endstop_x = None;
        let mut endstop_y = None;
        let destination = Vector2D::new(
            Distance::from_millimeters(-5.0),
            Distance::from_millimeters(5.0),
        );
        let speed = Speed::from_meters_per_second(0.01);
        let res = linear_move_to_2d::<StatefulOutputPinMock, StepperTimer, InputPinMock>(
            (&mut s_x, &mut s_y),
            destination,
            speed,
            (&mut endstop_x, &mut endstop_y),
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), -5.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s_y.get_steps(), 5.0, epsilon = 0.000001);
        assert_abs_diff_eq!(
            s_x.get_position().as_millimeters(),
            -5.0,
            epsilon = 0.000001
        );
        assert_abs_diff_eq!(s_y.get_position().as_millimeters(), 5.0, epsilon = 0.000001);
        assert_eq!(s_x.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert_abs_diff_eq!(
            0.0070361093,
            s_x.get_speed_from_attachment().as_meters_per_second(),
            epsilon = 0.00001
        );
        assert_abs_diff_eq!(
            0.0070361093,
            s_y.get_speed_from_attachment().as_meters_per_second(),
            epsilon = 0.00001
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
        let destination = Vector2D::new(
            Distance::from_millimeters(-5.0),
            Distance::from_millimeters(5.0),
        );
        let speed = Speed::from_meters_per_second(0.01);
        let mut endstop_x = None;
        let mut endstop_y = None;
        s_x.set_stepping_mode(SteppingMode::HalfStep);
        s_y.set_stepping_mode(SteppingMode::QuarterStep);
        let res = linear_move_to_2d::<StatefulOutputPinMock, StepperTimer, InputPinMock>(
            (&mut s_x, &mut s_y),
            destination,
            speed,
            (&mut endstop_x, &mut endstop_y),
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), -5.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s_y.get_steps(), 5.0, epsilon = 0.000001);
        assert_abs_diff_eq!(
            s_x.get_position().as_millimeters(),
            -5.0,
            epsilon = 0.000001
        );
        assert_abs_diff_eq!(s_y.get_position().as_millimeters(), 5.0, epsilon = 0.000001);
        assert_eq!(s_x.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert_abs_diff_eq!(
            0.00703610,
            s_x.get_speed_from_attachment().as_meters_per_second(),
            epsilon = 0.00001
        );
        assert_abs_diff_eq!(
            0.00703610,
            s_y.get_speed_from_attachment().as_meters_per_second(),
            epsilon = 0.00001
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
        let mut endstop_x = None;
        let mut endstop_y = None;
        let mut endstop_z = None;
        let destination = Vector3D::new(
            Distance::from_millimeters(-5.0),
            Distance::from_millimeters(5.0),
            Distance::from_millimeters(5.0),
        );
        let speed = Speed::from_meters_per_second(0.01);
        s_x.set_stepping_mode(SteppingMode::FullStep);
        s_y.set_stepping_mode(SteppingMode::FullStep);
        s_z.set_stepping_mode(SteppingMode::FullStep);
        let res = linear_move_to_3d::<StatefulOutputPinMock, StepperTimer, InputPinMock>(
            (&mut s_x, &mut s_y, &mut s_z),
            destination,
            speed,
            (&mut endstop_x, &mut endstop_y, &mut endstop_z),
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
            0.00574300,
            s_x.get_speed_from_attachment().as_meters_per_second(),
            epsilon = 0.00001
        );
        assert_abs_diff_eq!(
            0.00574300,
            s_y.get_speed_from_attachment().as_meters_per_second(),
            epsilon = 0.00001
        );
        assert_abs_diff_eq!(
            0.00574300,
            s_z.get_speed_from_attachment().as_meters_per_second(),
            epsilon = 0.00001
        );
    }

    #[tokio::test]
    async fn test_linear_move_to_3d_e() {
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
        let mut s_e = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default(),
        );
        let mut endstop_x = None;
        let mut endstop_y = None;
        let mut endstop_z = None;
        let mut endstop_e = None;
        let destination = Vector3D::new(
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
        );
        let e_destination = Distance::from_millimeters(3.0);
        let speed = Speed::from_meters_per_second(0.01);
        s_x.set_stepping_mode(SteppingMode::FullStep);
        s_y.set_stepping_mode(SteppingMode::FullStep);
        s_z.set_stepping_mode(SteppingMode::FullStep);
        s_e.set_stepping_mode(SteppingMode::FullStep);
        let res = linear_move_to_3d_e::<StatefulOutputPinMock, StepperTimer, InputPinMock>(
            (&mut s_x, &mut s_y, &mut s_z, &mut s_e),
            destination,
            speed,
            e_destination,
            (&mut endstop_x, &mut endstop_y, &mut endstop_z, &mut endstop_e),
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), 0.0);
        assert_abs_diff_eq!(s_y.get_steps(), 0.0);
        assert_abs_diff_eq!(s_z.get_steps(), 0.0);
        assert_abs_diff_eq!(s_e.get_steps(), 3.0);
        assert_abs_diff_eq!(s_x.get_position().as_millimeters(), 0.0);
        assert_abs_diff_eq!(s_y.get_position().as_millimeters(), 0.0);
        assert_abs_diff_eq!(s_z.get_position().as_millimeters(), 0.0);
        assert_abs_diff_eq!(s_e.get_position().as_millimeters(), 3.0);
        assert_eq!(s_x.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_z.get_direction(), RotationDirection::Clockwise);
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
        let mut endstop_x = None;
        let mut endstop_y = None;
        let mut endstop_z = None;
        let speed = Speed::from_meters_per_second(0.01);
        s_x.set_stepping_mode(SteppingMode::FullStep);
        s_y.set_stepping_mode(SteppingMode::FullStep);
        s_z.set_stepping_mode(SteppingMode::FullStep);
        let res = linear_move_to_3d::<StatefulOutputPinMock, StepperTimer, InputPinMock>(
            (&mut s_x, &mut s_y, &mut s_z),
            destination,
            speed,
            (&mut endstop_x, &mut endstop_y, &mut endstop_z),
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
            0.00679144,
            epsilon = 0.00001
        );
        assert_abs_diff_eq!(
            s_y.get_speed_from_attachment().as_meters_per_second(),
            0.00271656,
            epsilon = 0.0001
        );
        assert_abs_diff_eq!(
            s_z.get_speed_from_attachment().as_meters_per_second(),
            0.006791448,
            epsilon = 0.0001
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
        let mut endstop_x = None;
        let mut endstop_y = None;
        let mut endstop_z = None;
        let destination = Vector3D::new(
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
            Distance::from_millimeters(0.0),
        );
        let speed = Speed::from_meters_per_second(0.01);
        s_x.set_stepping_mode(SteppingMode::FullStep);
        s_y.set_stepping_mode(SteppingMode::FullStep);
        s_z.set_stepping_mode(SteppingMode::FullStep);
        let res = linear_move_to_3d::<StatefulOutputPinMock, StepperTimer, InputPinMock>(
            (&mut s_x, &mut s_y, &mut s_z),
            destination,
            speed,
            (&mut endstop_x, &mut endstop_y, &mut endstop_z),
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), 0.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s_y.get_steps(), 0.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s_z.get_steps(), 0.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s_x.get_position().as_millimeters(), 0.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s_y.get_position().as_millimeters(), 0.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s_z.get_position().as_millimeters(), 0.0, epsilon = 0.000001);
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
        let mut endstop_x = None;
        let mut endstop_y = None;
        let arc_length = Distance::from_millimeters(20.0);
        let center = Vector2D::new(
            Distance::from_millimeters(10.0),
            Distance::from_millimeters(10.0),
        );
        let speed = Speed::from_meters_per_second(0.01);
        let direction = RotationDirection::Clockwise;
        let res = arc_move_2d_arc_length::<StatefulOutputPinMock, StepperTimer, InputPinMock>(
            (&mut s_x, &mut s_y),
            arc_length,
            center,
            speed,
            direction,
            Distance::from_millimeters(1.0),
            (&mut endstop_x, &mut endstop_y),
        )
        .await;
        assert!(res.is_ok());
        assert_abs_diff_eq!(s_x.get_steps(), -2.0, epsilon = 0.000001);
        assert_abs_diff_eq!(s_y.get_steps(), 18.0, epsilon = 0.000001);
        assert_abs_diff_eq!(
            s_x.get_position().as_millimeters(),
            -2.0,
            epsilon = 0.000001
        );
        assert_abs_diff_eq!(
            s_y.get_position().as_millimeters(),
            18.0,
            epsilon = 0.000001
        );
        assert_eq!(s_x.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
    }

    #[tokio::test]
    async fn test_auto_home_failure() {
        let mut stepper = Stepper::new_with_attachment(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            StepperAttachment::default()
        );
        let mut trigger: InputPinMock = InputPinMock::new(Duration::from_millis(0));
        trigger.set_high();

        let result = auto_home::<InputPinMock, StatefulOutputPinMock, StepperTimer>(
            &mut stepper,
            &trigger,
        )
        .await;
        assert!(result.is_err());
        assert_eq!(StepperError::MoveNotValid, result.err().unwrap());
    }

    // FIXME
    // #[tokio::test]
    // async fn test_auto_home_success() {
    //     let mut stepper = Stepper::new(
    //         StatefulOutputPinMock::new(),
    //         StatefulOutputPinMock::new(),
    //         StepperOptions {
    //             steps_per_revolution: 100,
    //             stepping_mode: SteppingMode::FullStep,
    //             bounds: Some((-100.0, 100.0)),
    //             positive_direction: RotationDirection::Clockwise,
    //         },
    //     );
    //     let mut trigger: InputPinMock = InputPinMock::new(Duration::from_millis(0));
    //     // simulate collision with the trigger switch
    //     trigger.set_high();

    //     let result = auto_home::<InputPinMock, StatefulOutputPinMock, StepperTimer, NotAttached>(
    //         &mut stepper,
    //         &trigger,
    //     )
    //     .await;
    //     assert!(result.is_ok());
    //     assert_eq!(1, result.unwrap().as_secs());
    // }
}
