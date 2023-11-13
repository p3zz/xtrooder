use crate::stepper::a4988::{Stepper, StepperDirection};
use crate::stepper::units::{Speed, Length, Position2D, Position};
use micromath::F32Ext;
use futures::join;
use embassy_stm32::pwm::CaptureCompare16bitInstance;

pub async fn linear_move_2d
<'sa, 'da, 'sb, 'db, A: CaptureCompare16bitInstance, B: CaptureCompare16bitInstance>
(stepper_a: &mut Stepper<'sa, 'da, A>, stepper_b: &mut Stepper<'sb, 'db, B>, dest: Position2D, feedrate: Speed){
    let src = Position2D::new(stepper_a.get_position(), stepper_b.get_position());
    let th = src.angle(dest);

    // compute the velocity out of the speed and its angle
    let a_f = feedrate.to_mmps() as f32 * th.cos();
    let a_feedrate = Speed::from_mmps(a_f.abs() as f64).unwrap();

    let b_f = feedrate.to_mmps() as f32 * th.sin();
    let b_feedrate = Speed::from_mmps(b_f.abs() as f64).unwrap();

    join!(linear_move(stepper_a, dest.get_x(), a_feedrate), linear_move(stepper_b, dest.get_y(), b_feedrate));
}

pub async fn linear_move_2d_e
<'sa, 'da, 'sb, 'db, 'se, 'de, A: CaptureCompare16bitInstance, B: CaptureCompare16bitInstance, E: CaptureCompare16bitInstance>
(stepper_a: &mut Stepper<'sa, 'da, A>, stepper_b: &mut Stepper<'sb, 'db, B>, stepper_e: &mut Stepper<'se, 'de, E>, ab_dest: Position2D, e_dest: Position, feedrate: Speed){
    let source = Position2D::new(stepper_a.get_position(), stepper_b.get_position());
    let delta =  source.subtract(ab_dest);
    let distance = delta.get_magnitude();
    let time_taken = distance.to_mm() / feedrate.to_mmps();
    let e_delta = e_dest.subtract(stepper_e.get_position());
    let e_speed = Speed::from_mmps(e_delta.to_mm() / time_taken).unwrap();
    join!(
        linear_move_2d(stepper_a, stepper_b, ab_dest, feedrate),
        linear_move(stepper_e, e_dest, e_speed)
    );
}

pub async fn linear_move
<'s, 'd, T: CaptureCompare16bitInstance>
(stepper: &mut Stepper<'s, 'd, T>, dest: Position, feedrate: Speed){
    stepper.set_speed(feedrate);
    stepper.move_to(dest).await;
}

pub async fn linear_move_e
<'sa, 'da, 'se, 'de, A: CaptureCompare16bitInstance, E: CaptureCompare16bitInstance>
(stepper_a: &mut Stepper<'sa, 'da, A>, stepper_e: &mut Stepper<'se, 'de, E>, dest: Position, e_dest: Position, feedrate: Speed){
    // compute the time the stepper a takes to go from its position to the destination, at the given speed, then compute
    // the speed for the extruder stepper 
    let a_distance =  dest.subtract(stepper_a.get_position());
    let a_time = a_distance.to_mm() / feedrate.to_mmps();

    let e_distance = e_dest.subtract(stepper_e.get_position());
    let e_speed = Speed::from_mmps(e_distance.to_mm() / a_time).unwrap();

    join!(linear_move(stepper_a, dest, feedrate), linear_move(stepper_e, e_dest, e_speed));
}