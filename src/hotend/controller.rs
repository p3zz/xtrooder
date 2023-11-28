use embassy_stm32::{pwm::CaptureCompare16bitInstance, adc::{AdcPin, Instance}, gpio::Pin};
use core::time::Duration;
use pid_lite::Controller;

use super::{heater::Heater, thermistor::Thermistor};

pub struct Hotend<'l, H, I, P>
where
    I: Instance,
    P: AdcPin<I> + Pin,
    H: CaptureCompare16bitInstance,
{
    heater: Heater<'l, H>,
    thermistor: Thermistor<'l, I, P>,
    pid: Controller
}

impl <'l, H, I, P> Hotend <'l, H, I, P>
where
    I: Instance,
    P: AdcPin<I> + Pin,
    H: CaptureCompare16bitInstance,
{
    pub fn new(heater: Heater<'l, H>, thermistor: Thermistor<'l, I, P>) -> Hotend<'l, H, I, P>{
        let pid = Controller::new(80.0, 0.20, 0.0, 0.0);
        Hotend { heater, thermistor, pid }
    }

    fn set_target(&mut self, target: f64){
        self.pid.set_target(target);
    }

    fn update(&mut self, dt: Duration){
        let tmp = self.thermistor.read_temperature();
        let new_value = self.pid.update_elapsed(tmp.to_celsius(), dt);
        // TODO set heater value
    }

}