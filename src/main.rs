#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed, AnyPin};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::Channel;

#[embassy_executor::task(pool_size=2)]
async fn blink(mut pin: Output<'static, AnyPin>, duration: Duration){
    loop {
        info!("high");
        pin.set_high();
        Timer::after(duration).await;

        info!("low");
        pin.set_low();
        Timer::after(duration).await;
    }
}


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let red = Output::new(p.PB14, Level::High, Speed::Low);

    let green = Output::new(p.PB0, Level::High, Speed::Low);

    let _ =_spawner.spawn(blink(red.degrade(), Duration::from_millis(200)));
    let _ =_spawner.spawn(blink(green.degrade(), Duration::from_millis(500)));

    loop{
        Timer::after(Duration::from_millis(2000)).await;
        info!("Hello World!");
    }
    
}
