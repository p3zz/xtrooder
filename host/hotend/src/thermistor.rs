use core::array::IntoIter;
use core::future::Future;

use common::MyAdc;
use math::common::compute_ntf_thermistor_temperature;
use math::measurements::{Resistance, Temperature};
use math::Resolution;

pub type DmaBufType = [u16; 1];

/*
ADC value = R / (R + R0) * Vcc * resolution / Varef
Vcc: voltage reference of the board
Varef: voltage of the thermistor
*/

pub struct Thermistor<'a, A: MyAdc>{
    adc: A,
    dma_peri: A::DmaType,
    read_pin: A::PinType,
    readings: &'a mut DmaBufType,
    r0: Resistance,
    r_series: Resistance,
    b: Temperature,
    resolution: Resolution,
}

impl <'a, A: MyAdc> Thermistor<'a, A>{
    pub fn new<P, S>(
        adc_peri: P,
        dma_peri: A::DmaType,
        read_pin: A::PinType,
        resolution: Resolution,
        sample_time: A::SampleTime,
        r0: Resistance,
        r_series: Resistance,
        b: Temperature,
        readings: &'a mut DmaBufType,
    ) -> Self {
        let mut adc = A::new(adc_peri);
        adc.set_sample_time(sample_time);
        adc.set_resolution(resolution);
        Self {
            adc,
            read_pin,
            dma_peri,
            readings,
            r0,
            r_series,
            b,
            resolution
        }
    }

    pub async fn read_temperature(&mut self) -> Temperature {
        let readings = self.readings.as_mut();
        self.adc
            .read(
                &mut self.dma_peri,
                [(&mut self.read_pin, self.adc.sample_time())].into_iter(),
                readings,
            )
            .await;
        compute_ntf_thermistor_temperature(
            u64::from(readings[0]),
            self.resolution,
            Temperature::from_celsius(25.0),
            self.b,
            self.r0,
            self.r_series,
        )
    }

}
