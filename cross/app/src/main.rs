#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    dma::NoDma,
    gpio::{Level, Output, OutputType, Speed as PinSpeed},
    peripherals::{DMA1_CH0, DMA1_CH1, PB10, PB11, PD8, PD9, USART1, USART3},
    time::hz,
    timer::{
        simple_pwm::{PwmPin, SimplePwm},
        Channel, CountingMode,
    },
    usart::{Config, InterruptHandler, Uart},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use heapless::spsc::Queue;
use heapless::{String, Vec};
use math::{distance::Distance, speed::Speed as StepperSpeed};
use parser::parser::{parse_line, GCodeParser, GCommand};
use planner::{
    motion::{linear_move_for, linear_move_to},
    planner::Planner,
};
use stepper::a4988::{Stepper, SteppingMode};
mod hotend;
mod planner;
mod stepper;
mod utils;

use core::str;

use crate::planner::motion::linear_move_to_2d;

static COMMAND_QUEUE: Mutex<ThreadModeRawMutex, Queue<GCommand, 8>> = Mutex::new(Queue::new());

bind_interrupts!(struct Irqs {
    USART3 => InterruptHandler<USART3>;
});

#[embassy_executor::task]
async fn input_handler(peri: USART3, rx: PB11, tx: PB10, dma_rx: DMA1_CH0, dma_tx: DMA1_CH1) {
    let mut config = Config::default();
    config.baudrate = 19200;
    let mut uart =
        Uart::new(peri, rx, tx, Irqs, dma_tx, dma_rx, config).expect("Cannot initialize USART");

    let mut parser = GCodeParser::new();
    let mut buf = [0u8; 64];
    let msg = "#next";

    loop {
        let mut available = false;
        // check if the command queue is full
        {
            let q = COMMAND_QUEUE.lock().await;
            available = !q.is_full();
        }

        if !available {
            // info!("queue is full");
            Timer::after(Duration::from_millis(1)).await;
            continue;
        }

        if parser.is_queue_full() {
            info!("parser queue is full");
            Timer::after(Duration::from_millis(1)).await;
            continue;
        }

        if let Err(_) = uart.write_all(msg.as_bytes()).await {
            info!("Cannot send request");
            Timer::after(Duration::from_millis(1)).await;
            continue;
        }

        if let Ok(n) = uart.read_until_idle(&mut buf).await {
            let line = str::from_utf8(&buf).unwrap();
            info!("[{}] Found {}", n, line);
            match parser.parse(&buf) {
                Ok(()) => {
                    let mut q = COMMAND_QUEUE.lock().await;
                    for _ in 0..parser.queue_len() {
                        if !q.is_full() {
                            info!("command enqueued");
                            let cmd = parser.pick_from_queue().unwrap();
                            q.enqueue(cmd).unwrap();
                        }
                    }
                }
                Err(_) => (),
            };
        }
        buf = [0u8; 64];
    }
}
// use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let a_step = SimplePwm::new(
        p.TIM5,
        Some(PwmPin::new_ch1(p.PA0, OutputType::PushPull)),
        None,
        None,
        None,
        hz(1),
        CountingMode::EdgeAlignedUp,
    );

    let a_dir = Output::new(p.PB0, Level::Low, PinSpeed::Low);

    let mut a_stepper = Stepper::new(
        a_step,
        Channel::Ch1,
        a_dir,
        200,
        Distance::from_mm(0.15f64),
        SteppingMode::HalfStep,
    );

    // let planner = Planner::new(a_stepper, None, None, None);

    // let b_step = SimplePwm::new(
    //     p.TIM5,
    //     None,
    //     None,
    //     None,
    //     None,
    //     hz(1),
    //     CountingMode::EdgeAlignedUp,
    // );

    // let b_dir = Output::new(p.PB0, Level::Low, PinSpeed::Low);

    // let mut b_stepper = Stepper::new(
    //     a_step,
    //     Channel::Ch1,
    //     a_dir,
    //     200,
    //     Distance::from_mm(0.15f64),
    //     SteppingMode::HalfStep,
    // );

    _spawner
        .spawn(input_handler(
            p.USART3, p.PB11, p.PB10, p.DMA1_CH0, p.DMA1_CH1,
        ))
        .unwrap();

    loop {
        let mut c: Option<GCommand> = None;
        {
            let mut q = COMMAND_QUEUE.lock().await;
            c = q.dequeue();
        } // mutex is freed here

        match c {
            Some(cmd) => match cmd {
                GCommand::G0 { x, y, z, f } => {
                    info!("performing a linear movement");
                    linear_move_to(
                        &mut a_stepper,
                        Distance::from_mm(x.unwrap()),
                        StepperSpeed::from_mm_per_second(f.unwrap()),
                    )
                    .await
                    .unwrap_or_else(|_| info!("Cannot perform move"))
                }
                _ => info!("implement movement"),
            },
            None => (),
        };

        Timer::after(Duration::from_millis(1)).await;
    }
}
