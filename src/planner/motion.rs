use crate::math::angle::{cos, sin};
use crate::math::common::abs;
use crate::math::computable::Computable;
use crate::math::vector::{Vector, Vector2D};
use crate::stepper::a4988::Stepper;
use embassy_stm32::pwm::CaptureCompare16bitInstance;
use futures::join;
use micromath::F32Ext;

pub async fn linear_move_to<'s, S: CaptureCompare16bitInstance>(
    stepper: &mut Stepper<'s, S>,
    dest: Vector,
    speed: Vector,
) {
    stepper.set_speed(speed);
    stepper.move_to(dest).await;
}

pub async fn linear_move_to_e<
    's,
    A: CaptureCompare16bitInstance,
    E: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_e: &mut Stepper<'s, E>,
    a_dest: Vector,
    e_dest: Vector,
    a_speed: Vector,
) {
    // compute the time the stepper a takes to go from its position to the destination, at the given speed, then compute
    // the speed for the extruder stepper
    let a_distance = a_dest.sub(stepper_a.get_position());
    let a_time = abs(a_distance.div(a_speed).to_mm());

    let e_distance = e_dest.sub(stepper_e.get_position());
    let e_speed = Vector::from_mm(e_distance.to_mm() / a_time);

    join!(
        linear_move_to(stepper_a, a_dest, a_speed),
        linear_move_to(stepper_e, e_dest, e_speed)
    );
}

pub async fn linear_move_to_2d<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    dest: Vector2D,
    speed: Vector,
) {
    let src = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let direction = dest.sub(src).normalize();

    let ab_speed = direction.mul(speed);

    linear_move_to_2d_raw(stepper_a, stepper_b, dest, ab_speed).await;
}

pub async fn linear_move_to_2d_raw<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    dest: Vector2D,
    speed: Vector2D,
) {
    join!(
        linear_move_to(stepper_a, dest.get_x(), speed.get_x()),
        linear_move_to(stepper_b, dest.get_y(), speed.get_y())
    );
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
    ab_dest: Vector2D,
    e_dest: Vector,
    ab_speed: Vector,
) {
    let ab_source = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let time_taken = ab_dest.sub(ab_source).get_magnitude().div(ab_speed);
    let e_delta = e_dest.sub(stepper_e.get_position());
    let e_speed = e_delta.div(time_taken);
    join!(
        linear_move_to_2d(stepper_a, stepper_b, ab_dest, ab_speed),
        linear_move_to(stepper_e, e_dest, e_speed)
    );
}

pub async fn linear_move_for<'s, S: CaptureCompare16bitInstance>(
    stepper: &mut Stepper<'s, S>,
    distance: Vector,
    speed: Vector,
) {
    let dest = stepper.get_position().add(distance);
    linear_move_to(stepper, dest, speed).await;
}

pub async fn linear_move_for_e<
    's,
    A: CaptureCompare16bitInstance,
    E: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_e: &mut Stepper<'s, E>,
    a_distance: Vector,
    e_distance: Vector,
    feedrate: Vector,
) {
    let a_dest = stepper_a.get_position().add(a_distance);
    let e_dest = stepper_e.get_position().add(e_distance);
    linear_move_to_e(stepper_a, stepper_e, a_dest, e_dest, feedrate).await;
}

pub async fn linear_move_for_2d<
    's,
    A: CaptureCompare16bitInstance,
    B: CaptureCompare16bitInstance,
>(
    stepper_a: &mut Stepper<'s, A>,
    stepper_b: &mut Stepper<'s, B>,
    distance: Vector2D,
    speed: Vector2D,
) {
    let source = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let dest = source.add(distance);
    linear_move_to_2d_raw(stepper_a, stepper_b, dest, speed).await;
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
    ab_distance: Vector2D,
    e_distance: Vector,
    ab_speed: Vector,
) {
    let ab_source = Vector2D::new(stepper_a.get_position(), stepper_b.get_position());
    let ab_dest = ab_source.add(ab_distance);
    let e_dest = stepper_e.get_position().add(e_distance);
    linear_move_to_2d_e(stepper_a, stepper_b, stepper_e, ab_dest, e_dest, ab_speed).await;
}
