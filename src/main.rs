#![no_std]
#![no_main]

use stepper::{Stepper, dps_from_radius, StepperDirection, Length};
use panic_halt as _;
use cortex_m_rt::entry;

use stm32h7xx_hal::{
    prelude::*,
    timer::Timer,
    block, time::MilliSeconds
};

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
    let green = gpiob.pb0.into_push_pull_output();

    // Configure gpio B pin 14 (red led) as a push-pull output.
    let red = gpiob.pb14.into_push_pull_output();
    
    let timer = Timer::tim1(dp.TIM1, ccdr.peripheral.TIM1, &ccdr.clocks);

    let steps_per_revolution = 200;
    let pulley_radius = Length::from_millimeters(5.0);

    let mut stepper = Stepper::new(green, red, steps_per_revolution, timer, dps_from_radius(pulley_radius, steps_per_revolution));

    let mut t = Timer::tim2(dp.TIM2, ccdr.peripheral.TIM2, &ccdr.clocks);

    stepper.set_direction(StepperDirection::CounterClockwise);
    stepper.set_speed(60);

    loop{
        stepper.move_for(Length::from_millimeters(1000.0));
        t.start(MilliSeconds(500));
        block!(t.wait()).unwrap();
    }
}
