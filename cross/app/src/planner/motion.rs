use crate::stepper::a4988::{Stepper, StepperError};
use futures::join;
use math::angle::{cos, sin};
use math::common::{abs, compute_arc_destination, compute_arc_length, RotationDirection};
use math::computable::Computable;
use math::distance::Distance;
use math::speed::Speed;
use math::common::max;
use math::vector::{Vector2D, Vector3D};
use micromath::F32Ext;
use embassy_time::Duration;
use heapless::Vec;

#[derive(Clone, Copy)]
pub enum Positioning {
    Relative,
    Absolute,
}

pub fn no_move(
    stepper: &Stepper,
    positioning: Positioning,
) -> Result<Distance, StepperError> {
    match positioning {
        Positioning::Relative => Ok(Distance::from_mm(0.0)),
        Positioning::Absolute => stepper.get_position(),
    }
}

// ---------------------------- LINEAR MOVE 1D ----------------------------

#[cfg(not(test))]
pub async fn linear_move_to<'s>(
    stepper: &mut Stepper<'s>,
    dest: Distance,
    speed: Speed,
) -> Result<(), StepperError> {
    let s = Speed::from_mm_per_second(abs(speed.to_mm_per_second()));
    stepper.set_speed_from_attachment(s)?;
    stepper.move_to_destination(dest).await
}

#[cfg(test)]
pub fn linear_move_to<'s>(
    stepper: &mut Stepper<'s>,
    dest: Distance,
    speed: Speed,
) -> Result<(), StepperError> {
    let s = Speed::from_mm_per_second(abs(speed.to_mm_per_second()));
    stepper.set_speed_from_attachment(s)?;
    stepper.move_to_destination(dest)
}

// ---------------------------- LINEAR MOVE 2D ----------------------------

#[cfg(not(test))]
async fn linear_move_to_2d_raw<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
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

#[cfg(test)]
fn linear_move_to_2d_raw<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    dest: Vector2D<Distance>,
    speed: Vector2D<Speed>,
) -> Result<(), StepperError> {
    linear_move_to(stepper_a, dest.get_x(), speed.get_x())?;
    linear_move_to(stepper_b, dest.get_y(), speed.get_y())?;
    Ok(())
}


fn linear_move_to_2d_inner<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    dest: Vector2D<Distance>,
    speed: Speed,
) -> Result<Vector2D<Speed>, StepperError> {
    let src = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);
    let direction = dest.sub(&src).normalize();
    if direction.is_err() {
        return Err(StepperError::MoveNotValid);
    }
    let speed_x = Speed::from_mm_per_second(direction.unwrap().get_x() * speed.to_mm_per_second());
    let speed_y = Speed::from_mm_per_second(direction.unwrap().get_y() * speed.to_mm_per_second());

    Ok(Vector2D::new(speed_x, speed_y))
}

#[cfg(test)]
pub fn linear_move_to_2d<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    dest: Vector2D<Distance>,
    speed: Speed,
) -> Result<(), StepperError> {
    let speed = linear_move_to_2d_inner(stepper_a, stepper_b, dest, speed)?;
    linear_move_to_2d_raw(stepper_a, stepper_b, dest, speed)
}

#[cfg(not(test))]
pub async fn linear_move_to_2d<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    dest: Vector2D<Distance>,
    speed: Speed,
) -> Result<(), StepperError> {
    let speed = linear_move_to_2d_inner(stepper_a, stepper_b, dest, speed)?;
    linear_move_to_2d_raw(stepper_a, stepper_b, dest, speed).await
}

// ---------------------------- LINEAR MOVE 3D ----------------------------

#[cfg(not(test))]
pub async fn linear_move_3d<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
    dest: Vector3D<Distance>,
    speed: Speed,
    positioning: Positioning,
) -> Result<(), StepperError> {
    match positioning {
        Positioning::Relative => {
            linear_move_for_3d(stepper_a, stepper_b, stepper_c, dest, speed).await
        }
        Positioning::Absolute => {
            linear_move_to_3d(stepper_a, stepper_b, stepper_c, dest, speed).await
        }
    }
}

#[cfg(not(test))]
async fn linear_move_to_3d_raw<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
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

#[cfg(not(test))]
pub async fn linear_move_to_3d<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
    dest: Vector3D<Distance>,
    speed: Speed,
) -> Result<(), StepperError> {
    let src = Vector3D::new(
        stepper_a.get_position()?,
        stepper_b.get_position()?,
        stepper_c.get_position()?,
    );
    let direction = dest.sub(&src).normalize();
    if direction.is_err() {
        return Err(StepperError::MoveNotValid);
    }
    let speed_x = Speed::from_mm_per_second(direction.unwrap().get_x() * speed.to_mm_per_second());
    let speed_y = Speed::from_mm_per_second(direction.unwrap().get_y() * speed.to_mm_per_second());
    let speed_z = Speed::from_mm_per_second(direction.unwrap().get_z() * speed.to_mm_per_second());

    let speed = Vector3D::new(speed_x, speed_y, speed_z);

    linear_move_to_3d_raw(stepper_a, stepper_b, stepper_c, dest, speed).await
}

#[cfg(not(test))]
pub async fn linear_move_for_3d<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
    distance: Vector3D<Distance>,
    speed: Speed,
) -> Result<(), StepperError> {
    let source = Vector3D::new(
        stepper_a.get_position()?,
        stepper_b.get_position()?,
        stepper_c.get_position()?,
    );
    let dest = source.add(&distance);
    linear_move_to_3d(stepper_a, stepper_b, stepper_c, dest, speed).await
}

#[cfg(not(test))]
pub async fn linear_move_3d_e<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
    stepper_e: &mut Stepper<'s>,
    dest: Vector3D<Distance>,
    speed: Speed,
    e_dest: Distance,
    positioning: Positioning,
) -> Result<(), StepperError> {
    match positioning {
        Positioning::Relative => {
            linear_move_for_3d_e(
                stepper_a, stepper_b, stepper_c, stepper_e, dest, speed, e_dest,
            )
            .await
        }
        Positioning::Absolute => {
            linear_move_to_3d_e(
                stepper_a, stepper_b, stepper_c, stepper_e, dest, speed, e_dest,
            )
            .await
        }
    }
}

#[cfg(not(test))]
pub async fn linear_move_to_3d_e<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
    stepper_e: &mut Stepper<'s>,
    dest: Vector3D<Distance>,
    speed: Speed,
    e_dest: Distance,
) -> Result<(), StepperError> {
    let src = Vector3D::new(
        stepper_a.get_position()?,
        stepper_b.get_position()?,
        stepper_c.get_position()?,
    );
    let distance = dest.sub(&src);
    let time = distance.get_magnitude().to_mm() / speed.to_mm_per_second();

    let e_delta = e_dest.sub(&stepper_e.get_position()?);
    let e_speed = Speed::from_mm_per_second(e_delta.to_mm() / time);

    match join!(
        linear_move_to_3d(stepper_a, stepper_b, stepper_c, dest, speed),
        linear_move_to(stepper_e, e_dest, e_speed)
    ) {
        (Ok(_), Ok(_)) => Ok(()),
        _ => Err(StepperError::MoveNotValid),
    }
}

#[cfg(not(test))]
pub async fn linear_move_for_3d_e<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
    stepper_e: &mut Stepper<'s>,
    distance: Vector3D<Distance>,
    speed: Speed,
    e_distance: Distance,
) -> Result<(), StepperError> {
    let src = Vector3D::new(
        stepper_a.get_position()?,
        stepper_b.get_position()?,
        stepper_c.get_position()?,
    );
    let abc_destination = src.add(&distance);
    let e_destination = stepper_e.get_position()?.add(&e_distance);

    linear_move_to_3d_e(
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

#[cfg(not(test))]
pub async fn arc_move_2d_arc_length<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    arc_length: Distance,
    center: Vector2D<Distance>,
    speed: Speed,
    direction: RotationDirection,
) -> Result<(), StepperError> {
    let arc_unit_length = Distance::from_mm(1.0);
    if arc_length.to_mm() < arc_unit_length.to_mm() {
        return Err(StepperError::MoveTooShort);
    }
    let mut source = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);
    let arcs_n = (arc_length.div(&arc_unit_length).unwrap() as f32).floor() as u64;
    for _ in 0..(arcs_n + 1) {
        let arc_dst = match compute_arc_destination(source, center, arc_unit_length, direction) {
            Some(dst) => dst,
            None => return Err(StepperError::MoveNotValid),
        };
        linear_move_to_2d(stepper_a, stepper_b, arc_dst, speed).await?;
        source = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);
    }
    Ok(())
}

#[cfg(not(test))]
pub async fn arc_move_3d_e_center<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
    stepper_e: &mut Stepper<'s>,
    dest: Vector3D<Distance>,
    center: Vector2D<Distance>,
    speed: Speed,
    direction: RotationDirection,
    e_dest: Distance,
    full_circle_enabled: bool,
) -> Result<(), StepperError> {
    // TODO compute the minimum arc unit possible using the distance_per_step of each stepper
    let xy_dest = Vector2D::new(dest.get_x(), dest.get_y());
    let xy_center = Vector2D::new(center.get_x(), center.get_y());
    let xy_src = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);

    let arc_length = compute_arc_length(xy_src, xy_center, xy_dest, direction, full_circle_enabled);

    let time = arc_length.to_mm() / speed.to_mm_per_second();

    let z_delta = dest.get_z().sub(&stepper_c.get_position()?);
    let z_speed = Speed::from_mm_per_second(z_delta.to_mm() / time);

    let e_delta = e_dest.sub(&stepper_e.get_position()?);
    let e_speed = Speed::from_mm_per_second(e_delta.to_mm() / time);

    match join!(
        arc_move_2d_arc_length(stepper_a, stepper_b, arc_length, xy_center, speed, direction),
        linear_move_to(stepper_c, dest.get_z(), z_speed),
        linear_move_to(stepper_e, e_dest, e_speed)
    ) {
        (Ok(_), Ok(_), Ok(_)) => Ok(()),
        _ => Err(StepperError::MoveNotValid),
    }
}

#[cfg(not(test))]
pub async fn arc_move_3d_e_radius<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
    stepper_e: &mut Stepper<'s>,
    dest: Vector3D<Distance>,
    radius: Distance,
    speed: Speed,
    direction: RotationDirection,
    e_dest: Distance,
) -> Result<(), StepperError> {
    let source = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);
    let angle = source.get_angle();
    let center_offset_x = Distance::from_mm(radius.to_mm() * cos(angle));
    let center_offset_y = Distance::from_mm(radius.to_mm() * sin(angle));
    let center = source.add(&Vector2D::new(center_offset_x, center_offset_y));
    arc_move_3d_e_center(
        stepper_a, stepper_b, stepper_c, stepper_e, dest, center, speed, direction, e_dest, false,
    )
    .await
}

#[cfg(not(test))]
pub async fn arc_move_3d_e_offset_from_center<
    's,
>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
    stepper_e: &mut Stepper<'s>,
    dest: Vector3D<Distance>,
    offset: Vector2D<Distance>,
    speed: Speed,
    direction: RotationDirection,
    e_dest: Distance,
) -> Result<(), StepperError> {
    let source = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);
    let center = source.add(&offset);
    arc_move_3d_e_center(
        stepper_a, stepper_b, stepper_c, stepper_e, dest, center, speed, direction, e_dest, true,
    )
    .await
}
