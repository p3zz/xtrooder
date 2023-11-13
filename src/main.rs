#![allow(dead_code)]
#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::str;
use defmt::assert;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::Channel;
use embassy_stm32::time::hz;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use heapless::String;
use {defmt_rtt as _, panic_probe as _};

mod stepper;
use stepper::a4988::Stepper;
use stepper::units::{Length, Position, Position3D, Speed as StepperSpeed};

mod planner;
use planner::planner::Planner;

mod parser;
use parser::parser::parse_line;
use parser::test::test as parser_test;
use stepper::test::test as stepper_test;

bind_interrupts!(struct Irqs {
    USART3 => usart::InterruptHandler<peripherals::USART3>;
});

const TEST: bool = true;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    if TEST {
        info!("Testing");
        parser_test();
        stepper_test().await;
        info!("Test finished succesfully");
        return;
    }

    info!("Hello main");

    let p = embassy_stm32::init(Default::default());

    const STEPS_PER_REVOLUTION: u64 = 200;
    let pulley_radius: Length = Length::from_mm(5.0).unwrap();

    // setup X stepper

    let x_step = SimplePwm::new(
        p.TIM5,
        Some(PwmPin::new_ch1(p.PA0)),
        None,
        None,
        None,
        hz(1),
    );

    let x_dir = Output::new(p.PB0, Level::Low, Speed::Low);

    let x_stepper = Stepper::new(x_step, x_dir.degrade(), STEPS_PER_REVOLUTION, pulley_radius);

    // setup Y stepper

    let y_step = SimplePwm::new(
        p.TIM15,
        Some(PwmPin::new_ch1(p.PA2)),
        None,
        None,
        None,
        hz(1),
    );

    let y_dir = Output::new(p.PB1, Level::Low, Speed::Low);

    let y_stepper = Stepper::new(y_step, y_dir.degrade(), STEPS_PER_REVOLUTION, pulley_radius);

    // // setup Z stepper

    let z_step = SimplePwm::new(
        p.TIM3,
        Some(PwmPin::new_ch1(p.PA6)),
        None,
        None,
        None,
        hz(1),
    );

    let z_dir = Output::new(p.PB2, Level::Low, Speed::Low);

    let z_stepper = Stepper::new(z_step, z_dir.degrade(), STEPS_PER_REVOLUTION, pulley_radius);

    // // setup E stepper

    let e_step = SimplePwm::new(
        p.TIM14,
        Some(PwmPin::new_ch1(p.PA7)),
        None,
        None,
        None,
        hz(1),
    );

    let e_dir = Output::new(p.PB3, Level::Low, Speed::Low);

    let e_stepper = Stepper::new(e_step, e_dir.degrade(), STEPS_PER_REVOLUTION, pulley_radius);

    let mut planner = Planner::new(x_stepper, y_stepper, z_stepper, e_stepper);
    let mut uart = Uart::new(
        p.USART3,
        p.PD9,
        p.PD8,
        Irqs,
        NoDma,
        NoDma,
        Config::default(),
    );

    let mut buf = [0u8; 16];
    loop {
        match uart.blocking_read(&mut buf) {
            Ok(_) => {
                let line = str::from_utf8(&buf).unwrap();
                match parse_line(line) {
                    Some(cmd) => info!("valid command ready to be processed"), // TODO execute command
                    None => info!("invalid line"),
                };
            }
            Err(_) => info!("error reading from serial"),
        };

        // planner.move_to(Position3D::new(Position1D::from_mm(10.0),Position1D::from_mm(20.0),Position1D::from_mm(0.0)), StepperSpeed::from_mmps(5.0).unwrap()).await;
        // planner.move_to(Position3D::new(Position1D::from_mm(-5.0),Position1D::from_mm(20.0),Position1D::from_mm(0.0)), StepperSpeed::from_mmps(5.0).unwrap()).await;
        // planner.move_to(Position3D::new(Position1D::from_mm(15.0),Position1D::from_mm(0.0),Position1D::from_mm(0.0)), StepperSpeed::from_mmps(10.0).unwrap()).await;
    }
}
