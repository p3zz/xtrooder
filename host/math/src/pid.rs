use core::time::Duration;

pub struct PID {
    kp: f64,
    ki: f64,
    kd: f64,
    target: Option<f64>,
    prev_error: f64,
    integral: f64,
    bounds: Option<(f64, f64)>,
}

impl PID {
    pub fn new(kp: f64, ki: f64, kd: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            prev_error: 0.0,
            integral: 0.0,
            bounds: None,
            target: None,
        }
    }

    pub fn set_target(&mut self, target: f64) {
        self.target = Some(target);
    }

    pub fn reset_target(&mut self) {
        self.target = None;
    }

    pub fn get_target(&self) -> Option<f64> {
        self.target
    }

    pub fn set_output_bounds(&mut self, min: f64, max: f64) {
        self.bounds = Some((min, max));
    }

    pub fn update(&mut self, current: f64, dt: Duration) -> Result<f64, ()> {
        if self.target.is_none() {
            return Err(());
        }
        let target = self.target.unwrap();
        let error = target - current;

        // Proportional term
        let proportional = self.kp * error;

        // Integral term (only update if within output bounds)
        let out = proportional + self.ki * self.integral;
        if let Some(bounds) = self.bounds {
            if out >= bounds.0 && out <= bounds.1 {
                self.integral += error * dt.as_secs_f64();
            }
        } else {
            self.integral += error * dt.as_secs_f64();
        }

        // Derivative term
        let derivative = (error - self.prev_error) / dt.as_secs_f64();
        self.prev_error = error;

        // Compute total output
        let mut output = out + self.kd * derivative;
        if let Some(bounds) = self.bounds {
            output = output.clamp(bounds.0, bounds.1);
        }

        Ok(output)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid() {
        let elapsed = Duration::from_millis(40);
        let mut pid = PID::new(30.0, 0.0, 3.0);
        pid.set_target(30.0);
        let res = pid.update(20.0, elapsed);
        assert!(res.is_ok());
        assert_eq!(1050.0, res.unwrap());
        assert_eq!(pid.prev_error, 10.0);
        assert_eq!(pid.integral, 0.4);
    }
}
