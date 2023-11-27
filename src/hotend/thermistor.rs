use embassy_stm32::adc::{Adc, SampleTime, Resolution, Instance, AdcPin};
use embassy_stm32::gpio::Pin;
use micromath::F32Ext;

use {defmt_rtt as _, panic_probe as _};

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
    resolution: Resolution
}

impl <'a, T, P> Thermistor<'a, T, P>
where T: Instance, P: AdcPin<T> + Pin {
    pub fn new(mut adc: Adc<'a, T>, read_pin: P, resolution: Resolution) -> Thermistor<'a, T, P>{
        adc.set_sample_time(SampleTime::Cycles32_5);
        adc.set_resolution(resolution);
        Thermistor{
            adc,
            read_pin,
            resolution
        }
    }

    pub fn read_temperature(&mut self) -> Temperature{
        let adc_res_n = get_resolution(self.resolution);
        let r0 = 10_000.0;
        let b = 3950.0;
        let room_temperature = Temperature::from_kelvin(298.15);

        let sample = self.adc.read(&mut self.read_pin) as f64;
        
        let r = r0 * ((adc_res_n - 1.0) / sample - 1.0);

        let temperature_val = 1.0 / ((1.0 / room_temperature.to_kelvin() as f32) + (1.0 / b) * ((r as f32 / r0 as f32).ln())) as f64;

        Temperature::from_kelvin(temperature_val)
    }
}

fn get_resolution(resolution: Resolution) -> f64{
    match resolution{
        Resolution::SixteenBit => 65536.0,
        Resolution::FourteenBit => 16384.0,
        Resolution::TwelveBit => 4096.0,
        Resolution::TenBit => 1024.0,
        Resolution::EightBit => 256.0,
    }
}