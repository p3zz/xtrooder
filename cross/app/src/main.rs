#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    dma::NoDma,
    gpio::{Level, Output, OutputType, Speed as PinSpeed},
    peripherals::{DMA1_CH0, PB10, PB11, PD8, PD9, USART1, USART3},
    time::hz,
    timer::{
        simple_pwm::{PwmPin, SimplePwm},
        Channel, CountingMode,
    },
    usart::{Config, InterruptHandler, Uart},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use heapless::spsc::Queue;
use math::{distance::Distance, speed::Speed as StepperSpeed};
use parser::parser::{parse_line, GCommand};
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

static COMMAND_QUEUE: Mutex<ThreadModeRawMutex, Queue<GCommand, 16>> = Mutex::new(Queue::new());

bind_interrupts!(struct Irqs {
    USART3 => InterruptHandler<USART3>;
});

#[embassy_executor::task]
async fn input_handler(peri: USART3, rx: PB11, tx: PB10, dma_rx: DMA1_CH0) {
    let mut uart = Uart::new(peri, rx, tx, Irqs, NoDma, dma_rx, Config::default())
        .expect("Cannot initialize USART");

    let mut buf = [0u8; 32];

    let poll_interval = Duration::from_millis(50);

    loop {
        info!("waiting for uart rx");
        let mut available = false;
        {
            let q = COMMAND_QUEUE.lock().await;
            available = !q.is_full();
        }
        if !available {
            continue;
        }
        match uart.read_until_idle(&mut buf).await {
            Ok(_) => {
                let mut line = str::from_utf8(&buf).unwrap();
                line = line.trim_end_matches(|c| c == char::from_u32(0).unwrap());
                match parse_line(line) {
                    Some(cmd) => {
                        let mut q = COMMAND_QUEUE.lock().await;
                        q.enqueue(cmd).unwrap()
                    }
                    None => info!("invalid line"),
                };
            }
            Err(_) => (),
        };
        buf = [0u8; 32];
        Timer::after(poll_interval).await;
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
        SteppingMode::FullStep,
    );

    _spawner
        .spawn(input_handler(p.USART3, p.PB11, p.PB10, p.DMA1_CH0))
        .unwrap();

    // let planner = Planner::new(a_stepper, a_stepper, a_stepper, a_stepper);

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
                GCommand::G0 { x, y, z, f } => {
                    linear_move_for(
                        &mut a_stepper,
                        Distance::from_mm(x.unwrap()),
                        StepperSpeed::from_mm_per_second(f.unwrap()),
                    )
                    .await
                }
                _ => info!("implement movement"),
            },
            None => (),
        };

        info!("loop");
        Timer::after_millis(500).await;

        // match a_stepper.move_for(Distance::from_mm(distance)).await {
        //     Ok(_) => info!("move done"),
        //     Err(_) => info!("cannot move"),
        // };

        // distance = -distance;
    }
}
