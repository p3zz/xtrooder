use core::time::Duration;

use common::{MyAdc, MyPwm};
use math::measurements::Temperature;

use crate::{heater::Heater, thermistor::Thermistor};

pub struct ThermalActuator<'a, P: MyPwm, A: MyAdc>{
    heater: Heater<P>,
    thermistor: Thermistor<'a, A>
}

impl <'a, P: MyPwm, A: MyAdc> ThermalActuator<'a, P, A>{
    pub fn new(heater: Heater<P>, thermistor: Thermistor<'a, A>) -> Self{
        Self { heater, thermistor }
    }

    pub fn enable(&mut self, pwm: &mut P) {
        self.heater.enable(pwm);
    }

    pub fn disable(&mut self, pwm: &mut P) {
        self.heater.disable(pwm);
    }

    pub fn set_temperature(&mut self, temperature: Temperature) {
        self.heater.set_target_temperature(temperature);
    }

    pub async fn update(&mut self, dt: Duration, pwm: &mut P) -> Result<u64, ()> {
        let curr_tmp = self.read_temperature().await;
        // info!("Temperature: {}", curr_tmp.to_celsius());
        self.heater.update(curr_tmp, dt, pwm)
    }

    pub async fn read_temperature(&mut self) -> Temperature {
        self.thermistor.read_temperature().await
    }
        
}
