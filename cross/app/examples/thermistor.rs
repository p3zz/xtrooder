#![no_std]
#![no_main]

use app::{config::ThermistorOptionsConfig, AdcWrapper, ResolutionWrapper};
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::adc::{AdcChannel, Resolution, SampleTime};
use embassy_time::{Duration, Timer};
use math::measurements::{Resistance, Temperature};
use static_cell::StaticCell;
use thermal_actuator::thermistor::{DmaBufType, Thermistor};
use {defmt_rtt as _, panic_probe as _};

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

    let mut thermistor: Thermistor<'_, AdcWrapper<'_, _, _>> = Thermistor::new(
        p.ADC1,
        p.DMA1_CH0,
        p.PA0.degrade_adc(),
        SampleTime::CYCLES32_5,
        ResolutionWrapper::new(Resolution::BITS12),
        readings,
        ThermistorOptionsConfig {
            r_series: Resistance::from_ohms(100_000.0),
            r0: Resistance::from_ohms(10_000.0),
            b: Temperature::from_kelvin(3950.0),
        },
    );

    info!("Thermistor example");
    loop {
        let t = thermistor.read_temperature().await;
        info!("Temperature: {}°C", t.as_celsius());
        Timer::after(Duration::from_millis(200)).await;
    }
}
