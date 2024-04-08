#![no_std]
#![no_main]

use {defmt::info, defmt_rtt as _, embassy_executor::Spawner, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_stm32::init(embassy_stm32::Config::default());
    info!("Hotend example");
    loop{}
}