use embassy_stm32::timer::CaptureCompare16bitInstance;
use embassy_time::Timer;

use super::motion::{self, no_move, Positioning};
use crate::stepper::a4988::{Stepper, StepperError};
use core::time::Duration;
use math::distance::{Distance, DistanceUnit};
use math::speed::Speed;
use math::vector::{Vector2D, Vector3D};
use parser::parser::{GCodeParser, GCommand};

// we need to have a triple(s, d, T) for every stepper
pub struct Planner<'s, X, Y, Z, E> {
    feedrate: Speed,
    positioning: Positioning,
    x_stepper: Stepper<'s, X>,
    y_stepper: Stepper<'s, Y>,
    z_stepper: Stepper<'s, Z>,
    e_stepper: Stepper<'s, E>,
    parser: GCodeParser
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
            feedrate: Speed::from_mm_per_second(0.0),
            positioning: Positioning::Absolute,
            parser: GCodeParser::new()
        }
    }

    pub async fn execute(&mut self, command: GCommand) -> Result<(), StepperError> {
        match command {
            GCommand::G0 { x, y, z, f } => self.g0(x, y, z, f).await,
            GCommand::G1 { x, y, z, e, f } => self.g1(x, y, z, e, f).await,
            GCommand::G2 {
                x,
                y,
                z,
                e,
                f,
                i,
                j,
                r,
            } => todo!(),
            GCommand::G3 {
                x,
                y,
                z,
                e,
                f,
                i,
                j,
                r,
            } => todo!(),
            GCommand::G20 => Ok(self.g20()),
            GCommand::G21 => Ok(self.g21()),
            GCommand::G90 => Ok(self.g90()),
            GCommand::G91 => Ok(self.g91()),
            GCommand::M104 { s } => todo!(),
            GCommand::G4 { p, s } => Ok(self.g4(p, s).await),
            GCommand::M149 => todo!(),
        }
    }

    async fn g4(&mut self, p: Option<Duration>, s: Option<Duration>) {
        let d = match (p, s) {
            (None, None) => None,
            (None, Some(_)) | (Some(_), Some(_)) => s,
            (Some(_), None) => p,
        };
        if let Some(duration) = d {
            let t =  embassy_time::Duration::from_millis(duration.as_millis() as u64);
            Timer::after(t).await;
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
    ) -> Result<(), StepperError> {
        if let Some(feedrate) = f{
            self.feedrate = feedrate;    
        }
        let x = match x{
            Some(v) => v,
            None => no_move(&self.x_stepper, self.positioning),
        };

        let y = match y{
            Some(v) => v,
            None => no_move(&self.y_stepper, self.positioning),
        };
        
        let z = match z{
            Some(v) => v,
            None => no_move(&self.z_stepper, self.positioning),
        };

        let dst = Vector3D::new(x, y, z);

        motion::linear_move_3d(
            &mut self.x_stepper,
            &mut self.y_stepper, 
            &mut self.z_stepper, 
            dst, 
            self.feedrate, 
            self.positioning).await
    }

    pub async fn g1(
        &mut self,
        x: Option<Distance>,
        y: Option<Distance>,
        z: Option<Distance>,
        e: Option<Distance>,
        f: Option<Speed>,
    ) -> Result<(), StepperError> {
        if let Some(feedrate) = f{
            self.feedrate = feedrate;    
        }
        let x = match x{
            Some(v) => v,
            None => no_move(&self.x_stepper, self.positioning),
        };

        let y = match y{
            Some(v) => v,
            None => no_move(&self.y_stepper, self.positioning),
        };
        
        let z = match z{
            Some(v) => v,
            None => no_move(&self.z_stepper, self.positioning),
        };

        let e = match e{
            Some(v) => v,
            None => no_move(&self.e_stepper, self.positioning),
        };

        let dst = Vector3D::new(x, y, z);

        motion::linear_move_3d_e(
            &mut self.x_stepper,
            &mut self.y_stepper, 
            &mut self.z_stepper, 
            &mut self.e_stepper, 
            dst, 
            self.feedrate,
            e,
            self.positioning).await
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

     pub async fn g2(
        &mut self,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        e: Option<f64>,
        f: Option<f64>,
        i: Option<f64>,
        j: Option<f64>,
        r: Option<f64>,
    ) -> Result<(), StepperError> {
        match (i, j, r) {
            (Some(_), Some(_), Some(_))
            | (None, None, None)
            | (Some(_), None, Some(_))
            | (None, Some(_), Some(_)) => return Err(StepperError::MoveNotValid),
            _ => (),
        }




        todo!()
    }
}
