use embassy_stm32::{
    adc::{Instance, RxDma},
    timer::GeneralInstance4Channel,
};
use embassy_time::Duration;
use math::measurements::Temperature;

use super::{heater::Heater, thermistor::Thermistor};

pub struct Hotend<'l, H, I, P>
where
    I: Instance,
    P: RxDma<I>,
    H: GeneralInstance4Channel,
{
    heater: Heater<'l, H>,
    thermistor: Thermistor<'l, I, P>,
}

impl<'l, H, I, P> Hotend<'l, H, I, P>
where
    I: Instance,
    P: RxDma<I>,
    H: GeneralInstance4Channel,
{
    pub fn new(heater: Heater<'l, H>, thermistor: Thermistor<'l, I, P>) -> Hotend<'l, H, I, P> {
        Hotend { heater, thermistor }
    }

    pub fn set_temperature(&mut self, temperature: Temperature) {
        self.heater.set_target_temperature(temperature);
    }

    pub async fn update(&mut self, dt: Duration) -> Result<u32, ()> {
        let curr_tmp = self.read_temperature().await;
        // info!("Temperature: {}", curr_tmp.to_celsius());
        self.heater.update(curr_tmp, dt)
    }

    pub async fn read_temperature(&mut self) -> Temperature {
        self.thermistor.read_temperature().await
    }
}
