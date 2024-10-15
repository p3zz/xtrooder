#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_stm32::usart::{Config as UartConfig, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use heapless::{String, Vec};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[link_section = ".ram_d3"]
static mut DMA_BUF: [u8; 32] = [0u8; 32];

bind_interrupts!(struct Irqs {
    UART4 => usart::InterruptHandler<peripherals::UART4>;
});

#[embassy_executor::task]
async fn main_task() {
    let p = embassy_stm32::init(Default::default());

    let config = UartConfig::default();
    let mut uart = Uart::new(p.UART4, p.PC11, p.PC10, Irqs, p.DMA1_CH0, p.DMA1_CH1, config).unwrap();

    // unsafe{
    //     DMA_BUF[1] = b'a';
    // }

    // let mut v: Vec<u8, 64> = unsafe { Vec::from_slice(&DMA_BUF).unwrap() };
    // v[0] = b'a';
    // let s = String::from_utf8(v).unwrap();
    let tmp = unsafe { &mut DMA_BUF[..] };

    loop{
        match uart.read_until_idle(tmp).await{
            Ok(_) => {
                let vec = Vec::<u8, 32>::from_slice(tmp).unwrap();
                let str = String::from_utf8(vec).unwrap();
                info!("{}", str.as_str());
            },
            Err(e) => error!("{}", e),
        }
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let executor = EXECUTOR.init(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task()));
    })
}