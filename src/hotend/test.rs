use embassy_stm32::adc::{Adc, Resolution};
use embassy_time::Delay;

use crate::math::temperature::Temperature;

use super::thermistor::Thermistor;

fn test() {
    let p = embassy_stm32::init(Default::default());
    let adc = Adc::new(p.ADC1, &mut Delay);
    let mut t = Thermistor::new(
        adc,
        p.PA0,
        Resolution::SixteenBit,
        10_000.0,
        Temperature::from_kelvin(3950.0),
    );
    let temp = t.read_temperature();
    temp.to_celsius();
}
