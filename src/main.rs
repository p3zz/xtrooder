#![allow(dead_code)]

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, usart, peripherals};
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Output, Level, Speed};
use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::Channel;
use embassy_stm32::time::hz;
use {defmt_rtt as _, panic_probe as _};
use defmt::assert;
use embassy_stm32::usart::{Uart, Config};
use heapless::String;
use core::str;

mod stepper;
use stepper::a4988::{Stepper, dps_from_radius};
use stepper::units::{Speed as StepperSpeed, Position3D, Position, Length};

mod planner;
use planner::planner::{Planner};

mod parser;
use parser::parser::parse_line;
use parser::test::test;

bind_interrupts!(struct Irqs {
    USART3 => usart::InterruptHandler<peripherals::USART3>;
});

const TEST: bool = true;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    if TEST {
        info!("Testing");
        test();
        info!("Test finished succesfully");
        return;
    }

    info!("Hello main");

    // let p = embassy_stm32::init(Default::default());
    // let mut red = Output::new(p.PA0, Level::Low, Speed::Medium).degrade();
    // let mut green = Output::new(p.PA6, Level::Low, Speed::Medium).degrade();

    // const STEPS_PER_REVOLUTION: u64 = 200;
    // let pulley_radius: Length = Length::from_mm(5.0).unwrap();

    // let mut red_pwm = SimplePwm::new(p.TIM3, Some(PwmPin::new_ch1(p.PA6)),
    //     None, None, None, hz(1));
    // let red_max = red_pwm.get_max_duty();
    // red_pwm.set_duty(Channel::Ch1, red_max/2);

    // let red_dir = Output::new(p.PB0, Level::Low, Speed::Low);

    // let red_stepper = Stepper::new(red_pwm, red_dir.degrade(), STEPS_PER_REVOLUTION, pulley_radius);

    // let mut green_pwm = SimplePwm::new(p.TIM5, Some(PwmPin::new_ch1(p.PA0)),
    //     None, None, None, hz(1));
    // let green_max = green_pwm.get_max_duty();
    // green_pwm.set_duty(Channel::Ch1, green_max/2);

    // let green_dir = Output::new(p.PB14, Level::Low, Speed::Low);

    // let green_stepper = Stepper::new(green_pwm, green_dir.degrade(), STEPS_PER_REVOLUTION, pulley_radius);

    // let mut planner = Planner::new(red_stepper, green_stepper);
    // let mut uart = Uart::new(p.USART3, p.PD9, p.PD8, Irqs, NoDma, NoDma, Config::default());
    
    // let mut buf = [0u8; 16];
    // loop {
    //     match uart.blocking_read(&mut buf){
    //         Ok(_) => {
    //             let line = str::from_utf8(&buf).unwrap();
    //             match parse_line(line){
    //                 Some(cmd) => info!("valid command ready to be processed"), // TODO execute command
    //                 None => info!("invalid line")
    //             };
    //         },
    //         Err(_) => info!("error reading from serial"),
    //     };
        
        // planner.move_to(Position3D::new(Position1D::from_mm(10.0),Position1D::from_mm(20.0),Position1D::from_mm(0.0)), StepperSpeed::from_mmps(5.0).unwrap()).await;
        // planner.move_to(Position3D::new(Position1D::from_mm(-5.0),Position1D::from_mm(20.0),Position1D::from_mm(0.0)), StepperSpeed::from_mmps(5.0).unwrap()).await;
        // planner.move_to(Position3D::new(Position1D::from_mm(15.0),Position1D::from_mm(0.0),Position1D::from_mm(0.0)), StepperSpeed::from_mmps(10.0).unwrap()).await;
    // }
}