use defmt::info;
use embassy_stm32::{
    adc::{AdcPin, Instance},
    gpio::Pin,
    timer::CaptureCompare16bitInstance,
};
use embassy_time::Duration;
use math::temperature::Temperature;

use super::{heater::Heater, thermistor::Thermistor};

pub struct Hotend<'l, H, I, P>
where
    I: Instance,
    P: AdcPin<I> + Pin,
    H: CaptureCompare16bitInstance,
{
    heater: Heater<'l, H>,
    thermistor: Thermistor<'l, I, P>,
}

impl<'l, H, I, P> Hotend<'l, H, I, P>
where
    I: Instance,
    P: AdcPin<I> + Pin,
    H: CaptureCompare16bitInstance,
{
    pub fn new(heater: Heater<'l, H>, thermistor: Thermistor<'l, I, P>) -> Hotend<'l, H, I, P> {
        Hotend { heater, thermistor }
    }

    pub fn set_temperature(&mut self, temperature: Temperature) {
        self.heater.set_target_temperature(temperature);
    }

    pub fn update(&mut self, dt: Duration) {
        let curr_tmp = self.thermistor.read_temperature();
        info!("Temperature: {}", curr_tmp.to_celsius());
        self.heater.update(curr_tmp, dt);
    }
}
