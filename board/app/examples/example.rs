#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn blink(pin: embassy_stm32::peripherals::PB11) {
    let mut led = embassy_stm32::gpio::Output::new(
        pin,
        embassy_stm32::gpio::Level::Low,
        embassy_stm32::gpio::Speed::Low,
    );

    loop {
        led.toggle();
        embassy_time::Timer::after_millis(200).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut endstop =
        embassy_stm32::exti::ExtiInput::new(p.PE8, p.EXTI8, embassy_stm32::gpio::Pull::Down);

    spawner.spawn(blink(p.PB11)).unwrap();

    loop {
        endstop.wait_for_high().await;
        defmt::info!("Endstop hit");
        embassy_time::Timer::after_millis(1000).await;
    }
}
