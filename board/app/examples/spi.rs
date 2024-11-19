#![no_std]
#![no_main]

use core::cell::RefCell;

use app::Clock;
use defmt::{info, panic};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::Spi;
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_time::{Delay, Duration, Timer};
// use embedded_hal_1::spi::SpiBus;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_stm32::{spi, Config};
use embedded_sdmmc::{Mode, SdCard, VolumeIdx, VolumeManager};
use heapless::{String, Vec};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// #[embassy_executor::task]
// async fn main_task(mut spi: spi::Spi<'static, Blocking>) {
//     for n in 0u32.. {
//         let mut write: String<128> = String::new();
//         core::write!(&mut write, "Hello DMA World {}!\r\n", n).unwrap();
//         unsafe {
//             let result = spi.blocking_transfer_in_place(write.as_bytes_mut());
//             if let Err(_) = result {
//                 defmt::panic!("crap");
//             }
//         }
//         info!("read via spi: {}", from_utf8(write.as_bytes()).unwrap());
//     }
// }

// static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello World!");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8), // used by SPI3. 100Mhz.
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    let p = embassy_stm32::init(config);

    static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<'static, Blocking>>>> = StaticCell::new();
    let spi = spi::Spi::new_blocking(p.SPI1, p.PB3, p.PB5, p.PB4, Default::default());
    let spi_bus = NoopMutex::new(RefCell::new(spi));
    let spi_bus = SPI_BUS.init(spi_bus);

    // Device 1, using embedded-hal compatible driver for ST7735 LCD display
    let cs_pin = Output::new(p.PC12, Level::High, Speed::Low);

    let spi = SpiDevice::new(spi_bus, cs_pin);
    let sdcard = SdCard::new(spi, Delay);
    match sdcard.get_card_type() {
        Some(t) => info!("{}", t as u32),
        None => panic!("cannot read card type"),
    };

    let clock = Clock::new();
    let mut volume_mgr = VolumeManager::new(sdcard, clock);

    let volume0 = match volume_mgr.open_raw_volume(VolumeIdx(0)) {
        Ok(v) => v,
        Err(_) => panic!("Cannot find module"),
    };

    // info!("Volume 0: {:?}", volume0);main]
    // Open the root directory (mutably borrows from the volume).
    let root_dir = match volume_mgr.open_root_dir(volume0) {
        Ok(d) => d,
        Err(_) => {
            volume_mgr.close_volume(volume0).unwrap();
            panic!("Cannot open root dir")
        }
    };
    // Open a file called "MY_FILE.TXT" in the root directory
    // This mutably borrows the directory.
    let my_file = match volume_mgr.open_file_in_dir(root_dir, "MY_FILE.TXT", Mode::ReadOnly) {
        Ok(f) => f,
        Err(_) => {
            volume_mgr.close_dir(root_dir).unwrap();
            volume_mgr.close_volume(volume0).unwrap();
            panic!("Cannot open file");
        }
    };

    let mut buf = [0u8; 64];

    while let Ok(n) = volume_mgr.read(my_file, &mut buf) {
        if n == 0 {
            break;
        }
        let vec: Vec<u8, 64> = Vec::from_slice(&buf).expect("Malformed string");
        let str = String::from_utf8(vec).unwrap();
        info!("{}", str.as_str());
        // for b in &buf[0..n] {
        // info!("{}", *b as char);
        // }
    }

    // Print the contents of the file
    // while !my_file.is_eof() {
    //     let mut buffer = [0u8; 32];
    //     match my_file.read(&mut buffer).await {
    //         Ok(num_bytes) => {
    //             for b in &buffer[0..num_bytes] {
    //                 info!("{}", *b as char);
    //             }
    //         }
    //         Err(_) => todo!(),
    //     }
    // }

    volume_mgr.close_file(my_file).unwrap();
    volume_mgr.close_dir(root_dir).unwrap();
    volume_mgr.close_volume(volume0).unwrap();
    // let mut spi_config = spi::Config::default();
    // spi_config.frequency = mhz(1);

    // let mut spi = spi::Spi::new_blocking(p.SPI1, p.PA5, p.PB5, p.PA6, spi_config);

    // for n in 0u32..20u32 {
    //     let mut write: String<128> = String::new();
    //     core::write!(&mut write, "Hello DMA World {}!\r\n", n).unwrap();
    //     unsafe {
    //         let result = spi.blocking_transfer_in_place(write.as_bytes_mut());
    //         if let Err(_) = result {
    //             defmt::panic!("crap");
    //         }
    //     }
    //     info!("read via spi: {}", from_utf8(write.as_bytes()).unwrap());
    // }

    loop {
        info!("main loop");
        Timer::after(Duration::from_secs(1)).await;
    }
}
