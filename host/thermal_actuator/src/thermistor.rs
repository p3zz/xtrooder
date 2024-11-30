use common::AdcBase;
use math::common::compute_ntf_thermistor_temperature;
use math::measurements::{Resistance, Temperature};

pub type DmaBufType = [u16; 1];

/*
ADC value = R / (R + R0) * Vcc * resolution / Varef
Vcc: voltage reference of the board
Varef: voltage of the thermistor
*/

#[derive(Clone, Copy)]
pub struct ThermistorConfig {
    pub r_series: Resistance,
    pub r0: Resistance,
    pub b: Temperature,
    pub samples: u64,
}

pub struct Thermistor<'a, A: AdcBase> {
    read_pin: A::PinType,
    readings: &'a mut DmaBufType,
    config: ThermistorConfig,
}

impl<'a, A: AdcBase> Thermistor<'a, A> {
    pub fn new(
        read_pin: A::PinType,
        readings: &'a mut DmaBufType,
        config: ThermistorConfig,
    ) -> Self {
        Self {
            read_pin,
            readings,
            config,
        }
    }

    pub async fn read_temperature(&mut self, adc: &mut A) -> Temperature {
        let readings = self.readings.as_mut();
        let mut data = 0u64;
        for _ in 0..self.config.samples {
            adc.read(&mut self.read_pin, readings).await;
            data += u64::from(readings[0]);
        }
        let reading = data / self.config.samples;

        compute_ntf_thermistor_temperature(
            reading,
            adc.resolution().into(),
            Temperature::from_celsius(25.0),
            self.config.b,
            self.config.r0,
            self.config.r_series,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone)]
    pub enum Resolution {
        BITS12,
    }

    impl From<Resolution> for u64 {
        fn from(val: Resolution) -> Self {
            match val {
                Resolution::BITS12 => 1 << 12,
            }
        }
    }

    struct AdcWrapper {
        resolution: Resolution,
        value: u16,
    }

    impl AdcWrapper {
        pub fn new() -> Self {
            Self {
                resolution: Resolution::BITS12,
                value: 2048,
            }
        }
    }

    struct Pin;

    impl AdcBase for AdcWrapper {
        type SampleTime = ();

        type Resolution = Resolution;

        type PinType = ();

        fn set_sample_time(&mut self, _sample_time: Self::SampleTime) {}

        fn sample_time(&self) -> Self::SampleTime {}

        fn set_resolution(&mut self, resolution: Self::Resolution) {
            self.resolution = resolution;
        }

        async fn read(&mut self, _pin: &mut (), readings: &mut [u16]) {
            readings[0] = self.value
        }

        fn resolution(&self) -> Self::Resolution {
            self.resolution
        }
    }

    #[tokio::test]
    async fn test_thermistor() {
        let mut readings = [0u16; 1];
        let mut adc = AdcWrapper::new();
        let mut thermistor: Thermistor<'_, _> = Thermistor::new(
            (),
            &mut readings,
            ThermistorConfig {
                r_series: Resistance::from_ohms(10_000.0),
                r0: Resistance::from_ohms(10_000.0),
                b: Temperature::from_kelvin(3950.0),
                samples: 1,
            },
        );
        let t = thermistor.read_temperature(&mut adc).await;
        assert_eq!(25.0, t.as_celsius());
    }
}
