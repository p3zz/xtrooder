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

#[derive(Clone, Copy)]
pub struct ThermistorConfig{
    pub r_series: Resistance,
    pub r0: Resistance,
    pub b: Temperature,
}

pub struct Thermistor<'a, A: MyAdc>{
    adc: A,
    dma_peri: A::DmaType,
    read_pin: A::PinType,
    readings: &'a mut DmaBufType,
    config: ThermistorConfig,
    resolution: A::Resolution
}

impl <'a, A: MyAdc> Thermistor<'a, A>{
    pub fn new(
        adc_peri: A::PeriType,
        dma_peri: A::DmaType,
        read_pin: A::PinType,
        sample_time: A::SampleTime,
        resolution: A::Resolution,
        readings: &'a mut DmaBufType,
        config: ThermistorConfig
    ) -> Self {
        let mut adc = A::new(adc_peri);
        adc.set_sample_time(sample_time);
        adc.set_resolution(resolution);
        Self {
            adc,
            read_pin,
            dma_peri,
            readings,
            config,
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
            self.resolution.into(),
            Temperature::from_celsius(25.0),
            self.config.b,
            self.config.r0,
            self.config.r_series,
        )
    }

}
