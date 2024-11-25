#![no_std]
#![no_main]

use app::config::{PidConfig, ThermistorOptionsConfig};
use app::{timer_channel, AdcWrapper, Clock, ResolutionWrapper, SimplePwmWrapper};
use common::PwmBase;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel, Resolution, SampleTime};
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::{hz, khz};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::{low_level::CountingMode, Channel};
use embassy_time::{Duration, Timer};
use math::measurements::{Resistance, Temperature};
use static_cell::StaticCell;
use thermal_actuator::controller::ThermalActuator;
use thermal_actuator::heater::Heater;
use thermal_actuator::thermistor::{DmaBufType, Thermistor};
use {defmt_rtt as _, panic_probe as _};

#[cfg(feature="defmt-log")]
use defmt::{error, info, println};

#[link_section = ".ram_d3"]
static DMA_BUF: StaticCell<DmaBufType> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // TODO check this configuration. It's in the embassy stm32 examples of ADC. Not so sure why it's needed but without this the
    // program won't run
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8), // SPI1 cksel defaults to pll1_q
            divr: None,
        });
        config.rcc.pll2 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV8), // 100mhz
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.mux.adcsel = mux::Adcsel::PLL2_P;
    }

    let p = embassy_stm32::init(config);

    let readings = DMA_BUF.init([0u16; 1]);

    let mut adc = Adc::new(p.ADC1);
    adc.set_sample_time(SampleTime::CYCLES32_5);
    let mut adc = AdcWrapper::new(adc, p.DMA1_CH0, ResolutionWrapper::new(Resolution::BITS12));

    let thermistor: Thermistor<'_, _> = Thermistor::new(
        p.PA0.degrade_adc(),
        readings,
        ThermistorOptionsConfig {
            r_series: Resistance::from_ohms(10_000.0),
            r0: Resistance::from_ohms(100_000.0),
            b: Temperature::from_kelvin(3950.0),
            samples: 5
        },
    );

    let heater_out = SimplePwm::new(
        p.TIM4,
        None,
        None,
        None,
        Some(PwmPin::new_ch4(p.PB9, OutputType::PushPull)),
        khz(1),
        CountingMode::EdgeAlignedUp,
    );

    let mut heater_out_wrapper = SimplePwmWrapper::new(heater_out);

    let channel = 4;
    let channel = timer_channel!(channel).expect("Invalid timer channel");

    let heater = Heater::new(
        channel,
        PidConfig {
            k_p: 400.0,
            k_i: 5.0,
            k_d: 1.0,
        },
    );

    let mut hotend = ThermalActuator::new(heater, thermistor);

    hotend.enable(&mut heater_out_wrapper);

    hotend.set_temperature(Temperature::from_celsius(200f64));
    
    #[cfg(feature="defmt-log")]
    info!("ThermalActuator example");
    let dt = Duration::from_millis(10);
    #[cfg(feature="defmt-log")]
    println!("Max duty cycle: {}", heater_out_wrapper.get_max_duty());

    loop {
        match hotend.update(dt.into(), &mut heater_out_wrapper, &mut adc).await {
            Ok(r) => {
                #[cfg(feature="defmt-log")]
                println!("Dt: {}\tTemperaure: {}\tDuty cycle: {}", dt.as_millis(), r.0.as_celsius(), r.1);
            },
            Err(_) => {
                #[cfg(feature="defmt-log")]
                error!("Target temperature not set")
            },
        };
        Timer::after(dt).await;
    }
}
