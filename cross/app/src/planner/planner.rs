use embassy_time::Timer;
use math::common::RotationDirection;

use math::distance::{Distance, DistanceUnit};
use math::speed::Speed;
use math::vector::{Vector2D, Vector3D};
use parser::parser::{GCodeParser, GCommand};
use stepper::motion::{
    arc_move_3d_e_offset_from_center, arc_move_3d_e_radius, linear_move_3d, linear_move_3d_e,
    no_move, Positioning,
};
use stepper::stepper::{StatefulOutputPin, Stepper, StepperError, TimerTrait};

struct StepperTimer {}

impl TimerTrait for StepperTimer {
    async fn after(duration: core::time::Duration) {
        let duration = embassy_time::Duration::from_micros(duration.as_micros() as u64);
        Timer::after(duration).await
    }
}
// we need to have a triple(s, d, T) for every stepper
pub struct Planner<P: StatefulOutputPin> {
    feedrate: Speed,
    positioning: Positioning,
    x_stepper: Stepper<P>,
    y_stepper: Stepper<P>,
    z_stepper: Stepper<P>,
    e_stepper: Stepper<P>,
    parser: GCodeParser,
}
impl<P: StatefulOutputPin> Planner<P> {
    pub fn new(
        x_stepper: Stepper<P>,
        y_stepper: Stepper<P>,
        z_stepper: Stepper<P>,
        e_stepper: Stepper<P>,
    ) -> Self {
        Planner {
            x_stepper,
            y_stepper,
            z_stepper,
            e_stepper,
            feedrate: Speed::from_mm_per_second(0.0),
            positioning: Positioning::Absolute,
            parser: GCodeParser::new(),
        }
    }

    pub async fn execute(&mut self, command: GCommand) -> Result<(), StepperError> {
        match command {
            GCommand::G0 { x, y, z, f } => self.g0(x, y, z, f).await.map(|_| ()),
            GCommand::G1 { x, y, z, e, f } => self.g1(x, y, z, e, f).await.map(|_| ()),
            GCommand::G2 {
                x,
                y,
                z,
                e,
                f,
                i,
                j,
                r,
            } => self.g2(x, y, z, e, f, i, j, r).await.map(|_| ()),
            GCommand::G3 {
                x,
                y,
                z,
                e,
                f,
                i,
                j,
                r,
            } => self.g3(x, y, z, e, f, i, j, r).await.map(|_| ()),
            GCommand::G20 => {
                self.g20();
                Ok(())
            }
            GCommand::G21 => {
                self.g21();
                Ok(())
            }
            GCommand::G90 => {
                self.g90();
                Ok(())
            }
            GCommand::G91 => {
                self.g91();
                Ok(())
            }
            GCommand::M104 { s } => todo!(),
            GCommand::G4 { p, s } => {
                self.g4(p, s).await;
                Ok(())
            }
            GCommand::M149 => todo!(),
        }
    }

    async fn g4(&mut self, p: Option<core::time::Duration>, s: Option<core::time::Duration>) {
        let d = match (p, s) {
            (None, None) => None,
            (None, Some(_)) | (Some(_), Some(_)) => s,
            (Some(_), None) => p,
        };
        if let Some(duration) = d {
            let t = embassy_time::Duration::from_millis(duration.as_millis() as u64);
            Timer::after(t).await
        }
    }

    fn g20(&mut self) {
        self.parser.set_distance_unit(DistanceUnit::Inch);
    }

    fn g21(&mut self) {
        self.parser.set_distance_unit(DistanceUnit::Millimeter);
    }

    fn g90(&mut self) {
        self.positioning = Positioning::Absolute;
    }

    fn g91(&mut self) {
        self.positioning = Positioning::Relative;
    }

    pub async fn g0(
        &mut self,
        x: Option<Distance>,
        y: Option<Distance>,
        z: Option<Distance>,
        f: Option<Speed>,
    ) -> Result<core::time::Duration, StepperError> {
        if let Some(feedrate) = f {
            self.feedrate = feedrate;
        }
        let x = match x {
            Some(v) => v,
            None => no_move(&self.x_stepper, self.positioning)?,
        };

        let y = match y {
            Some(v) => v,
            None => no_move(&self.y_stepper, self.positioning)?,
        };

        let z = match z {
            Some(v) => v,
            None => no_move(&self.z_stepper, self.positioning)?,
        };

        let dst = Vector3D::new(x, y, z);

        linear_move_3d::<P, StepperTimer>(
            &mut self.x_stepper,
            &mut self.y_stepper,
            &mut self.z_stepper,
            dst,
            self.feedrate,
            self.positioning,
        )
        .await
    }

    pub async fn g1(
        &mut self,
        x: Option<Distance>,
        y: Option<Distance>,
        z: Option<Distance>,
        e: Option<Distance>,
        f: Option<Speed>,
    ) -> Result<core::time::Duration, StepperError> {
        if let Some(feedrate) = f {
            self.feedrate = feedrate;
        }
        let x = match x {
            Some(v) => v,
            None => no_move(&self.x_stepper, self.positioning)?,
        };

        let y = match y {
            Some(v) => v,
            None => no_move(&self.y_stepper, self.positioning)?,
        };

        let z = match z {
            Some(v) => v,
            None => no_move(&self.z_stepper, self.positioning)?,
        };

        let e = match e {
            Some(v) => v,
            None => no_move(&self.e_stepper, self.positioning)?,
        };

        let dst = Vector3D::new(x, y, z);

        linear_move_3d_e::<P, StepperTimer>(
            &mut self.x_stepper,
            &mut self.y_stepper,
            &mut self.z_stepper,
            &mut self.e_stepper,
            dst,
            self.feedrate,
            e,
            self.positioning,
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
            self.feedrate = feedrate;
        }

        let z = match z {
            Some(v) => v,
            None => no_move(&self.z_stepper, Positioning::Absolute)?,
        };

        let e = match e {
            Some(v) => v,
            None => no_move(&self.z_stepper, Positioning::Relative)?,
        };

        if i.is_some() || j.is_some() {
            let x = match x {
                Some(v) => v,
                None => no_move(&self.x_stepper, Positioning::Absolute)?,
            };

            let y = match y {
                Some(v) => v,
                None => no_move(&self.y_stepper, Positioning::Absolute)?,
            };

            let dst = Vector3D::new(x, y, z);

            let i = match i {
                Some(v) => v,
                None => Distance::from_mm(0f64),
            };

            let j = match j {
                Some(v) => v,
                None => Distance::from_mm(0f64),
            };

            let offset_from_center = Vector2D::new(i, j);
            return arc_move_3d_e_offset_from_center::<P, StepperTimer>(
                &mut self.x_stepper,
                &mut self.y_stepper,
                &mut self.z_stepper,
                &mut self.e_stepper,
                dst,
                offset_from_center,
                self.feedrate,
                d,
                e,
            )
            .await;
        }

        if r.is_some() {
            if x.is_none() && y.is_none() {
                return Err(StepperError::MoveNotValid);
            }

            let x = match x {
                Some(v) => v,
                None => no_move(&self.x_stepper, Positioning::Absolute)?,
            };

            let y = match y {
                Some(v) => v,
                None => no_move(&self.y_stepper, Positioning::Absolute)?,
            };

            let dst = Vector3D::new(x, y, z);

            let r = r.unwrap();

            return arc_move_3d_e_radius::<P, StepperTimer>(
                &mut self.x_stepper,
                &mut self.y_stepper,
                &mut self.z_stepper,
                &mut self.e_stepper,
                dst,
                r,
                self.feedrate,
                d,
                e,
            )
            .await;
        }

        Err(StepperError::MoveNotValid)
    }

    pub async fn g2(
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

    pub async fn g3(
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
}
