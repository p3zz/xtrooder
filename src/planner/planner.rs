#![allow(dead_code)]

use super::motion;
use crate::parser::parser::GCommand;
use crate::stepper::a4988::Stepper;
use crate::stepper::units::LengthUnit;
use crate::stepper::units::{Position, Position2D, Position3D, Speed};
use embassy_stm32::pwm::CaptureCompare16bitInstance;

pub enum Positioning {
    Relative,
    Absolute,
}

// we need to have a triple(s, d, T) for every stepper
pub struct Planner<'s, X, Y, Z, E> {
    feedrate: Speed,
    unit: LengthUnit,
    positioning: Positioning,
    x_stepper: Stepper<'s, X>,
    y_stepper: Stepper<'s, Y>,
    z_stepper: Stepper<'s, Z>,
    e_stepper: Stepper<'s, E>,
}
impl<'s, X, Y, Z, E> Planner<'s, X, Y, Z, E>
where
    X: CaptureCompare16bitInstance,
    Y: CaptureCompare16bitInstance,
    Z: CaptureCompare16bitInstance,
    E: CaptureCompare16bitInstance,
{
    pub fn new(
        x_stepper: Stepper<'s, X>,
        y_stepper: Stepper<'s, Y>,
        z_stepper: Stepper<'s, Z>,
        e_stepper: Stepper<'s, E>,
    ) -> Planner<'s, X, Y, Z, E> {
        Planner {
            x_stepper,
            y_stepper,
            z_stepper,
            e_stepper,
            feedrate: Speed::from_mmps(0.0).unwrap(),
            unit: LengthUnit::Millimeter,
            positioning: Positioning::Absolute,
        }
    }

    pub async fn execute(&mut self, command: GCommand) {
        match command {
            GCommand::G0 { x, y, z, f } => self.g0(x, y, z, f).await,
            GCommand::G1 { x, y, z, e, f } => self.g1(x, y, z, e, f).await,
            GCommand::G2 { x, y, z, e, f, i, j, r } => todo!(),
            GCommand::G3 { x, y, z, e, f, i, j, r } => todo!(),
            GCommand::G20 => self.g20(),
            GCommand::G21 => self.g21(),
            GCommand::G90 => self.g90(),
            GCommand::G91 => self.g91(),
            GCommand::M104 { s } => todo!(),
        }
    }

    fn g20(&mut self) {
        self.unit = LengthUnit::Inch;
    }

    fn g21(&mut self) {
        self.unit = LengthUnit::Inch;
    }

    fn g90(&mut self) {
        self.positioning = Positioning::Absolute;
    }

    fn g91(&mut self) {
        self.positioning = Positioning::Relative;
    }

    pub async fn g0(&mut self, x: Option<f64>, y: Option<f64>, z: Option<f64>, f: Option<f64>) {
        self.feedrate = match f {
            Some(speed) => Speed::from_mmps(speed).unwrap(),
            None => self.feedrate,
        };
        match (x, y, z) {
            (None, None, None) => (),
            (None, None, Some(z)) => {
                self.linear_move_z(Position::from_unit(z, self.unit), self.feedrate)
                    .await
            }
            (None, Some(y), None) => {
                self.linear_move_y(Position::from_unit(y, self.unit), self.feedrate)
                    .await
            }
            (Some(x), None, None) => {
                self.linear_move_x(Position::from_unit(x, self.unit), self.feedrate)
                    .await
            }
            (None, Some(y), Some(z)) => {
                self.linear_move_yz(
                    Position2D::new(
                        Position::from_unit(y, self.unit),
                        Position::from_unit(z, self.unit),
                    ),
                    self.feedrate,
                )
                .await
            }
            (Some(x), None, Some(z)) => {
                self.linear_move_xz(
                    Position2D::new(
                        Position::from_unit(x, self.unit),
                        Position::from_unit(z, self.unit),
                    ),
                    self.feedrate,
                )
                .await
            }
            (Some(x), Some(y), None) => {
                self.linear_move_xy(
                    Position2D::new(
                        Position::from_unit(x, self.unit),
                        Position::from_unit(y, self.unit),
                    ),
                    self.feedrate,
                )
                .await
            }
            (Some(x), Some(y), Some(z)) => {
                self.linear_move_xyz(
                    Position3D::new(
                        Position::from_unit(x, self.unit),
                        Position::from_unit(y, self.unit),
                        Position::from_unit(z, self.unit),
                    ),
                    self.feedrate,
                )
                .await
            }
        }
    }

    pub async fn g1(
        &mut self,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        e: Option<f64>,
        f: Option<f64>,
    ) {
        let e_dest = match e {
            Some(e_dest) => Position::from_mm(e_dest),
            None => return self.g0(x, y, z, f).await,
        };

        self.feedrate = match f {
            Some(speed) => Speed::from_mmps(speed).unwrap(),
            None => self.feedrate,
        };

        match (x, y, z) {
            (None, None, None) => (),
            (None, None, Some(z)) => {
                self.linear_move_ze(Position::from_unit(z, self.unit), e_dest, self.feedrate)
                    .await
            }
            (None, Some(y), None) => {
                self.linear_move_ye(Position::from_unit(y, self.unit), e_dest, self.feedrate)
                    .await
            }
            (Some(x), None, None) => {
                self.linear_move_xe(Position::from_unit(x, self.unit), e_dest, self.feedrate)
                    .await
            }
            (None, Some(y), Some(z)) => {
                self.linear_move_yze(
                    Position2D::new(
                        Position::from_unit(y, self.unit),
                        Position::from_unit(z, self.unit),
                    ),
                    self.feedrate,
                    e_dest,
                )
                .await
            }
            (Some(x), None, Some(z)) => {
                self.linear_move_xze(
                    Position2D::new(
                        Position::from_unit(x, self.unit),
                        Position::from_unit(z, self.unit),
                    ),
                    self.feedrate,
                    e_dest,
                )
                .await
            }
            (Some(x), Some(y), None) => {
                self.linear_move_xye(
                    Position2D::new(
                        Position::from_unit(x, self.unit),
                        Position::from_unit(y, self.unit),
                    ),
                    self.feedrate,
                    e_dest,
                )
                .await
            }
            (Some(x), Some(y), Some(z)) => {
                self.linear_move_xyze(
                    Position3D::new(
                        Position::from_unit(x, self.unit),
                        Position::from_unit(y, self.unit),
                        Position::from_unit(z, self.unit),
                    ),
                    self.feedrate,
                    e_dest,
                )
                .await
            }
        }
    }

    pub async fn linear_move_x(&mut self, dest: Position, feedrate: Speed) {
        motion::linear_move_to(&mut self.x_stepper, dest, feedrate).await
    }

    pub async fn linear_move_xe(&mut self, dest: Position, e_dest: Position, feedrate: Speed) {
        motion::linear_move_to_e(
            &mut self.x_stepper,
            &mut self.e_stepper,
            dest,
            e_dest,
            feedrate,
        )
        .await
    }

    pub async fn linear_move_y(&mut self, dest: Position, feedrate: Speed) {
        motion::linear_move_to(&mut self.y_stepper, dest, feedrate).await
    }

    pub async fn linear_move_ye(&mut self, dest: Position, e_dest: Position, feedrate: Speed) {
        motion::linear_move_to_e(
            &mut self.y_stepper,
            &mut self.e_stepper,
            dest,
            e_dest,
            feedrate,
        )
        .await
    }

    pub async fn linear_move_z(&mut self, dest: Position, feedrate: Speed) {
        motion::linear_move_to(&mut self.z_stepper, dest, feedrate).await
    }

    pub async fn linear_move_ze(&mut self, dest: Position, e_dest: Position, feedrate: Speed) {
        motion::linear_move_to_e(
            &mut self.z_stepper,
            &mut self.e_stepper,
            dest,
            e_dest,
            feedrate,
        )
        .await
    }

    pub async fn linear_move_xy(&mut self, dest: Position2D, feedrate: Speed) {
        motion::linear_move_to_2d(&mut self.x_stepper, &mut self.y_stepper, dest, feedrate).await
    }

    pub async fn linear_move_xye(&mut self, dest: Position2D, feedrate: Speed, e_dst: Position) {
        motion::linear_move_to_2d_e(
            &mut self.x_stepper,
            &mut self.y_stepper,
            &mut self.e_stepper,
            dest,
            e_dst,
            feedrate,
        )
        .await
    }

    pub async fn linear_move_xz(&mut self, dest: Position2D, feedrate: Speed) {
        motion::linear_move_to_2d(&mut self.x_stepper, &mut self.z_stepper, dest, feedrate).await;
    }

    pub async fn linear_move_xze(&mut self, dest: Position2D, feedrate: Speed, e_dst: Position) {
        motion::linear_move_to_2d_e(
            &mut self.x_stepper,
            &mut self.z_stepper,
            &mut self.e_stepper,
            dest,
            e_dst,
            feedrate,
        )
        .await
    }

    pub async fn linear_move_yz(&mut self, dest: Position2D, feedrate: Speed) {
        motion::linear_move_to_2d(&mut self.y_stepper, &mut self.z_stepper, dest, feedrate).await;
    }

    pub async fn linear_move_yze(&mut self, dest: Position2D, feedrate: Speed, e_dst: Position) {
        motion::linear_move_to_2d_e(
            &mut self.y_stepper,
            &mut self.z_stepper,
            &mut self.e_stepper,
            dest,
            e_dst,
            feedrate,
        )
        .await
    }

    pub async fn linear_move_xyz(&mut self, dest: Position3D, feedrate: Speed) {
        todo!()
    }

    pub async fn linear_move_xyze(&mut self, dest: Position3D, feedrate: Speed, e_dst: Position) {
        todo!()
    }
}
