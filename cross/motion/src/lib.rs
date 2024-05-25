#![no_std]
#![no_main]

use core::f64::consts::PI;

use defmt::info;
use embassy_time::Duration;
use futures::join;
use math::angle::{cos, sin, Angle};
use math::common::max;
use math::common::{abs, compute_arc_destination, compute_arc_length, RotationDirection};
use math::computable::Computable;
use math::distance::Distance;
use math::speed::Speed;
use math::vector::{Vector2D, Vector3D};
use micromath::F32Ext;
use stepper::{Stepper, StepperError};

#[derive(Clone, Copy)]
pub enum Positioning {
    Relative,
    Absolute,
}

pub fn no_move(stepper: &Stepper, positioning: Positioning) -> Result<Distance, StepperError> {
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
async fn linear_move_to_2d_raw<'s>(
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
fn linear_move_to_2d_raw<'s>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    dest: Vector2D<Distance>,
    speed: Vector2D<Speed>,
) -> Result<(), StepperError> {
    linear_move_to(stepper_a, dest.get_x(), speed.get_x())?;
    linear_move_to(stepper_b, dest.get_y(), speed.get_y())?;
    Ok(())
}

fn linear_move_to_2d_inner<'s>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    dest: Vector2D<Distance>,
    speed: Speed,
) -> Result<Vector2D<Speed>, StepperError> {
    let src = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);
    let angle = dest.sub(&src).get_angle();
    let speed_x = Speed::from_mm_per_second(cos(angle) * speed.to_mm_per_second());
    let speed_y = Speed::from_mm_per_second(sin(angle) * speed.to_mm_per_second());

    Ok(Vector2D::new(speed_x, speed_y))
}

#[cfg(test)]
pub fn linear_move_to_2d<'s>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    dest: Vector2D<Distance>,
    speed: Speed,
) -> Result<(), StepperError> {
    let speed = linear_move_to_2d_inner(stepper_a, stepper_b, dest, speed)?;
    linear_move_to_2d_raw(stepper_a, stepper_b, dest, speed)
}

#[cfg(not(test))]
pub async fn linear_move_to_2d<'s>(
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
pub async fn linear_move_3d<'s>(
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
async fn linear_move_to_3d_raw<'s>(
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

#[cfg(test)]
fn linear_move_to_3d_raw<'s>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
    dest: Vector3D<Distance>,
    speed: Vector3D<Speed>,
) -> Result<(), StepperError> {
    linear_move_to(stepper_a, dest.get_x(), speed.get_x())?;
    linear_move_to(stepper_b, dest.get_y(), speed.get_y())?;
    linear_move_to(stepper_c, dest.get_z(), speed.get_z())?;
    Ok(())
}

pub fn linear_move_to_3d_inner<'s>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
    dest: Vector3D<Distance>,
    speed: Speed,
) -> Result<Vector3D<Speed>, StepperError> {
    let src = Vector3D::new(
        stepper_a.get_position()?,
        stepper_b.get_position()?,
        stepper_c.get_position()?,
    );
    let delta = dest.sub(&src);
    let xy_angle = Vector2D::new(delta.get_x(), delta.get_y()).get_angle();
    let xz_angle = Vector2D::new(delta.get_x(), delta.get_z()).get_angle();
    let speed_x = Speed::from_mm_per_second(cos(xy_angle) * speed.to_mm_per_second());
    let speed_y = Speed::from_mm_per_second(sin(xy_angle) * speed.to_mm_per_second());
    let speed_z = Speed::from_mm_per_second(sin(xz_angle) * speed.to_mm_per_second());

    Ok(Vector3D::new(speed_x, speed_y, speed_z))
}

#[cfg(not(test))]
pub async fn linear_move_to_3d<'s>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
    dest: Vector3D<Distance>,
    speed: Speed,
) -> Result<(), StepperError> {
    let speed = linear_move_to_3d_inner(stepper_a, stepper_b, stepper_c, dest, speed)?;
    linear_move_to_3d_raw(stepper_a, stepper_b, stepper_c, dest, speed).await
}

#[cfg(test)]
pub fn linear_move_to_3d<'s>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    stepper_c: &mut Stepper<'s>,
    dest: Vector3D<Distance>,
    speed: Speed,
) -> Result<(), StepperError> {
    let speed = linear_move_to_3d_inner(stepper_a, stepper_b, stepper_c, dest, speed)?;
    linear_move_to_3d_raw(stepper_a, stepper_b, stepper_c, dest, speed)
}

#[cfg(not(test))]
pub async fn linear_move_for_3d<'s>(
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
pub async fn linear_move_3d_e<'s>(
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
pub async fn linear_move_to_3d_e<'s>(
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
pub async fn linear_move_for_3d_e<'s>(
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
pub async fn arc_move_2d_arc_length<'s>(
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
        let arc_dst = compute_arc_destination(source, center, arc_unit_length, direction);
        linear_move_to_2d(stepper_a, stepper_b, arc_dst, speed).await?;
        source = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);
    }
    Ok(())
}

#[cfg(test)]
pub fn arc_move_2d_arc_length<'s>(
    stepper_a: &mut Stepper<'s>,
    stepper_b: &mut Stepper<'s>,
    arc_length: Distance,
    center: Vector2D<Distance>,
    speed: Speed,
    direction: RotationDirection,
) -> Result<(), StepperError> {
    use math::common::approximate_arc;

    let arc_unit_length = Distance::from_mm(1.0);
    let source = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);
    let points = approximate_arc(source, center, arc_length, direction, arc_unit_length);
    for p in points {
        linear_move_to_2d(stepper_a, stepper_b, p, speed)?;
    }
    Ok(())
}

#[cfg(not(test))]
pub async fn arc_move_3d_e_center<'s>(
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
pub async fn arc_move_3d_e_radius<'s>(
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
pub async fn arc_move_3d_e_offset_from_center<'s>(
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

#[cfg(test)]
#[defmt_test::tests]
mod tests {
    use super::*;
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

    use stepper::{Stepper, StepperAttachment, StepperOptions, SteppingMode};

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
    fn test_linear_move_to_no_move(s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>)) {
        let destination = Distance::from_mm(0.0);
        let speed = Speed::from_mm_per_second(10.0);
        s.0.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = linear_move_to(&mut s.0, destination, speed);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 0.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s.0.get_direction(), RotationDirection::Clockwise);
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
        let res = linear_move_to(&mut s.0, destination, speed);
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
        let res = linear_move_to(&mut s.0, destination, speed);
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
        let res = linear_move_to_2d(&mut s.0, &mut s.1, destination, speed);
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
        let res = linear_move_to_2d(&mut s.0, &mut s.1, destination, speed);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 0.0);
        assert_eq!(s.1.get_steps(), 0.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s.1.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s.0.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s.1.get_direction(), RotationDirection::Clockwise);
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
        let res = linear_move_to_2d(&mut s.0, &mut s.1, destination, speed);
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
        let res = linear_move_to_2d(&mut s.0, &mut s.1, destination, speed);
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
        let res = linear_move_to_3d(&mut s.0, &mut s.1, &mut s.2, destination, speed);
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
        let res = linear_move_to_3d(&mut s.0, &mut s.1, &mut s.2, destination, speed);
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
        let res = linear_move_to_3d(&mut s.0, &mut s.1, &mut s.2, destination, speed);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), 0.0);
        assert_eq!(s.1.get_steps(), 0.0);
        assert_eq!(s.2.get_steps(), 0.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s.1.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s.2.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s.0.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s.1.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s.2.get_direction(), RotationDirection::Clockwise);
    }

    #[test]
    fn test_arc_move_2d_arc_length(s: &mut (Stepper<'static>, Stepper<'static>, Stepper<'static>)) {
        let arc_length = Distance::from_mm(20.0);
        let center = Vector2D::new(Distance::from_mm(10.0), Distance::from_mm(10.0));
        let speed = Speed::from_mm_per_second(10.0);
        let direction = RotationDirection::Clockwise;
        s.0.reset();
        s.1.reset();
        s.0.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s.1.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = arc_move_2d_arc_length(&mut s.0, &mut s.1, arc_length, center, speed, direction);
        assert!(res.is_ok());
        assert_eq!(s.0.get_steps(), -2.0);
        assert_eq!(s.1.get_steps(), 18.0);
        assert_eq!(s.0.get_position().unwrap().to_mm(), -2.0);
        assert_eq!(s.1.get_position().unwrap().to_mm(), 18.0);
        assert_eq!(s.0.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s.1.get_direction(), RotationDirection::Clockwise);
    }
}
