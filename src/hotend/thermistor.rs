use embassy_stm32::adc::{Adc, SampleTime, Resolution, Instance, AdcPin};
use embassy_stm32::gpio::Pin;
use micromath::F32Ext;

use {defmt_rtt as _, panic_probe as _};

#[derive(Clone, Copy)]
struct Temperature{
    // unit: C (celsius)
    value: f64
}

impl Temperature{
    pub fn from_celsius(value: f64) -> Temperature{
        Temperature { value }
    }

    pub fn from_kelvin(value: f64) -> Temperature{
        Temperature { value: value - 273.15 } 
    }

    pub fn to_kelvin(&self) -> f64 {
        return self.value + 273.15
    }

    pub fn to_celsius(&self) -> f64 {
        return self.value
    }
}

/*
ADC value = R / (R + R0) * Vcc * resolution / Varef
Vcc: voltage reference of the board
Varef: voltage of the thermistor
*/

struct Thermistor<'a, T, P>
where T: Instance, P: AdcPin<T> + Pin{
    adc: Adc<'a, T>,
    read_pin: P,
    resolution: Resolution,
    r0: f64,
    b: Temperature
}

impl <'a, T, P> Thermistor<'a, T, P>
where T: Instance, P: AdcPin<T> + Pin {
    pub fn new(mut adc: Adc<'a, T>, read_pin: P, resolution: Resolution, r0: f64, b: Temperature) -> Thermistor<'a, T, P>{
        adc.set_sample_time(SampleTime::Cycles32_5);
        adc.set_resolution(resolution);
        Thermistor{
            adc,
            read_pin,
            resolution,
            r0,
            b
        }
    }

    pub fn read_temperature(&mut self) -> Temperature{
        let sample = self.adc.read(&mut self.read_pin) as f64;        
        compute_temperature(sample, self.resolution, Temperature::from_kelvin(298.15), self.b, self.r0)
    }
}

fn get_steps(resolution: Resolution) -> f64{
    match resolution{
        Resolution::SixteenBit => 65536.0,
        Resolution::FourteenBit => 16384.0,
        Resolution::TwelveBit => 4096.0,
        Resolution::TenBit => 1024.0,
        Resolution::EightBit => 256.0,
    }
}

// Steinhart–Hart equation simplified for ntc thermistors
fn compute_temperature(sample: f64, resolution: Resolution, t0: Temperature, b: Temperature, r0: f64) -> Temperature {
    let resolution_steps = get_steps(resolution);
    let r = r0 * ((resolution_steps - 1.0) / sample - 1.0);
    let val_inv = (1.0 / t0.to_kelvin()) + (1.0 / b.to_kelvin()) * (((r / r0) as f32).ln() as f64);
    Temperature::from_kelvin(1.0 / val_inv)
}