use core::time::Duration;

use common::{AdcBase, PwmBase};
use math::measurements::Temperature;

use crate::{heater::Heater, thermistor::Thermistor};

pub struct ThermalActuator<'a, P: PwmBase, A: AdcBase> {
    heater: Heater<P>,
    thermistor: Thermistor<'a, A>,
}

impl<'a, P: PwmBase, A: AdcBase> ThermalActuator<'a, P, A> {
    pub fn new(heater: Heater<P>, thermistor: Thermistor<'a, A>) -> Self {
        Self { heater, thermistor }
    }

    pub fn enable(&mut self, pwm: &mut P) {
        self.heater.enable(pwm);
    }

    pub fn disable(&mut self, pwm: &mut P) {
        self.heater.disable(pwm);
    }

    pub fn set_temperature(&mut self, temperature: Temperature) {
        self.heater.set_target_temperature(temperature);
    }

    pub async fn update(
        &mut self,
        dt: Duration,
        pwm: &mut P,
        adc: &mut A,
    ) -> (Temperature, Option<f64>) {
        let curr_tmp = self.read_temperature(adc).await;
        let duty_cycle = self.heater.update(curr_tmp, dt, pwm).ok();
        (curr_tmp, duty_cycle)
    }

    pub async fn read_temperature(&mut self, adc: &mut A) -> Temperature {
        self.thermistor.read_temperature(adc).await
    }
}

#[cfg(test)]
mod tests {
    use common::PidConfig;
    use math::measurements::Resistance;

    use crate::thermistor::ThermistorConfig;

    use super::*;

    #[derive(Default)]
    struct PwmChannel {
        enabled: bool,
        duty_cycle: u64,
    }

    #[derive(Clone, Copy)]
    enum Channel {
        Ch1,
        Ch2,
        Ch3,
        Ch4,
    }

    struct PwmWrapper {
        pub ch1: PwmChannel,
        pub ch2: PwmChannel,
        pub ch3: PwmChannel,
        pub ch4: PwmChannel,
        pub max_duty: u64,
    }

    impl PwmWrapper {
        fn new() -> Self {
            Self {
                ch1: PwmChannel::default(),
                ch2: PwmChannel::default(),
                ch3: PwmChannel::default(),
                ch4: PwmChannel::default(),
                max_duty: 4096,
            }
        }
    }

    impl PwmBase for PwmWrapper {
        type Channel = Channel;

        fn enable(&mut self, channel: Self::Channel) {
            match channel {
                Channel::Ch1 => self.ch1.enabled = true,
                Channel::Ch2 => self.ch2.enabled = true,
                Channel::Ch3 => self.ch3.enabled = true,
                Channel::Ch4 => self.ch4.enabled = true,
            }
        }

        fn disable(&mut self, channel: Self::Channel) {
            match channel {
                Channel::Ch1 => self.ch1.enabled = false,
                Channel::Ch2 => self.ch2.enabled = false,
                Channel::Ch3 => self.ch3.enabled = false,
                Channel::Ch4 => self.ch4.enabled = false,
            }
        }

        fn get_max_duty(&self) -> u64 {
            self.max_duty
        }

        fn set_duty(&mut self, channel: Self::Channel, duty_cycle: u64) {
            match channel {
                Channel::Ch1 => self.ch1.duty_cycle = duty_cycle,
                Channel::Ch2 => self.ch2.duty_cycle = duty_cycle,
                Channel::Ch3 => self.ch3.duty_cycle = duty_cycle,
                Channel::Ch4 => self.ch4.duty_cycle = duty_cycle,
            }
        }
    }

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
                value: 2000,
            }
        }
    }

    impl AdcBase for AdcWrapper {
        type PinType = ();

        type SampleTime = ();

        type Resolution = Resolution;

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
    async fn test_thermal_actuator() {
        let target_temp = Temperature::from_celsius(140.0);
        let mut pwm = PwmWrapper::new();
        let mut adc = AdcWrapper::new();
        let heater: Heater<PwmWrapper> = Heater::new(
            Channel::Ch2,
            PidConfig {
                k_p: 30.0,
                k_i: 0.0,
                k_d: 0.1,
            },
        );
        let mut readings = [0u16; 1];
        let thermistor: Thermistor<'_, _> = Thermistor::new(
            (),
            &mut readings,
            ThermistorConfig {
                r_series: Resistance::from_ohms(10_000.0),
                r0: Resistance::from_ohms(10_000.0),
                b: Temperature::from_kelvin(3950.0),
                samples: 1,
            },
        );
        let mut actuator = ThermalActuator::new(heater, thermistor);
        actuator.enable(&mut pwm);
        actuator.set_temperature(target_temp);
        let temp = actuator
            .update(Duration::from_millis(50), &mut pwm, &mut adc)
            .await;
        assert_eq!(26.984236773480745, temp.0.as_celsius());
        assert!(temp.1.is_some());
        // FIXME
        // assert_eq!(3616, temp.1.unwrap());
    }
}
