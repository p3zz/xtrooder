use defmt::info;
use embassy_stm32::adc::{Adc, AdcPin, Instance, Resolution, SampleTime};
use embassy_stm32::gpio::Pin;
use embassy_stm32::Peripheral;
use embassy_time::Delay;
use micromath::F32Ext;

use math::temperature::Temperature;

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
    r_series: f64,
    b: Temperature,
}

impl<'a, T, P> Thermistor<'a, T, P>
where
    T: Instance,
    P: AdcPin<T> + Pin,
{
    pub fn new(
        adc_peri: impl Peripheral<P = T> + 'a,
        read_pin: P,
        resolution: Resolution,
        r0: f64,
        r_series: f64,
        b: Temperature,
    ) -> Thermistor<'a, T, P> {
        let mut adc = Adc::new(adc_peri, &mut Delay);
        adc.set_sample_time(SampleTime::CYCLES32_5);
        adc.set_resolution(resolution);
        Thermistor {
            adc,
            read_pin,
            resolution,
            r0,
            r_series,
            b,
        }
    }

    pub fn read_temperature(&mut self) -> Temperature {
        let sample = self.adc.read(&mut self.read_pin) as f64;
        info!("sample: {}", sample);
        compute_temperature(
            sample,
            self.resolution,
            Temperature::from_celsius(25.0),
            self.b,
            self.r0,
            self.r_series,
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
        _ => 0.0,
    }
}

// https://circuitdigest.com/microcontroller-projects/interfacing-Thermistor-with-arduino
// https://www.petervis.com/electronics%20guides/calculators/thermistor/thermistor.html
// Steinhart–Hart equation simplified for ntc thermistors
fn compute_temperature(
    sample: f64,
    resolution: Resolution,
    t0: Temperature,
    b: Temperature,
    r0: f64,
    r_series: f64,
) -> Temperature {
    let max_sample = get_steps(resolution) - 1.0;
    let r_ntc = r_series * (max_sample / sample - 1.0);
    info!("R: {}", r_ntc);
    let val_inv =
        (1.0 / t0.to_kelvin()) + (1.0 / b.to_kelvin()) * (((r_ntc / r0) as f32).ln() as f64);
    Temperature::from_kelvin(1.0 / val_inv)
}
