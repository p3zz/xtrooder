use crate::motion::auto_home;

use super::motion::{
    arc_move_3d_e_offset_from_center, arc_move_3d_e_radius, linear_move_3d, linear_move_3d_e,
    linear_move_to, no_move, retract, Positioning,
};
use super::stepper::{Attached, Stepper, StepperError};
use core::marker::PhantomData;
use core::time::Duration;
use math::common::RotationDirection;
use math::measurements::{Distance, Length, Speed};
use math::vector::{Vector2D, Vector3D};
use parser::gcode::GCommand;

use common::{TimerBase, ExtiInputPinBase, OutputPinBase};

#[derive(Clone, Copy)]
pub struct RecoverMotionConfig {
    pub feedrate: Speed,
    pub length: Length,
}

#[derive(Clone, Copy)]
pub struct RetractionMotionConfig {
    pub feedrate: Speed,
    pub length: Length,
    pub z_lift: Length,
}

pub struct MotionConfig {
    pub arc_unit_length: Length,
    pub feedrate: Speed,
    pub positioning: Positioning,
    pub feedrate_multiplier: f64,
    pub retraction: RetractionMotionConfig,
    pub recover: RecoverMotionConfig,
}

pub struct Planner<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase> {
    x_stepper: Stepper<P, Attached>,
    y_stepper: Stepper<P, Attached>,
    z_stepper: Stepper<P, Attached>,
    e_stepper: Stepper<P, Attached>,
    config: MotionConfig,
    _timer: PhantomData<T>,
    endstops: (Option<I>, Option<I>, Option<I>, Option<I>),
}

impl<P: OutputPinBase, T: TimerBase, I: ExtiInputPinBase> Planner<P, T, I> {
    pub fn new(
        x_stepper: Stepper<P, Attached>,
        y_stepper: Stepper<P, Attached>,
        z_stepper: Stepper<P, Attached>,
        e_stepper: Stepper<P, Attached>,
        config: MotionConfig,
        endstops: (Option<I>, Option<I>, Option<I>, Option<I>),
    ) -> Self {
        Planner {
            x_stepper,
            y_stepper,
            z_stepper,
            e_stepper,
            _timer: PhantomData,
            config,
            endstops,
        }
    }

    pub fn get_x_position(&self) -> Distance {
        self.x_stepper.get_position()
    }

    pub fn get_y_position(&self) -> Distance {
        self.y_stepper.get_position()
    }

    pub fn get_z_position(&self) -> Distance {
        self.z_stepper.get_position()
    }

    pub fn get_e_position(&self) -> Distance {
        self.e_stepper.get_position()
    }

    pub async fn execute(&mut self, command: GCommand) -> Result<Option<Duration>, StepperError> {
        match command {
            GCommand::G0 { x, y, z, f } => {
                let duration = self.g0(x, y, z, f).await?;
                Ok(Some(duration))
            }
            GCommand::G1 { x, y, z, e, f } => {
                let duration = self.g1(x, y, z, e, f).await?;
                Ok(Some(duration))
            }
            GCommand::G2 {
                x,
                y,
                z,
                e,
                f,
                i,
                j,
                r,
            } => {
                let duration = self.g2(x, y, z, e, f, i, j, r).await?;
                Ok(Some(duration))
            }
            GCommand::G3 {
                x,
                y,
                z,
                e,
                f,
                i,
                j,
                r,
            } => {
                let duration = self.g3(x, y, z, e, f, i, j, r).await?;
                Ok(Some(duration))
            }
            GCommand::G4 { p, s } => {
                self.g4(p, s).await;
                Ok(None)
            }
            GCommand::G90 => {
                self.g90();
                Ok(None)
            }
            GCommand::G91 => {
                self.g91();
                Ok(None)
            }
            GCommand::G10 => {
                self.g10().await?;
                Ok(None)
            }
            GCommand::G11 => {
                self.g11().await?;
                Ok(None)
            }
            GCommand::G28 { x, y, z } => {
                let duration = self.g28((x, y, z)).await?;
                Ok(Some(duration))
            }
            GCommand::M207 { f, s, z } => {
                self.m207(f, s, z);
                Ok(None)
            }
            GCommand::M208 { f, s } => {
                self.m208(f, s);
                Ok(None)
            }
            GCommand::M220 { s } => {
                self.config.feedrate_multiplier = s;
                Ok(None)
            }
            _ => Err(StepperError::NotSupported),
        }
    }

    async fn g4(&mut self, p: Option<core::time::Duration>, s: Option<core::time::Duration>) {
        let d = match (p, s) {
            (None, None) => None,
            (None, Some(_)) | (Some(_), Some(_)) => s,
            (Some(_), None) => p,
        };
        if let Some(duration) = d {
            T::after(duration).await
        }
    }

    fn g90(&mut self) {
        self.config.positioning = Positioning::Absolute;
    }

    fn g91(&mut self) {
        self.config.positioning = Positioning::Relative;
    }

    // firmware retraction settings
    fn m207(&mut self, f: Speed, s: Distance, z: Distance) {
        self.config.retraction.feedrate = f;
        self.config.retraction.length = s;
        self.config.retraction.z_lift = z;
    }

    fn m208(&mut self, f: Speed, s: Distance) {
        self.config.recover.feedrate = f;
        self.config.recover.length = s + self.config.retraction.length;
    }

    async fn g0(
        &mut self,
        x: Option<Distance>,
        y: Option<Distance>,
        z: Option<Distance>,
        f: Option<Speed>,
    ) -> Result<core::time::Duration, StepperError> {
        if let Some(feedrate) = f {
            self.config.feedrate = feedrate;
        }
        let feedrate = self.config.feedrate * self.config.feedrate_multiplier;
        let x = match x {
            Some(v) => v,
            None => no_move(&self.x_stepper, self.config.positioning),
        };

        let y = match y {
            Some(v) => v,
            None => no_move(&self.y_stepper, self.config.positioning),
        };

        let z = match z {
            Some(v) => v,
            None => no_move(&self.z_stepper, self.config.positioning),
        };

        let dst = Vector3D::new(x, y, z);

        linear_move_3d::<P, T, I>(
            (
                &mut self.x_stepper,
                &mut self.y_stepper,
                &mut self.z_stepper,
            ),
            dst,
            feedrate,
            self.config.positioning,
            (
                &mut self.endstops.0,
                &mut self.endstops.1,
                &mut self.endstops.2,
            ),
        )
        .await
    }

    async fn g1(
        &mut self,
        x: Option<Distance>,
        y: Option<Distance>,
        z: Option<Distance>,
        e: Option<Distance>,
        f: Option<Speed>,
    ) -> Result<core::time::Duration, StepperError> {
        if let Some(feedrate) = f {
            self.config.feedrate = feedrate;
        }
        let feedrate = self.config.feedrate * self.config.feedrate_multiplier;
        let x = match x {
            Some(v) => v,
            None => no_move(&self.x_stepper, self.config.positioning),
        };

        let y = match y {
            Some(v) => v,
            None => no_move(&self.y_stepper, self.config.positioning),
        };

        let z = match z {
            Some(v) => v,
            None => no_move(&self.z_stepper, self.config.positioning),
        };

        let e = match e {
            Some(v) => v,
            None => no_move(&self.e_stepper, self.config.positioning),
        };

        let dst = Vector3D::new(x, y, z);

        linear_move_3d_e::<P, T, I>(
            (
                &mut self.x_stepper,
                &mut self.y_stepper,
                &mut self.z_stepper,
                &mut self.e_stepper,
            ),
            dst,
            feedrate,
            e,
            self.config.positioning,
            (
                &mut self.endstops.0,
                &mut self.endstops.1,
                &mut self.endstops.2,
                &mut self.endstops.3,
            ),
        )
        .await
    }

    /**
     * clockwise arc move
     * IJ form:
     * - i or j is required. Omitting both will throw an error
     * - x and y can be omitted to do a complete circle
     *
     * R form:
     * - x or y is required. Omitting both will throw an error
     * - x or y must differ from the current xy position
     *
     * mixing i or j with r will throw an error
     *  
     */
    async fn g2_3(
        &mut self,
        x: Option<Distance>,
        y: Option<Distance>,
        z: Option<Distance>,
        e: Option<Distance>,
        f: Option<Speed>,
        i: Option<Distance>,
        j: Option<Distance>,
        r: Option<Distance>,
        d: RotationDirection,
    ) -> Result<core::time::Duration, StepperError> {
        match (i, j, r) {
            (Some(_), Some(_), Some(_))
            | (None, None, None)
            | (Some(_), None, Some(_))
            | (None, Some(_), Some(_)) => return Err(StepperError::MoveNotValid),
            _ => (),
        }

        if let Some(feedrate) = f {
            self.config.feedrate = feedrate;
        }
        let feedrate = self.config.feedrate * self.config.feedrate_multiplier;

        let z = match z {
            Some(v) => v,
            None => no_move(&self.z_stepper, Positioning::Absolute),
        };

        let e = match e {
            Some(v) => v,
            None => no_move(&self.z_stepper, Positioning::Relative),
        };

        if i.is_some() || j.is_some() {
            let x = match x {
                Some(v) => v,
                None => no_move(&self.x_stepper, Positioning::Absolute),
            };

            let y = match y {
                Some(v) => v,
                None => no_move(&self.y_stepper, Positioning::Absolute),
            };

            let dst = Vector3D::new(x, y, z);

            let i = match i {
                Some(v) => v,
                None => Distance::from_millimeters(0f64),
            };

            let j = match j {
                Some(v) => v,
                None => Distance::from_millimeters(0f64),
            };

            let offset_from_center = Vector2D::new(i, j);
            return arc_move_3d_e_offset_from_center::<P, T, I>(
                (
                    &mut self.x_stepper,
                    &mut self.y_stepper,
                    &mut self.z_stepper,
                    &mut self.e_stepper,
                ),
                dst,
                offset_from_center,
                feedrate,
                d,
                e,
                self.config.arc_unit_length,
                (
                    &mut self.endstops.0,
                    &mut self.endstops.1,
                    &mut self.endstops.2,
                    &mut self.endstops.3,
                ),
            )
            .await;
        }

        if let Some(r) = r {
            if x.is_none() && y.is_none() {
                return Err(StepperError::MoveNotValid);
            }

            let x = match x {
                Some(v) => v,
                None => no_move(&self.x_stepper, Positioning::Absolute),
            };

            let y = match y {
                Some(v) => v,
                None => no_move(&self.y_stepper, Positioning::Absolute),
            };

            let dst = Vector3D::new(x, y, z);

            return arc_move_3d_e_radius::<P, T, I>(
                (
                    &mut self.x_stepper,
                    &mut self.y_stepper,
                    &mut self.z_stepper,
                    &mut self.e_stepper,
                ),
                dst,
                r,
                feedrate,
                d,
                e,
                self.config.arc_unit_length,
                (
                    &mut self.endstops.0,
                    &mut self.endstops.1,
                    &mut self.endstops.2,
                    &mut self.endstops.3,
                ),
            )
            .await;
        }

        Err(StepperError::MoveNotValid)
    }

    async fn g2(
        &mut self,
        x: Option<Distance>,
        y: Option<Distance>,
        z: Option<Distance>,
        e: Option<Distance>,
        f: Option<Speed>,
        i: Option<Distance>,
        j: Option<Distance>,
        r: Option<Distance>,
    ) -> Result<core::time::Duration, StepperError> {
        self.g2_3(x, y, z, e, f, i, j, r, RotationDirection::Clockwise)
            .await
    }

    async fn g3(
        &mut self,
        x: Option<Distance>,
        y: Option<Distance>,
        z: Option<Distance>,
        e: Option<Distance>,
        f: Option<Speed>,
        i: Option<Distance>,
        j: Option<Distance>,
        r: Option<Distance>,
    ) -> Result<core::time::Duration, StepperError> {
        self.g2_3(x, y, z, e, f, i, j, r, RotationDirection::CounterClockwise)
            .await
    }

    // retract
    async fn g10(&mut self) -> Result<core::time::Duration, StepperError> {
        retract::<P, T, I>(
            &mut self.e_stepper,
            &mut self.z_stepper,
            self.config.retraction.feedrate,
            self.config.retraction.length,
            self.config.retraction.z_lift,
            (&mut self.endstops.2, &mut self.endstops.3),
        )
        .await
    }

    // recover
    async fn g11(&mut self) -> Result<core::time::Duration, StepperError> {
        let e_destination = self.e_stepper.get_position() + self.config.recover.length;
        linear_move_to::<P, T, I>(
            &mut self.e_stepper,
            e_destination,
            self.config.recover.feedrate,
            &mut self.endstops.3,
        )
        .await
    }

    // auto home
    async fn g28(
        &mut self,
        enabled: (bool, bool, bool),
    ) -> Result<core::time::Duration, StepperError> {
        let mut duration = Duration::ZERO;
        if enabled.0 {
            let e = self.endstops.0.as_ref().ok_or(StepperError::MoveNotValid)?;
            duration += auto_home::<_, _, T, _>(&mut self.x_stepper, e).await?;
        }
        if enabled.1 {
            let e = self.endstops.1.as_ref().ok_or(StepperError::MoveNotValid)?;
            duration += auto_home::<_, _, T, _>(&mut self.y_stepper, e).await?;
        }
        if enabled.2 {
            let e = self.endstops.2.as_ref().ok_or(StepperError::MoveNotValid)?;
            duration += auto_home::<_, _, T, _>(&mut self.z_stepper, e).await?;
        }
        Ok(duration)
    }
}
