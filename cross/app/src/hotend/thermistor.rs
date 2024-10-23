use embassy_stm32::adc::{Adc, AnyAdcChannel, Instance, Resolution, RxDma, SampleTime};
use embassy_stm32::Peripheral;
use math::resistance::Resistance;
use micromath::F32Ext;

use math::temperature::Temperature;

pub type DmaBufType = [u16; 1];

/*
ADC value = R / (R + R0) * Vcc * resolution / Varef
Vcc: voltage reference of the board
Varef: voltage of the thermistor
*/

pub struct Thermistor<'a, T, D>
where
    T: Instance,
    D: RxDma<T>,
{
    adc: Adc<'a, T>,
    dma_peri: D,
    read_pin: AnyAdcChannel<T>,
    resolution: Resolution,
    r0: Resistance,
    r_series: Resistance,
    b: Temperature,
    readings: &'a mut DmaBufType,
}

impl<'a, T, D> Thermistor<'a, T, D>
where
    T: Instance,
    D: RxDma<T>,
{
    pub fn new(
        adc_peri: impl Peripheral<P = T> + 'a,
        dma_peri: D,
        read_pin: AnyAdcChannel<T>,
        resolution: Resolution,
        r0: Resistance,
        r_series: Resistance,
        b: Temperature,
        readings: &'a mut DmaBufType,
    ) -> Thermistor<'a, T, D> {
        let mut adc = Adc::new(adc_peri);
        adc.set_sample_time(SampleTime::CYCLES32_5);
        adc.set_resolution(resolution);
        Thermistor {
            adc,
            read_pin,
            dma_peri,
            resolution,
            r0,
            r_series,
            b,
            readings,
        }
    }

    pub async fn read_temperature(&mut self) -> Temperature {
        let readings = self.readings.as_mut();
        self.adc
            .read(
                &mut self.dma_peri,
                [(&mut self.read_pin, SampleTime::CYCLES32_5)].into_iter(),
                readings,
            )
            .await;
        compute_temperature(
            readings[0] as usize,
            self.resolution,
            Temperature::from_celsius(25.0),
            self.b,
            self.r0,
            self.r_series,
        )
    }
}

fn get_steps(resolution: Resolution) -> usize {
    match resolution {
        Resolution::BITS16 => 1 << 16,
        Resolution::BITS14 => 1 << 14,
        Resolution::BITS12 => 1 << 12,
        Resolution::BITS10 => 1 << 10,
        Resolution::BITS8 => 1 << 8,
        _ => 0,
    }
}

// https://circuitdigest.com/microcontroller-projects/interfacing-Thermistor-with-arduino
// https://www.petervis.com/electronics%20guides/calculators/thermistor/thermistor.html
// Steinhart–Hart equation simplified for ntc thermistors
fn compute_temperature(
    sample: usize,
    resolution: Resolution,
    t0: Temperature,
    b: Temperature,
    r0: Resistance,
    r_series: Resistance,
) -> Temperature {
    let max_sample = get_steps(resolution) - 1;
    let r_ntc = Resistance::from_ohm(r_series.as_ohm() * (max_sample / sample - 1));
    let val_inv = (1.0 / t0.to_kelvin())
        + (1.0 / b.to_kelvin()) * (((r_ntc.as_ohm() / r0.as_ohm()) as f32).ln() as f64);
    Temperature::from_kelvin(1.0 / val_inv)
}
