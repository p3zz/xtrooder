use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;
// TODO add embedded_time for duration and time conversions

struct Stepper<O, T>{
    step: O,
    dir: O,
    steps_per_revolution: u32,
    timer: T,
    step_delay: f32
}

impl <O, T> Stepper<O, T>
where O: OutputPin, T: CountDown,
{
    pub fn new(step: O, dir: O, steps_per_revolution: u32, timer: T) -> Stepper<O,T>{
        Stepper{
            step, dir, steps_per_revolution, timer, step_delay: 1000.0
        }
    }

    pub fn set_speed(&mut self, rpm: u32) -> (){
        let rps = (rpm / 60) as f32;
        let spr = 1.0 / rps;
        self.step_delay = spr / (self.steps_per_revolution as f32);
    }

    pub fn step(&mut self) -> (){
        self.timer.start();
        self.step.set_high();
        self.timer.wait().unwrap();
        self.step.set_low();
    }
}
