#![no_std]
#![no_main]

use stepper::Stepper;
use panic_halt as _;
use cortex_m_rt::entry;

use stm32h7xx_hal::{
    prelude::*,
    timer::Timer,
};

use embedded_hal::digital::v2::OutputPin;

#[entry]
fn main() -> ! {
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = stm32h7xx_hal::stm32::Peripherals::take().unwrap();

    // Take ownership over the RCC devices and convert them into the corresponding HAL structs
    let rcc = dp.RCC.constrain();

    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();

    // Freeze the configuration of all the clocks in the system and
    // retrieve the Core Clock Distribution and Reset (CCDR) object
    let ccdr = rcc.freeze(pwrcfg, &dp.SYSCFG);

    // Acquire the GPIOB peripheral
    let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);

    // Configure gpio B pin 0 (green led) as a push-pull output.
    let mut green = gpiob.pb0.into_push_pull_output();

    // Configure gpio B pin 14 (red led) as a push-pull output.
    let mut red = gpiob.pb14.into_push_pull_output();
    
    let mut timer = Timer::tim1(dp.TIM1, ccdr.peripheral.TIM1, &ccdr.clocks);

    let stepper = Stepper::new(green, red, 200, timer, 1.0);

    loop{
    }
}
