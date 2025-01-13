#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::usart::{Config as UartConfig, Uart, UartRx};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use heapless::{String, Vec};
use static_cell::{ConstStaticCell, StaticCell};
use {defmt_rtt as _, panic_probe as _};

#[cfg(feature = "defmt-log")]
use defmt::*;

#[link_section = ".ram_d3"]
static DMA_BUF: StaticCell<[u8; 32]> = StaticCell::new();

bind_interrupts!(struct Irqs {
    UART4 => usart::InterruptHandler<peripherals::UART4>;
});

#[embassy_executor::task]
async fn main_task() {
    let p = embassy_stm32::init(Default::default());

    let mut config = UartConfig::default();
    config.baudrate = 19200;
    let mut uart = Uart::new(
        p.UART4, p.PC11, p.PC10, Irqs, p.DMA1_CH0, p.DMA1_CH1, config,
    )
    .unwrap();

    let tmp = DMA_BUF.init_with(|| [0u8; 32]);

    loop {
        match uart.read_until_idle(tmp).await {
            Ok(_) => {
                let vec = Vec::<u8, 32>::from_slice(tmp).unwrap();
                let s = String::from_utf8(vec).unwrap();
                #[cfg(feature = "defmt-log")]
                info!("{}", s.as_str());
            }
            Err(e) => {
                #[cfg(feature = "defmt-log")]
                error!("{}", e)
            }
        };
        tmp.fill(0u8);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    #[cfg(feature = "defmt-log")]
    info!("Hello World!");

    spawner.spawn(main_task()).unwrap();
}
