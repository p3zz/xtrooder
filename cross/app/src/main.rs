#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    adc::{Adc, AdcPin, Resolution},
    bind_interrupts,
    dma::NoDma,
    gpio::{AnyPin, Level, Output, OutputType, Speed as PinSpeed},
    peripherals::{
        ADC1, DMA1_CH0, DMA1_CH1, PA1, PA2, PA3, PB10, PB11, PD8, PD9, TIM1, TIM15, USART1, USART3,
    },
    time::hz,
    timer::{
        simple_pwm::{PwmPin, SimplePwm},
        Channel, CountingMode,
    },
    usart::{InterruptHandler, Uart},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::{Delay, Duration, Timer};
use embedded_io_async::Write;
use heapless::spsc::Queue;
use heapless::{String, Vec};
use hotend::{controller::Hotend, heater::Heater, thermistor::Thermistor};
use math::{distance::Distance, speed::Speed as StepperSpeed, temperature::Temperature};
use parser::parser::{parse_line, GCodeParser, GCommand};
use planner::{
    motion::{linear_move_for, linear_move_to, linear_move_to_2d},
    planner::Planner,
};
use stepper::a4988::{Stepper, SteppingMode};
mod hotend;
mod planner;
mod stepper;
mod utils;

use core::str;

static COMMAND_QUEUE: Mutex<ThreadModeRawMutex, Queue<GCommand, 8>> = Mutex::new(Queue::new());

bind_interrupts!(struct Irqs {
    USART3 => InterruptHandler<USART3>;
});

#[embassy_executor::task]
async fn input_handler(peri: USART3, rx: PB11, tx: PB10, dma_rx: DMA1_CH0, dma_tx: DMA1_CH1) {
    let mut config = embassy_stm32::usart::Config::default();
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
            let line = match str::from_utf8(&buf) {
                Ok(l) => l,
                Err(_) => continue,
            };
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

// https://dev.to/apollolabsbin/embedded-rust-embassy-analog-sensing-with-adcs-1e2n
#[embassy_executor::task]
async fn temperature_handler(
    adc_peri: ADC1,
    read_pin: PA3,
    heater_tim: TIM15,
    heater_out_pin: PA2,
) {
    let adc = Adc::new(adc_peri, &mut Delay);
    let thermistor = Thermistor::new(
        adc,
        read_pin,
        Resolution::BITS12,
        100_000.0,
        10_000.0,
        Temperature::from_kelvin(3950.0),
    );

    let heater_out = SimplePwm::new(
        heater_tim,
        Some(PwmPin::new_ch1(heater_out_pin, OutputType::PushPull)),
        None,
        None,
        None,
        hz(1),
        CountingMode::EdgeAlignedUp,
    );
    let heater = Heater::new(heater_out, Channel::Ch1);
    let mut hotend = Hotend::new(heater, thermistor);

    hotend.set_temperature(Temperature::from_celsius(30f64));

    let dt = Duration::from_millis(500);
    loop {
        hotend.update(dt);
        Timer::after(dt).await;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // TODO check this configuration. It's in the embassy stm32 examples of ADC. Not so sure why it's needed but without this the
    // program won't run
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8), // SPI1 cksel defaults to pll1_q
            divr: None,
        });
        config.rcc.pll2 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV8), // 100mhz
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.mux.adcsel = mux::Adcsel::PLL2_P;
    }
    let p = embassy_stm32::init(config);

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

    _spawner
        .spawn(input_handler(
            p.USART3, p.PB11, p.PB10, p.DMA1_CH0, p.DMA1_CH1,
        ))
        .unwrap();

    _spawner
        .spawn(temperature_handler(p.ADC1, p.PA3, p.TIM15, p.PA2))
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
