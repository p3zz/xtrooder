use core::time::Duration as CoreDuration;

use embassy_stm32::{
    adc::{AdcPin, Instance},
    gpio::Pin, timer::CaptureCompare16bitInstance,
    // pwm::CaptureCompare16bitInstance,
};
use embassy_time::Duration;
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
    pid: Controller,
}

impl<'l, H, I, P> Hotend<'l, H, I, P>
where
    I: Instance,
    P: AdcPin<I> + Pin,
    H: CaptureCompare16bitInstance,
{
    pub fn new(heater: Heater<'l, H>, thermistor: Thermistor<'l, I, P>) -> Hotend<'l, H, I, P> {
        let pid = Controller::new(0.0, 0.20, 0.0, 0.0);
        Hotend {
            heater,
            thermistor,
            pid,
        }
    }

    pub fn set_target_temperature(&mut self, temperature: f64) {
        self.pid.set_target(temperature);
    }

    pub fn update(&mut self, dt: Duration) {
        let tmp = self.thermistor.read_temperature();
        let new_value = self
            .pid
            .update_elapsed(tmp.to_celsius(), CoreDuration::from_millis(dt.as_millis()));
        self.heater.set_value(new_value)
    }
}
