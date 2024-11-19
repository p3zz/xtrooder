use common::MyAdc;
use math::common::compute_ntf_thermistor_temperature;
use math::measurements::{Resistance, Temperature};

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

#[cfg(test)]
mod tests{
    use super::*;

    #[derive(Copy, Clone)]
    pub enum Resolution{
        BITS12,
    }

    impl From<Resolution> for u64{
        fn from(val: Resolution) -> Self {
            match val {
                Resolution::BITS12 => 1 << 12,
            }    
        }
    }

    struct AdcWrapper{
        resolution: Resolution,
        value: u16,
    }

    impl MyAdc for AdcWrapper{
        type PinType = ();
    
        type DmaType = ();
    
        type PeriType = ();
    
        type SampleTime = ();
    
        type Resolution = Resolution;
    
        fn new(_peripheral: Self::PeriType) -> Self {
            Self{
                resolution: Resolution::BITS12,
                value: 2000
            }
        }
    
        fn set_sample_time(&mut self, _sample_time: Self::SampleTime) {
        }
    
        fn sample_time(&self) -> Self::SampleTime {
            
        }
    
        fn set_resolution(&mut self, resolution: Self::Resolution) {
            self.resolution = resolution;
        }
    
        async fn read(
            &mut self,
            _dma: &mut Self::DmaType,
            _pin: core::array::IntoIter<(&mut Self::PinType, Self::SampleTime), 1>,
            readings: &mut [u16]
        ) {
            readings[0] = self.value
        }
    }

    #[tokio::test]
    async fn test_thermistor(){
        let mut readings = [0u16;1];
        let mut thermistor: Thermistor<'_, AdcWrapper> = Thermistor::new(
            (), 
            (), 
            (),
            (),
            Resolution::BITS12,
            &mut readings,
            ThermistorConfig{
                r_series: Resistance::from_ohms(10_000.0),
                r0: Resistance::from_ohms(10_000.0),
                b: Temperature::from_kelvin(3950.0)
            }
        );
        let t = thermistor.read_temperature().await;
        assert_eq!(25.0, t.as_celsius());
    }

}