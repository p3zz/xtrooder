#![no_std]
#![no_main]

use {app::init_input_pin, defmt::info, defmt_rtt as _, embassy_executor::Spawner, embassy_stm32::{exti::ExtiInput, gpio::Pull}, embassy_time::{Duration, Timer}, panic_probe as _};
use common::ExtiInputPinBase;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let endstop = ExtiInput::new(p.PE8, p.EXTI8, Pull::Down);
    let mut endstop = init_input_pin!(endstop);
    loop {
        info!("Waiting for endstop to hit");
        endstop.wait_for_high().await;
        #[cfg(feature = "defmt-log")]
        info!("Endstop hit");
        Timer::after(Duration::from_millis(1000)).await;
    }
}