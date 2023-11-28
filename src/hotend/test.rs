use embassy_stm32::{adc::{Adc, AdcPin, Resolution}, fmc::A0Pin, gpio::Input};
use embassy_time::Delay;

use crate::stepper::units::Temperature;

use super::thermistor::Thermistor;

fn test(){
    let p = embassy_stm32::init(Default::default());
    let mut adc = Adc::new(p.ADC1, &mut Delay);
    let mut t = Thermistor::new(adc, p.PA0, Resolution::SixteenBit, 10_000.0, Temperature::from_kelvin(3950.0));
    let temp = t.read_temperature();
}