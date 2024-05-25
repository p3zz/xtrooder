#![no_std]
#![no_main]

use app::hotend::controller::Hotend;
use app::hotend::heater::Heater;
use app::hotend::thermistor::Thermistor;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::adc::Resolution;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::{Channel, CountingMode};
use embassy_time::{Duration, Timer};
use math::temperature::Temperature;
use {defmt_rtt as _, panic_probe as _};

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

    let thermistor = Thermistor::new(
        p.ADC1,
        p.PA3,
        Resolution::BITS12,
        100_000.0,
        10_000.0,
        Temperature::from_kelvin(3950.0),
    );

    let heater_out = SimplePwm::new(
        p.TIM4,
        None,
        None,
        None,
        Some(PwmPin::new_ch4(p.PB9, OutputType::PushPull)),
        hz(1),
        CountingMode::EdgeAlignedUp,
    );
    let heater = Heater::new(heater_out, Channel::Ch4);
    let mut hotend = Hotend::new(heater, thermistor);

    hotend.set_temperature(Temperature::from_celsius(100f64));

    info!("Hotend example");
    let dt = Duration::from_millis(100);

    loop {
        hotend.update(dt);
        Timer::after(dt).await;
    }
}
