#![no_std]
#![no_main]

use embassy_executor::Spawner;

mod hotend;
mod planner;
mod stepper;

use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    loop {}
}
