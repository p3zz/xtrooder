use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;

struct Stepper<O, T>{
    step: O,
    dir: O,
    steps_per_revolution: u32,
    timer: T,
}

impl <O, T> Stepper<O, T>
where O: OutputPin, T: CountDown,
{
    pub fn new(step: O, dir: O, steps_per_revolution: u32, timer: T) -> Stepper<O,T>{
        Stepper{
            step, dir, steps_per_revolution, timer
        }
    }
}
