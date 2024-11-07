use embassy_stm32::{
    adc::{Instance, RxDma},
    timer::{simple_pwm::SimplePwm, GeneralInstance4Channel},
};
use embassy_time::Duration;
use math::measurements::Temperature;

use super::{heater::Heater, thermistor::Thermistor};

pub struct Hotend<'l, I, P>
where
    I: Instance,
    P: RxDma<I>,
{
    heater: Heater,
    thermistor: Thermistor<'l, I, P>,
}

impl<'l, I, P> Hotend<'l, I, P>
where
    I: Instance,
    P: RxDma<I>,
{
    pub fn new(heater: Heater, thermistor: Thermistor<'l, I, P>) -> Hotend<'l, I, P> {
        Hotend { heater, thermistor }
    }

    pub fn set_temperature(&mut self, temperature: Temperature) {
        self.heater.set_target_temperature(temperature);
    }

    pub async fn update<T: GeneralInstance4Channel>(
        &mut self,
        dt: Duration,
        pwm: &mut SimplePwm<'_, T>,
    ) -> Result<u32, ()> {
        let curr_tmp = self.read_temperature().await;
        // info!("Temperature: {}", curr_tmp.to_celsius());
        self.heater.update(curr_tmp, dt, pwm)
    }

    pub async fn read_temperature(&mut self) -> Temperature {
        self.thermistor.read_temperature().await
    }
}
