use embassy_time::Timer;
use math::common::RotationDirection;

use math::computable::Computable;
use math::distance::Distance;
use math::speed::Speed;
use math::vector::{Vector2D, Vector3D};
use parser::gcode::GCommand;
use stepper::motion::{
    arc_move_3d_e_offset_from_center, arc_move_3d_e_radius, auto_home_3d, linear_move_3d, linear_move_3d_e, linear_move_for_3d, linear_move_to, no_move, retract, Positioning
};
use stepper::stepper::{StatefulOutputPin, Stepper, StepperError, StepperInputPin, TimerTrait};

struct StepperTimer {}

impl TimerTrait for StepperTimer {
    async fn after(duration: core::time::Duration) {
        let duration = embassy_time::Duration::from_micros(duration.as_micros() as u64);
        Timer::after(duration).await
    }
}

pub struct Planner<P: StatefulOutputPin> {
    retraction_feedrate: Option<Speed>,
    retraction_length: Option<Distance>,
    retraction_z_lift: Option<Distance>,
    recover_feedrate: Option<Speed>,
    recover_length: Option<Distance>,
    feedrate: Speed,
    positioning: Positioning,
    x_stepper: Stepper<P>,
    y_stepper: Stepper<P>,
    z_stepper: Stepper<P>,
    e_stepper: Stepper<P>,
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
            recover_feedrate: None,
            retraction_length: None,
            retraction_z_lift: None,
            recover_length: None,
            retraction_feedrate: None,
        }
    }

    pub fn get_x_position(&self) -> Result<Distance, StepperError> {
        self.x_stepper.get_position()
    }

    pub fn get_y_position(&self) -> Result<Distance, StepperError> {
        self.y_stepper.get_position()
    }

    pub fn get_z_position(&self) -> Result<Distance, StepperError> {
        self.z_stepper.get_position()
    }

    pub fn get_e_position(&self) -> Result<Distance, StepperError> {
        self.e_stepper.get_position()
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
            GCommand::G90 => {
                self.g90();
                Ok(())
            }
            GCommand::G91 => {
                self.g91();
                Ok(())
            }
            GCommand::G4 { p, s } => {
                self.g4(p, s).await;
                Ok(())
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
            let t = embassy_time::Duration::from_millis(duration.as_millis() as u64);
            Timer::after(t).await
        }
    }

    fn g90(&mut self) {
        self.positioning = Positioning::Absolute;
    }

    fn g91(&mut self) {
        self.positioning = Positioning::Relative;
    }

    // firmware retraction settings
    fn m207(&mut self, f: Speed, s: Distance, z: Distance){
        self.retraction_feedrate.replace(f);
        self.retraction_length.replace(s);
        self.retraction_z_lift.replace(z);
    }

    fn m208(&mut self, f: Speed, s: Distance){
        if self.retraction_length.is_none(){
            return;
        }
        self.retraction_feedrate.replace(f);
        self.retraction_length.replace(s.add(&self.recover_length.unwrap()));
    }

    async fn g0(
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

    async fn g1(
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
    async fn g10(&mut self) -> Result<core::time::Duration, StepperError>{
        let retraction_length = self.retraction_length.ok_or(StepperError::MoveNotValid)?;
        let retraction_speed = self.retraction_feedrate.ok_or(StepperError::MoveNotValid)?;
        let retraction_z_lift = self.retraction_z_lift.ok_or(StepperError::MoveNotValid)?;
        retract::<P, StepperTimer>(&mut self.e_stepper, &mut self.z_stepper, retraction_speed, retraction_length, retraction_z_lift).await
    }

    // recover
    async fn g11(&mut self) -> Result<core::time::Duration, StepperError>{
        let recover_length = self.recover_length.ok_or(StepperError::MoveNotValid)?;
        let recover_speed = self.recover_feedrate.ok_or(StepperError::MoveNotValid)?;
        let e_destination = self.e_stepper.get_position()?.add(&recover_length);
        linear_move_to::<P, StepperTimer>(&mut self.e_stepper, e_destination, recover_speed).await
    }

    // auto home
    async fn g28<I: StepperInputPin>(&mut self, x_button: &I, y_button: &I, z_button: &I) -> Result<core::time::Duration, StepperError>{
        auto_home_3d::<I, P, StepperTimer>(&mut self.x_stepper, &mut self.y_stepper, &mut self.z_stepper, x_button, y_button, z_button).await
    }
}
