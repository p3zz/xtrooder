use core::time::Duration;

use futures::join;
use math::angle::{cos, sin};
use math::common::{
    abs, compute_arc_destination, compute_arc_length, floor, max, RotationDirection,
};
use math::computable::Computable;
use math::distance::Distance;
use math::speed::Speed;
use math::vector::{Vector2D, Vector3D};

use crate::stepper::{StatefulOutputPin, Stepper, StepperError, TimerTrait};

#[derive(Clone, Copy)]
pub enum Positioning {
    Relative,
    Absolute,
}

pub fn no_move<P: StatefulOutputPin>(
    stepper: &Stepper<P>,
    positioning: Positioning,
) -> Result<Distance, StepperError> {
    match positioning {
        Positioning::Relative => Ok(Distance::from_mm(0.0)),
        Positioning::Absolute => stepper.get_position(),
    }
}

// ---------------------------- LINEAR MOVE 1D ----------------------------

pub async fn linear_move_to<P: StatefulOutputPin, T: TimerTrait>(
    stepper: &mut Stepper<P>,
    dest: Distance,
    speed: Speed,
) -> Result<Duration, StepperError> {
    let s = Speed::from_mm_per_second(abs(speed.to_mm_per_second()));
    stepper.set_speed_from_attachment(s)?;
    stepper.move_to_destination::<T>(dest).await
}

// ---------------------------- LINEAR MOVE 2D ----------------------------

async fn linear_move_to_2d_raw<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    dest: Vector2D<Distance>,
    speed: Vector2D<Speed>,
) -> Result<Duration, StepperError> {
    match join!(
        linear_move_to::<P, T>(stepper_a, dest.get_x(), speed.get_x()),
        linear_move_to::<P, T>(stepper_b, dest.get_y(), speed.get_y()),
    ) {
        (Ok(da), Ok(db)) => {
            let max = *max(&[da.as_micros(), db.as_micros()]).unwrap();
            Ok(Duration::from_micros(max as u64))
        }
        _ => Err(StepperError::MoveNotValid),
    }
}

fn linear_move_to_2d_inner<P: StatefulOutputPin>(
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    dest: Vector2D<Distance>,
    speed: Speed,
) -> Result<Vector2D<Speed>, StepperError> {
    let src = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);
    let angle = dest.sub(&src).get_angle();
    let speed_x = Speed::from_mm_per_second(cos(angle) * speed.to_mm_per_second());
    let speed_y = Speed::from_mm_per_second(sin(angle) * speed.to_mm_per_second());

    Ok(Vector2D::new(speed_x, speed_y))
}

pub async fn linear_move_to_2d<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    dest: Vector2D<Distance>,
    speed: Speed,
) -> Result<Duration, StepperError> {
    let speed = linear_move_to_2d_inner(stepper_a, stepper_b, dest, speed)?;
    linear_move_to_2d_raw::<P, T>(stepper_a, stepper_b, dest, speed).await
}

// ---------------------------- LINEAR MOVE 3D ----------------------------

pub async fn linear_move_3d<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    stepper_c: &mut Stepper<P>,
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
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    stepper_c: &mut Stepper<P>,
    dest: Vector3D<Distance>,
    speed: Vector3D<Speed>,
) -> Result<Duration, StepperError> {
    match join!(
        linear_move_to::<P, T>(stepper_a, dest.get_x(), speed.get_x()),
        linear_move_to::<P, T>(stepper_b, dest.get_y(), speed.get_y()),
        linear_move_to::<P, T>(stepper_c, dest.get_z(), speed.get_z()),
    ) {
        (Ok(da), Ok(db), Ok(dc)) => {
            let max = *max(&[da.as_micros(), db.as_micros(), dc.as_micros()]).unwrap();
            Ok(Duration::from_micros(max as u64))
        }
        _ => Err(StepperError::MoveNotValid),
    }
}

pub fn linear_move_to_3d_inner<P: StatefulOutputPin>(
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    stepper_c: &mut Stepper<P>,
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

pub async fn linear_move_to_3d<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    stepper_c: &mut Stepper<P>,
    dest: Vector3D<Distance>,
    speed: Speed,
) -> Result<Duration, StepperError> {
    let speed = linear_move_to_3d_inner::<P>(stepper_a, stepper_b, stepper_c, dest, speed)?;
    linear_move_to_3d_raw::<P, T>(stepper_a, stepper_b, stepper_c, dest, speed).await
}

pub async fn linear_move_for_3d<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    stepper_c: &mut Stepper<P>,
    distance: Vector3D<Distance>,
    speed: Speed,
) -> Result<Duration, StepperError> {
    let source = Vector3D::new(
        stepper_a.get_position()?,
        stepper_b.get_position()?,
        stepper_c.get_position()?,
    );
    let dest = source.add(&distance);
    linear_move_to_3d::<P, T>(stepper_a, stepper_b, stepper_c, dest, speed).await
}

pub async fn linear_move_3d_e<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    stepper_c: &mut Stepper<P>,
    stepper_e: &mut Stepper<P>,
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
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    stepper_c: &mut Stepper<P>,
    stepper_e: &mut Stepper<P>,
    dest: Vector3D<Distance>,
    speed: Speed,
    e_dest: Distance,
) -> Result<Duration, StepperError> {
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
        linear_move_to_3d::<P, T>(stepper_a, stepper_b, stepper_c, dest, speed),
        linear_move_to::<P, T>(stepper_e, e_dest, e_speed)
    ) {
        (Ok(dabc), Ok(de)) => {
            let max = *max(&[dabc.as_micros(), de.as_micros()]).unwrap();
            Ok(Duration::from_micros(max as u64))
        }
        _ => Err(StepperError::MoveNotValid),
    }
}

pub async fn linear_move_for_3d_e<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    stepper_c: &mut Stepper<P>,
    stepper_e: &mut Stepper<P>,
    distance: Vector3D<Distance>,
    speed: Speed,
    e_distance: Distance,
) -> Result<Duration, StepperError> {
    let src = Vector3D::new(
        stepper_a.get_position()?,
        stepper_b.get_position()?,
        stepper_c.get_position()?,
    );
    let abc_destination = src.add(&distance);
    let e_destination = stepper_e.get_position()?.add(&e_distance);

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
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    arc_length: Distance,
    center: Vector2D<Distance>,
    speed: Speed,
    direction: RotationDirection,
) -> Result<Duration, StepperError> {
    let arc_unit_length = Distance::from_mm(1.0);
    if arc_length.to_mm() < arc_unit_length.to_mm() {
        return Err(StepperError::MoveTooShort);
    }
    let source = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);
    let arcs_n = floor(arc_length.div(&arc_unit_length).unwrap()) as u64;
    let mut total_duration = Duration::ZERO;
    for n in 0..(arcs_n + 1) {
        let arc_length = Distance::from_mm(arc_unit_length.to_mm() * n as f64);
        let arc_dst = compute_arc_destination(source, center, arc_length, direction);
        total_duration += linear_move_to_2d::<P, T>(stepper_a, stepper_b, arc_dst, speed).await?;
    }
    Ok(total_duration)
}

pub async fn arc_move_3d_e_center<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    stepper_c: &mut Stepper<P>,
    stepper_e: &mut Stepper<P>,
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
    let xy_src = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);

    let arc_length = compute_arc_length(xy_src, xy_center, xy_dest, direction, full_circle_enabled);

    let time = arc_length.to_mm() / speed.to_mm_per_second();

    let z_delta = dest.get_z().sub(&stepper_c.get_position()?);
    let z_speed = Speed::from_mm_per_second(z_delta.to_mm() / time);

    let e_delta = e_dest.sub(&stepper_e.get_position()?);
    let e_speed = Speed::from_mm_per_second(e_delta.to_mm() / time);

    match join!(
        arc_move_2d_arc_length::<P, T>(
            stepper_a, stepper_b, arc_length, xy_center, speed, direction
        ),
        linear_move_to::<P, T>(stepper_c, dest.get_z(), z_speed),
        linear_move_to::<P, T>(stepper_e, e_dest, e_speed)
    ) {
        (Ok(dab), Ok(dc), Ok(de)) => {
            let max = *max(&[dab.as_micros(), dc.as_micros(), de.as_micros()]).unwrap();
            Ok(Duration::from_micros(max as u64))
        }
        _ => Err(StepperError::MoveNotValid),
    }
}

pub async fn arc_move_3d_e_radius<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    stepper_c: &mut Stepper<P>,
    stepper_e: &mut Stepper<P>,
    dest: Vector3D<Distance>,
    radius: Distance,
    speed: Speed,
    direction: RotationDirection,
    e_dest: Distance,
) -> Result<Duration, StepperError> {
    let source = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);
    let angle = source.get_angle();
    let center_offset_x = Distance::from_mm(radius.to_mm() * cos(angle));
    let center_offset_y = Distance::from_mm(radius.to_mm() * sin(angle));
    let center = source.add(&Vector2D::new(center_offset_x, center_offset_y));
    arc_move_3d_e_center::<P, T>(
        stepper_a, stepper_b, stepper_c, stepper_e, dest, center, speed, direction, e_dest, false,
    )
    .await
}

pub async fn arc_move_3d_e_offset_from_center<P: StatefulOutputPin, T: TimerTrait>(
    stepper_a: &mut Stepper<P>,
    stepper_b: &mut Stepper<P>,
    stepper_c: &mut Stepper<P>,
    stepper_e: &mut Stepper<P>,
    dest: Vector3D<Distance>,
    offset: Vector2D<Distance>,
    speed: Speed,
    direction: RotationDirection,
    e_dest: Distance,
) -> Result<Duration, StepperError> {
    let source = Vector2D::new(stepper_a.get_position()?, stepper_b.get_position()?);
    let center = source.add(&offset);
    arc_move_3d_e_center::<P, T>(
        stepper_a, stepper_b, stepper_c, stepper_e, dest, center, speed, direction, e_dest, true,
    )
    .await
}

#[cfg(test)]
mod tests {
    use math::{
        common::RotationDirection,
        distance::Distance,
        speed::Speed,
        vector::{Vector2D, Vector3D},
    };

    use crate::stepper::{StepperAttachment, StepperOptions, SteppingMode};
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

    #[tokio::test]
    async fn test_linear_move_to_no_move() {
        let mut s = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let destination = Distance::from_mm(0.0);
        let speed = Speed::from_mm_per_second(10.0);
        s.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res =
            linear_move_to::<StatefulOutputPinMock, StepperTimer>(&mut s, destination, speed).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 0.0);
        assert_eq!(s.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s.get_direction(), RotationDirection::Clockwise);
        assert!(s.get_speed_from_attachment().is_ok());
    }

    #[tokio::test]
    async fn test_linear_move_to() {
        let mut s = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let destination = Distance::from_mm(10.0);
        let speed = Speed::from_mm_per_second(10.0);
        s.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res =
            linear_move_to::<StatefulOutputPinMock, StepperTimer>(&mut s, destination, speed).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), 10.0);
        assert_eq!(s.get_position().unwrap().to_mm(), 10.0);
        assert_eq!(s.get_direction(), RotationDirection::Clockwise);
        assert!(s.get_speed_from_attachment().is_ok());
        assert_eq!(
            s.get_speed_from_attachment().unwrap().to_mm_per_second(),
            10.0
        );
    }

    #[tokio::test]
    async fn test_linear_move_to_negative_speed() {
        let mut s = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let destination = Distance::from_mm(-10.0);
        let speed = Speed::from_mm_per_second(-10.0);
        s.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res =
            linear_move_to::<StatefulOutputPinMock, StepperTimer>(&mut s, destination, speed).await;
        assert!(res.is_ok());
        assert_eq!(s.get_steps(), -10.0);
        assert_eq!(s.get_position().unwrap().to_mm(), -10.0);
        assert_eq!(s.get_direction(), RotationDirection::CounterClockwise);
    }

    #[tokio::test]
    async fn test_linear_move_to_2d() {
        let mut s_x = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let mut s_y = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let destination = Vector2D::new(Distance::from_mm(-10.0), Distance::from_mm(-10.0));
        let speed = Speed::from_mm_per_second(-10.0);
        s_x.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s_y.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = linear_move_to_2d::<StatefulOutputPinMock, StepperTimer>(
            &mut s_x,
            &mut s_y,
            destination,
            speed,
        )
        .await;
        assert!(res.is_ok());
        assert_eq!(s_x.get_steps(), -10.0);
        assert_eq!(s_y.get_steps(), -10.0);
        assert_eq!(s_x.get_position().unwrap().to_mm(), -10.0);
        assert_eq!(s_y.get_position().unwrap().to_mm(), -10.0);
        assert_eq!(s_x.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::CounterClockwise);
        assert!(s_x.get_speed_from_attachment().is_ok());
        assert_eq!(
            s_x.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.078142695356739
        );
        assert_eq!(
            s_y.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.078142695356739
        );
    }

    #[tokio::test]
    async fn test_linear_move_to_2d_no_move() {
        let mut s_x = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let mut s_y = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let destination = Vector2D::new(Distance::from_mm(0.0), Distance::from_mm(0.0));
        let speed = Speed::from_mm_per_second(-10.0);
        s_x.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s_y.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = linear_move_to_2d::<StatefulOutputPinMock, StepperTimer>(
            &mut s_x,
            &mut s_y,
            destination,
            speed,
        )
        .await;
        assert!(res.is_ok());
        assert_eq!(s_x.get_steps(), 0.0);
        assert_eq!(s_y.get_steps(), 0.0);
        assert_eq!(s_x.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s_y.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s_x.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert!(s_x.get_speed_from_attachment().is_ok());
        assert!(s_y.get_speed_from_attachment().is_ok());
        assert_eq!(
            s_x.get_speed_from_attachment().unwrap().to_mm_per_second(),
            10.0
        );
        assert_eq!(
            s_y.get_speed_from_attachment().unwrap().to_mm_per_second(),
            0.0
        );
    }

    #[tokio::test]
    async fn test_linear_move_to_2d_2() {
        let mut s_x = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let mut s_y = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let destination = Vector2D::new(Distance::from_mm(-5.0), Distance::from_mm(5.0));
        let speed = Speed::from_mm_per_second(10.0);
        s_x.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s_y.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = linear_move_to_2d::<StatefulOutputPinMock, StepperTimer>(
            &mut s_x,
            &mut s_y,
            destination,
            speed,
        )
        .await;
        assert!(res.is_ok());
        assert_eq!(s_x.get_steps(), -5.0);
        assert_eq!(s_y.get_steps(), 5.0);
        assert_eq!(s_x.get_position().unwrap().to_mm(), -5.0);
        assert_eq!(s_y.get_position().unwrap().to_mm(), 5.0);
        assert_eq!(s_x.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert_eq!(
            s_x.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.078142695356739
        );
        assert_eq!(
            s_y.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.078142695356739
        );
    }

    #[tokio::test]
    async fn test_linear_move_to_2d_different_stepping_mode() {
        let mut s_x = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let mut s_y = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let destination = Vector2D::new(Distance::from_mm(-5.0), Distance::from_mm(5.0));
        let speed = Speed::from_mm_per_second(10.0);
        s_x.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s_y.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
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
        assert_eq!(s_x.get_steps(), -5.0);
        assert_eq!(s_y.get_steps(), 5.0);
        assert_eq!(s_x.get_position().unwrap().to_mm(), -5.0);
        assert_eq!(s_y.get_position().unwrap().to_mm(), 5.0);
        assert_eq!(s_x.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert_eq!(
            s_x.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.078142695356739
        );
        assert_eq!(
            s_y.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.078142695356739
        );
    }

    #[tokio::test]
    async fn test_linear_move_to_3d() {
        let mut s_x = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let mut s_y = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let mut s_z = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let destination = Vector3D::new(
            Distance::from_mm(-5.0),
            Distance::from_mm(5.0),
            Distance::from_mm(5.0),
        );
        let speed = Speed::from_mm_per_second(10.0);
        s_x.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s_y.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s_z.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
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
        assert_eq!(s_x.get_steps(), -5.0);
        assert_eq!(s_y.get_steps(), 5.0);
        assert_eq!(s_z.get_steps(), 5.0);
        assert_eq!(s_x.get_position().unwrap().to_mm(), -5.0);
        assert_eq!(s_y.get_position().unwrap().to_mm(), 5.0);
        assert_eq!(s_z.get_position().unwrap().to_mm(), 5.0);
        assert_eq!(s_x.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_z.get_direction(), RotationDirection::Clockwise);
        assert_eq!(
            s_x.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.078142695356739
        );
        assert_eq!(
            s_y.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.078142695356739
        );
        assert_eq!(
            s_z.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.078142695356739
        );
    }

    #[tokio::test]
    async fn test_linear_move_to_3d_lower_distance_per_step() {
        let mut s_x = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let mut s_y = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let mut s_z = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let destination = Vector3D::new(
            Distance::from_mm(-5.0),
            Distance::from_mm(-2.0),
            Distance::from_mm(5.0),
        );
        let speed = Speed::from_mm_per_second(10.0);
        s_x.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(0.5),
        });
        s_y.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(0.5),
        });
        s_z.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(0.5),
        });
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
        assert_eq!(s_x.get_steps(), -10.0);
        assert_eq!(s_y.get_steps(), -4.0);
        assert_eq!(s_z.get_steps(), 10.0);
        assert_eq!(s_x.get_position().unwrap().to_mm(), -5.0);
        assert_eq!(s_y.get_position().unwrap().to_mm(), -2.0);
        assert_eq!(s_z.get_position().unwrap().to_mm(), 5.0);
        assert_eq!(s_x.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::CounterClockwise);
        assert_eq!(s_z.get_direction(), RotationDirection::Clockwise);
        assert_eq!(
            s_x.get_speed_from_attachment().unwrap().to_mm_per_second(),
            9.282120778955576
        );
        assert_eq!(
            s_y.get_speed_from_attachment().unwrap().to_mm_per_second(),
            3.725338260714073
        );
        assert_eq!(
            s_z.get_speed_from_attachment().unwrap().to_mm_per_second(),
            7.078142695356739
        );
    }

    #[tokio::test]
    async fn test_linear_move_to_3d_no_move() {
        let mut s_x = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let mut s_y = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let mut s_z = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let destination = Vector3D::new(
            Distance::from_mm(0.0),
            Distance::from_mm(0.0),
            Distance::from_mm(0.0),
        );
        let speed = Speed::from_mm_per_second(10.0);
        s_x.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s_y.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s_z.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
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
        assert_eq!(s_x.get_steps(), 0.0);
        assert_eq!(s_y.get_steps(), 0.0);
        assert_eq!(s_z.get_steps(), 0.0);
        assert_eq!(s_x.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s_y.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s_z.get_position().unwrap().to_mm(), 0.0);
        assert_eq!(s_x.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_z.get_direction(), RotationDirection::Clockwise);
    }

    #[tokio::test]
    async fn test_arc_move_2d_arc_length() {
        let mut s_x = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let mut s_y = Stepper::new(
            StatefulOutputPinMock::new(),
            StatefulOutputPinMock::new(),
            StepperOptions::default(),
            Some(StepperAttachment {
                distance_per_step: Distance::from_mm(1.0),
            }),
        );
        let arc_length = Distance::from_mm(20.0);
        let center = Vector2D::new(Distance::from_mm(10.0), Distance::from_mm(10.0));
        let speed = Speed::from_mm_per_second(10.0);
        let direction = RotationDirection::Clockwise;
        s_x.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        s_y.set_attachment(StepperAttachment {
            distance_per_step: Distance::from_mm(1.0),
        });
        let res = arc_move_2d_arc_length::<StatefulOutputPinMock, StepperTimer>(
            &mut s_x, &mut s_y, arc_length, center, speed, direction,
        )
        .await;
        assert!(res.is_ok());
        assert_eq!(s_x.get_steps(), -2.0);
        assert_eq!(s_y.get_steps(), 18.0);
        assert_eq!(s_x.get_position().unwrap().to_mm(), -2.0);
        assert_eq!(s_y.get_position().unwrap().to_mm(), 18.0);
        assert_eq!(s_x.get_direction(), RotationDirection::Clockwise);
        assert_eq!(s_y.get_direction(), RotationDirection::Clockwise);
    }
}
