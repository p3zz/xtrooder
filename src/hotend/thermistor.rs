use embassy_stm32::adc::{Adc, AdcPin, Instance, Resolution, SampleTime};
use embassy_stm32::gpio::Pin;
use micromath::F32Ext;

use crate::math::temperature::Temperature;

use {defmt_rtt as _, panic_probe as _};

/*
ADC value = R / (R + R0) * Vcc * resolution / Varef
Vcc: voltage reference of the board
Varef: voltage of the thermistor
*/

pub struct Thermistor<'a, T, P>
where
    T: Instance,
    P: AdcPin<T> + Pin,
{
    adc: Adc<'a, T>,
    read_pin: P,
    resolution: Resolution,
    r0: f64,
    b: Temperature,
}

impl<'a, T, P> Thermistor<'a, T, P>
where
    T: Instance,
    P: AdcPin<T> + Pin,
{
    pub fn new(
        mut adc: Adc<'a, T>,
        read_pin: P,
        resolution: Resolution,
        r0: f64,
        b: Temperature,
    ) -> Thermistor<'a, T, P> {
        adc.set_sample_time(SampleTime::CYCLES32_5);
        adc.set_resolution(resolution);
        Thermistor {
            adc,
            read_pin,
            resolution,
            r0,
            b,
        }
    }

    pub fn read_temperature(&mut self) -> Temperature {
        let sample = self.adc.read(&mut self.read_pin) as f64;
        compute_temperature(
            sample,
            self.resolution,
            Temperature::from_kelvin(298.15),
            self.b,
            self.r0,
        )
    }
}

fn get_steps(resolution: Resolution) -> f64 {
    match resolution {
        Resolution::BITS16 => 65536.0,
        Resolution::BITS14 => 16384.0,
        Resolution::BITS12 => 4096.0,
        Resolution::BITS10 => 1024.0,
        Resolution::BITS8 => 256.0,
        _ => 0.0
    }
}

// Steinhart–Hart equation simplified for ntc thermistors
fn compute_temperature(
    sample: f64,
    resolution: Resolution,
    t0: Temperature,
    b: Temperature,
    r0: f64,
) -> Temperature {
    let resolution_steps = get_steps(resolution);
    let r = r0 * ((resolution_steps - 1.0) / sample - 1.0);
    let val_inv = (1.0 / t0.to_kelvin()) + (1.0 / b.to_kelvin()) * (((r / r0) as f32).ln() as f64);
    Temperature::from_kelvin(1.0 / val_inv)
}
