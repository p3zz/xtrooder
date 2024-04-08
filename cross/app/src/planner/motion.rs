use crate::stepper::a4988::{Stepper, StepperError};
use embassy_stm32::timer::CaptureCompare16bitInstance;
use futures::join;
use math::common::abs;
use math::computable::Computable;
use math::distance::Distance;
use math::speed::Speed;
use math::vector::{Vector2D, Vector3D};

pub async fn linear_move_to<'s, S: CaptureCompare16bitInstance>(
    stepper: &mut Stepper<'s, S>,
    dest: Distance,
    speed: Speed,
) -> Result<(), StepperError> {
    let s = Speed::from_mm_per_second(abs(speed.to_mm_per_second()));
    stepper.set_speed(s);
    stepper.move_to(dest).await
}

pub async fn linear_move_to_e<
    's,
    A: CaptureCompare16bitInstance,
    E: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_e: &mut Stepper<'s, E>,
    a_dest: Distance,
    e_dest: Distance,
    a_speed: Speed,
) -> Result<(), StepperError> {
    // compute the time the stepper a takes to go from its position to the destination, at the given speed, then compute
    // the speed for the extruder stepper
    let a_distance = a_dest.sub(&stepper_a.get_position());
    let a_time = abs(a_distance.to_mm() / a_speed.to_mm_per_second());

    let e_distance = e_dest.sub(&stepper_e.get_position());
    let e_speed = Speed::from_mm_per_second(e_distance.to_mm() / a_time);

    match join!(
        linear_move_to(stepper_a, a_dest, a_speed),
        linear_move_to(stepper_e, e_dest, e_speed)
    ) {
        (Ok(_), Ok(_)) => Ok(()),
        _ => Err(StepperError::MoveNotValid),
    }
}

pub async fn linear_move_to_2d<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    dest: Vector2D<Distance>,
    speed: Speed,
) -> Result<(), StepperError> {
    let src = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let direction = dest.sub(&src).normalize();
    if direction.is_err() {
        return Err(StepperError::MoveNotValid);
    }
    let ab_speed_x =
        Speed::from_mm_per_second(direction.unwrap().get_x() * speed.to_mm_per_second());
    let ab_speed_y =
        Speed::from_mm_per_second(direction.unwrap().get_y() * speed.to_mm_per_second());
    let ab_speed = Vector2D::new(ab_speed_x, ab_speed_y);

    linear_move_to_2d_raw(stepper_a, stepper_b, dest, ab_speed).await
}

pub async fn linear_move_to_2d_raw<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    dest: Vector2D<Distance>,
    speed: Vector2D<Speed>,
) -> Result<(), StepperError> {
    match join!(
        linear_move_to(stepper_a, dest.get_x(), speed.get_x()),
        linear_move_to(stepper_b, dest.get_y(), speed.get_y())
    ) {
        (Ok(_), Ok(_)) => Ok(()),
        _ => Err(StepperError::MoveNotValid),
    }
}

pub async fn linear_move_to_3d<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
    C: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    stepper_c: &mut Stepper<'s, C>,
    dest: Vector3D<Distance>,
    speed: Speed,
) -> Result<(), StepperError> {
    let src = Vector3D::new(stepper_a.get_position(), stepper_b.get_position(), stepper_c.get_position());
    let direction = dest.sub(&src).normalize();
    if direction.is_err() {
        return Err(StepperError::MoveNotValid);
    }
    let ab_speed_x =
        Speed::from_mm_per_second(direction.unwrap().get_x() * speed.to_mm_per_second());
    let ab_speed_y =
        Speed::from_mm_per_second(direction.unwrap().get_y() * speed.to_mm_per_second());
    let ab_speed_z =
        Speed::from_mm_per_second(direction.unwrap().get_z() * speed.to_mm_per_second());
    
    let ab_speed = Vector3D::new(ab_speed_x, ab_speed_y, ab_speed_z);

    linear_move_to_3d_raw(stepper_a, stepper_b, stepper_c, dest, ab_speed).await
}

pub async fn linear_move_to_3d_e<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
    C: CaptureCompare16bitInstance,
    E: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    stepper_c: &mut Stepper<'s, C>,
    stepper_e: &mut Stepper<'s, E>,
    dest: Vector3D<Distance>,
    speed: Speed,
    e_dest: Distance,
) -> Result<(), StepperError> {
    let ab_src = Vector3D::new(stepper_a.get_position(), stepper_b.get_position(), stepper_c.get_position());
    let ab_distance = dest.sub(&ab_src);
    let ab_time = ab_distance.get_magnitude().to_mm() / speed.to_mm_per_second();
    
    let e_delta = e_dest.sub(&stepper_e.get_position());
    let e_speed = Speed::from_mm_per_second(e_delta.to_mm() / ab_time);

    match join!(
        linear_move_to_3d(stepper_a, stepper_b, stepper_c, dest, speed),
        linear_move_to(stepper_e, e_dest, e_speed)
    ){
        (Ok(_), Ok(_)) => Ok(()),
        _ => Err(StepperError::MoveNotValid),
    }
}


pub async fn linear_move_to_3d_raw<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
    C: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    stepper_c: &mut Stepper<'s, C>,
    dest: Vector3D<Distance>,
    speed: Vector3D<Speed>,
) -> Result<(), StepperError> {
    match join!(
        linear_move_to(stepper_a, dest.get_x(), speed.get_x()),
        linear_move_to(stepper_b, dest.get_y(), speed.get_y()),
        linear_move_to(stepper_c, dest.get_z(), speed.get_z()),
    ) {
        (Ok(_), Ok(_), Ok(_)) => Ok(()),
        _ => Err(StepperError::MoveNotValid),
    }
}

pub async fn linear_move_to_2d_e<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
    E: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    stepper_e: &mut Stepper<'s, E>,
    ab_dest: Vector2D<Distance>,
    e_dest: Distance,
    ab_speed: Speed,
) -> Result<(), StepperError> {
    let ab_source = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let ab_distance = ab_dest.sub(&ab_source);
    let ab_time = ab_distance.get_magnitude().to_mm() / ab_speed.to_mm_per_second();
    let e_delta = e_dest.sub(&stepper_e.get_position());
    let e_speed = Speed::from_mm_per_second(e_delta.to_mm() / ab_time);
    match join!(
        linear_move_to_2d(stepper_a, stepper_b, ab_dest, ab_speed),
        linear_move_to(stepper_e, e_dest, e_speed)
    ) {
        (Ok(_), Ok(_)) => Ok(()),
        _ => Err(StepperError::MoveNotValid),
    }
}

pub async fn linear_move_for<'s, S: CaptureCompare16bitInstance>(
    stepper: &mut Stepper<'s, S>,
    distance: Distance,
    speed: Speed,
) -> Result<(), StepperError> {
    let dest = stepper.get_position().add(&distance);
    linear_move_to(stepper, dest, speed).await
}

pub async fn linear_move_for_e<
    's,
    A: CaptureCompare16bitInstance,
    E: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_e: &mut Stepper<'s, E>,
    a_distance: Distance,
    e_distance: Distance,
    feedrate: Speed,
) -> Result<(), StepperError> {
    let a_dest = stepper_a.get_position().add(&a_distance);
    let e_dest = stepper_e.get_position().add(&e_distance);
    linear_move_to_e(stepper_a, stepper_e, a_dest, e_dest, feedrate).await
}

pub async fn linear_move_for_2d<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    distance: Vector2D<Distance>,
    speed: Vector2D<Speed>,
) -> Result<(), StepperError> {
    let source = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let dest = source.add(&distance);
    linear_move_to_2d_raw(stepper_a, stepper_b, dest, speed).await
}

pub async fn linear_move_for_2d_e<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
    E: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    stepper_e: &mut Stepper<'s, E>,
    ab_distance: Vector2D<Distance>,
    e_distance: Distance,
    ab_speed: Speed,
) -> Result<(), StepperError> {
    let ab_source = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let ab_dest = ab_source.add(&ab_distance);
    let e_dest = stepper_e.get_position().add(&e_distance);
    linear_move_to_2d_e(stepper_a, stepper_b, stepper_e, ab_dest, e_dest, ab_speed).await
}
