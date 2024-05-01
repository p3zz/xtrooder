use core::time::Duration;

use crate::stepper::a4988::{Stepper, StepperError};
use embassy_stm32::timer::CaptureCompare16bitInstance;
use futures::join;
use heapless::Vec;
use math::angle::{asin, cos, sin};
use math::common::{abs, compute_arc_destination, sqrt, RotationDirection};
use math::computable::Computable;
use math::distance::Distance;
use math::speed::Speed;
use math::vector::{Vector2D, Vector3D};
use micromath::F32Ext;

#[derive(Clone, Copy)]
pub enum Positioning {
    Relative,
    Absolute,
}

// ---------------------------- LINEAR MOVE 1D ----------------------------

pub async fn linear_move_for<'s, S: CaptureCompare16bitInstance>(
    stepper: &mut Stepper<'s, S>,
    distance: Distance,
    speed: Speed,
) -> Result<(), StepperError> {
    let dest = stepper.get_position().add(&distance);
    linear_move_to(stepper, dest, speed).await
}

pub async fn linear_move_to<'s, S: CaptureCompare16bitInstance>(
    stepper: &mut Stepper<'s, S>,
    dest: Distance,
    speed: Speed,
) -> Result<(), StepperError> {
    let s = Speed::from_mm_per_second(abs(speed.to_mm_per_second()));
    stepper.set_speed(s);
    stepper.move_to(dest).await
}

// ---------------------------- LINEAR MOVE 2D ----------------------------

async fn linear_move_to_2d_raw<
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
        linear_move_to(stepper_b, dest.get_y(), speed.get_y()),
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
    let src = Vector2D::new(
        stepper_a.get_position(),
        stepper_b.get_position(),
    );
    let direction = dest.sub(&src).normalize();
    if direction.is_err() {
        return Err(StepperError::MoveNotValid);
    }
    let speed_x =
        Speed::from_mm_per_second(direction.unwrap().get_x() * speed.to_mm_per_second());
    let speed_y =
        Speed::from_mm_per_second(direction.unwrap().get_y() * speed.to_mm_per_second());

    let speed = Vector2D::new(speed_x, speed_y);

    linear_move_to_2d_raw(stepper_a, stepper_b, dest, speed).await
}


// ---------------------------- LINEAR MOVE 3D ----------------------------

pub async fn linear_move_3d<
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
    positioning: Positioning
) -> Result<(), StepperError> {
    match positioning{
        Positioning::Relative => linear_move_for_3d(stepper_a, stepper_b, stepper_c, dest, speed).await,
        Positioning::Absolute => linear_move_to_3d(stepper_a, stepper_b, stepper_c, dest, speed).await,
    }
}

async fn linear_move_to_3d_raw<
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
    let src = Vector3D::new(
        stepper_a.get_position(),
        stepper_b.get_position(),
        stepper_c.get_position(),
    );
    let direction = dest.sub(&src).normalize();
    if direction.is_err() {
        return Err(StepperError::MoveNotValid);
    }
    let speed_x =
        Speed::from_mm_per_second(direction.unwrap().get_x() * speed.to_mm_per_second());
    let speed_y =
        Speed::from_mm_per_second(direction.unwrap().get_y() * speed.to_mm_per_second());
    let speed_z =
        Speed::from_mm_per_second(direction.unwrap().get_z() * speed.to_mm_per_second());

    let speed = Vector3D::new(speed_x, speed_y, speed_z);

    linear_move_to_3d_raw(stepper_a, stepper_b, stepper_c, dest, speed).await
}

pub async fn linear_move_for_3d<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
    C: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    stepper_c: &mut Stepper<'s, C>,
    distance: Vector3D<Distance>,
    speed: Speed,
) -> Result<(), StepperError>{
    let source = Vector3D::new(stepper_a.get_position(), stepper_b.get_position(), stepper_c.get_position());
    let dest = source.add(&distance);
    linear_move_to_3d(stepper_a, stepper_b, stepper_c, dest, speed).await
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
    let src = Vector3D::new(
        stepper_a.get_position(),
        stepper_b.get_position(),
        stepper_c.get_position(),
    );
    let distance = dest.sub(&src);
    let time = distance.get_magnitude().to_mm() / speed.to_mm_per_second();

    let e_delta = e_dest.sub(&stepper_e.get_position());
    let e_speed = Speed::from_mm_per_second(e_delta.to_mm() / time);

    match join!(
        linear_move_to_3d(stepper_a, stepper_b, stepper_c, dest, speed),
        linear_move_to(stepper_e, e_dest, e_speed)
    ) {
        (Ok(_), Ok(_)) => Ok(()),
        _ => Err(StepperError::MoveNotValid),
    }
}

// ---------------------------- ARC MOVE 2D ----------------------------

pub async fn arc_move_2d_arc_length<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    arc_length: Distance,
    center: Vector2D<Distance>,
    speed: Speed,
    direction: RotationDirection,
) -> Result<(), StepperError> {
    let arc_unit_length = Distance::from_mm(1.0);
    if arc_length.to_mm() < arc_unit_length.to_mm() {
        return Err(StepperError::MoveTooShort);
    }
    let mut source = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let arcs_n = (arc_length.div(&arc_unit_length).unwrap() as f32).floor() as u64;
    for _ in 0..(arcs_n + 1) {
        let arc_dst = match compute_arc_destination(source, center, arc_unit_length, direction) {
            Some(dst) => dst,
            None => return Err(StepperError::MoveNotValid),
        };
        linear_move_to_2d(stepper_a, stepper_b, arc_dst, speed).await?;
        source = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    }
    Ok(())
}

pub async fn arc_move_3d_center<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
    C: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    stepper_c: &mut Stepper<'s, C>,
    dest: Vector3D<Distance>,
    center: Vector3D<Distance>,
    speed: Speed,
    direction: RotationDirection,
) -> Result<(), StepperError> {
    // TODO compute the minimum arc unit possible using the distance_per_step of each stepper
    let xy_dest = Vector2D::new(dest.get_x(), dest.get_y());
    let xy_center = Vector2D::new(center.get_x(), center.get_y());
    let xy_src = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let radius = xy_dest.sub(&xy_center).get_magnitude();
    let chord_length = xy_dest.sub(&xy_src).get_magnitude();
    if chord_length.to_mm() == 0f64 {
        return Err(StepperError::MoveNotValid);
    }
    let th: f64 = 2.0 * asin(chord_length.to_mm() / (2.0 * radius.to_mm())).to_radians();
    let arc_length = Distance::from_mm(radius.to_mm() * th);
    let time = arc_length.to_mm() / speed.to_mm_per_second();
    let z_delta = dest.get_z().sub(&stepper_c.get_position());
    let z_speed = Speed::from_mm_per_second(z_delta.to_mm() / time);
    match join!(
        arc_move_2d_arc_length(stepper_a, stepper_b, arc_length, xy_center, speed, direction),
        linear_move_to(stepper_c, dest.get_z(), z_speed)
    ){
        (Ok(_), Ok(_)) => todo!(),
        _ => todo!(),
    }
}

pub async fn arc_move_3d_radius<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
    C: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    stepper_c: &mut Stepper<'s, C>,
    dest: Vector3D<Distance>,
    radius: Distance,
    speed: Speed,
    direction: RotationDirection,
) -> Result<(), StepperError> {
    let source = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let angle = source.get_angle();
    let center_offset_x = Distance::from_mm(radius.to_mm() * cos(angle));
    let center_offset_y = Distance::from_mm(radius.to_mm() * sin(angle));
    let center = source.add(&Vector2D::new(center_offset_x, center_offset_y));
    let center = Vector3D::new(center.get_x(), center.get_y(), Distance::from_mm(0f64));
    arc_move_3d_center(stepper_a, stepper_b, stepper_c, dest, center, speed, direction).await
}
