use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;
use embedded_time::duration::*;

struct Stepper<O, T>{
    step: O,
    dir: O,
    steps_per_revolution: u32,
    timer: T,
    step_delay: Microseconds
}

impl <O, T> Stepper<O, T>
where O: OutputPin, T: CountDown<Time = Microseconds>,
{
    pub fn new(step: O, dir: O, steps_per_revolution: u32, timer: T) -> Stepper<O,T>{
        Stepper{
            step, dir, steps_per_revolution, timer, step_delay: sps_from_rpm(1, steps_per_revolution)
        }
    }

    pub fn set_speed(&mut self, rpm: u32) -> (){
        self.step_delay = sps_from_rpm(rpm, self.steps_per_revolution);
    }

    pub fn step(&mut self) -> (){
        let _ = self.step.set_high();
        self.timer.start(self.step_delay);
        self.timer.wait().unwrap();
        let _ = self.step.set_low();
    }
}

// get second per step from round per minute
fn sps_from_rpm(rpm: u32, steps_per_revolution: u32) -> Microseconds<u32> {
    let rps = (rpm / 60) as f32;
    let spr = 1.0 / rps;
    let sps = spr/(steps_per_revolution as f32);
    let microsps = (sps * 1_000_000.0) as u32;
    return Microseconds(microsps);
}
