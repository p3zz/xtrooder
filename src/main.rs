#![allow(dead_code)]
#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use crate::math::distance::Distance;
use core::str;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, Resolution};
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::{ADC1, DMA1_CH0, PA1, PA10, PD8, PD9, TIM1, USART3};
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Duration, Timer};

use heapless::spsc::Queue;
use {defmt_rtt as _, panic_probe as _};

mod stepper;
use stepper::a4988::Stepper;
use stepper::test::test as stepper_test;

mod planner;
use planner::planner::Planner;
use planner::test::test as planner_test;

mod parser;
use parser::parser::{parse_line, GCommand};
use parser::test::test as parser_test;
// use planner::test::test as planner_test;
// use stepper::test::test as stepper_test;

mod hotend;
use hotend::controller::Hotend;
use hotend::heater::Heater;
use hotend::thermistor::Thermistor;

mod math;
use math::temperature::Temperature;

bind_interrupts!(struct Irqs {
    USART3 => usart::InterruptHandler<peripherals::USART3>;
});

static TEST: bool = true;
static COMMAND_QUEUE: Mutex<ThreadModeRawMutex, Queue<GCommand, 16>> = Mutex::new(Queue::new());

static TARGET_TEMPERATURE: Mutex<ThreadModeRawMutex, Option<f64>> = Mutex::new(None);

#[embassy_executor::task]
async fn input_handler(peri: USART3, rx: PD9, tx: PD8, dma_rx: DMA1_CH0) {
    let mut uart = Uart::new(peri, rx, tx, Irqs, NoDma, dma_rx, Config::default());
    if uart.is_err() {
        return;
    }

    let mut uart = uart.unwrap();

    let mut buf = [0u8; 16];

    let poll_interval = Duration::from_millis(50);

    loop {
        let mut q = COMMAND_QUEUE.lock().await;
        if !q.is_full() {
            match uart.read(&mut buf).await {
                Ok(_) => {
                    let line = str::from_utf8(&buf).unwrap();
                    info!("{}", line);
                    match parse_line(line) {
                        Some(cmd) => q.enqueue(cmd).unwrap(),
                        None => info!("invalid line"),
                    };
                }
                Err(_) => (),
            };
        }
        Timer::after(poll_interval).await;
    }
}

#[embassy_executor::task]
async fn hotend_handler(adc: ADC1, read_pin: PA1, heater_tim: TIM1, heater_pin: PA10) {
    let hotend_thermistor_adc = Adc::new(adc, &mut Delay);
    let hotend_thermistor = Thermistor::new(
        hotend_thermistor_adc,
        read_pin,
        Resolution::BITS16,
        10_000.0,
        Temperature::from_kelvin(3950.0),
    );

    let heater_out = SimplePwm::new(
        heater_tim,
        None,
        None,
        Some(PwmPin::new_ch3(
            heater_pin,
            embassy_stm32::gpio::OutputType::PushPull,
        )),
        None,
        hz(1),
        embassy_stm32::timer::CountingMode::EdgeAlignedUp,
    );

    let hotend_heater = Heater::new(heater_out, Channel::Ch3);
    let mut hotend = Hotend::new(hotend_heater, hotend_thermistor);

    let dt = Duration::from_millis(500);
    loop {
        let mut tmp = TARGET_TEMPERATURE.lock().await;
        match *tmp {
            Some(t) => hotend.set_target_temperature(t),
            None => (),
        }
        *tmp = None;
        hotend.update(dt);
        Timer::after(dt).await;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    if TEST {
        info!("Testing");
        parser_test();
        planner_test().await;
        stepper_test();
        info!("Test finished succesfully");
        return;
    }

    info!("Hello main");

    let p = embassy_stm32::init(Default::default());

    const STEPS_PER_REVOLUTION: u64 = 200;
    let pulley_radius = Distance::from_mm(5.0);

    // setup X stepper
    let x_step = SimplePwm::new(
        p.TIM5,
        Some(PwmPin::new_ch1(
            p.PA0,
            embassy_stm32::gpio::OutputType::PushPull,
        )),
        None,
        None,
        None,
        hz(1),
        embassy_stm32::timer::CountingMode::EdgeAlignedUp,
    );

    let x_dir = Output::new(p.PB0, Level::Low, Speed::Low);

    let x_stepper = Stepper::new(
        x_step,
        Channel::Ch1,
        x_dir,
        STEPS_PER_REVOLUTION,
        pulley_radius,
    );

    // setup Y stepper

    let y_step = SimplePwm::new(
        p.TIM15,
        Some(PwmPin::new_ch1(
            p.PA2,
            embassy_stm32::gpio::OutputType::PushPull,
        )),
        None,
        None,
        None,
        hz(1),
        embassy_stm32::timer::CountingMode::EdgeAlignedUp,
    );

    let y_dir = Output::new(p.PB1, Level::Low, Speed::Low);

    let y_stepper = Stepper::new(
        y_step,
        Channel::Ch1,
        y_dir,
        STEPS_PER_REVOLUTION,
        pulley_radius,
    );

    // // setup Z stepper

    let z_step = SimplePwm::new(
        p.TIM3,
        Some(PwmPin::new_ch1(
            p.PA6,
            embassy_stm32::gpio::OutputType::PushPull,
        )),
        None,
        None,
        None,
        hz(1),
        embassy_stm32::timer::CountingMode::EdgeAlignedUp,
    );

    let z_dir = Output::new(p.PB2, Level::Low, Speed::Low);

    let z_stepper = Stepper::new(
        z_step,
        Channel::Ch1,
        z_dir,
        STEPS_PER_REVOLUTION,
        pulley_radius,
    );

    // // setup E stepper

    let e_step = SimplePwm::new(
        p.TIM14,
        Some(PwmPin::new_ch1(
            p.PA7,
            embassy_stm32::gpio::OutputType::PushPull,
        )),
        None,
        None,
        None,
        hz(1),
        embassy_stm32::timer::CountingMode::EdgeAlignedUp,
    );

    let e_dir = Output::new(p.PB3, Level::Low, Speed::Low);

    let e_stepper = Stepper::new(
        e_step,
        Channel::Ch1,
        e_dir,
        STEPS_PER_REVOLUTION,
        pulley_radius,
    );

    let mut planner = Planner::new(x_stepper, y_stepper, z_stepper, e_stepper);

    _spawner
        .spawn(input_handler(p.USART3, p.PD9, p.PD8, p.DMA1_CH0))
        .unwrap();

    _spawner
        .spawn(hotend_handler(p.ADC1, p.PA1, p.TIM1, p.PA10))
        .unwrap();

    let dt = Duration::from_millis(50);

    loop {
        let mut c: Option<GCommand> = None;
        // we need to unlock the mutex earlier than the loop scope so the read_input task can
        // fill the queue in background while the command is being executed
        {
            let mut q = COMMAND_QUEUE.lock().await;
            c = q.dequeue();
        } // mutex is freed here

        match c {
            Some(cmd) => match cmd {
                GCommand::M104 { s } => {
                    let mut tmp = TARGET_TEMPERATURE.lock().await;
                    *tmp = s;
                }
                _ => planner.execute(cmd).await,
            },
            None => (),
        };

        Timer::after(dt).await;
    }
}
