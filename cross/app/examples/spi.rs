#![no_std]
#![no_main]

use app::sdcard::SdmmcDevice;
use app::utils::stopwatch::Clock;
use defmt::{info, panic};
use embassy_executor::Spawner;
use embassy_stm32::sdmmc::Sdmmc;
use embassy_stm32::time::mhz;
use embassy_stm32::{bind_interrupts, peripherals, sdmmc, Config};
use fs::filesystem::files::Mode;
use fs::volume_mgr::{VolumeIdx, VolumeManager};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SDMMC1 => sdmmc::InterruptHandler<peripherals::SDMMC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
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
            divq: Some(PllDiv::DIV4), // default clock chosen by SDMMCSEL. 200 Mhz
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
    info!("Hello World!");

    let mut sdmmc = Sdmmc::new_4bit(
        p.SDMMC1,
        Irqs,
        p.PC12,
        p.PD2,
        p.PC8,
        p.PC9,
        p.PC10,
        p.PC11,
        Default::default(),
    );

    // Should print 400kHz for initialization
    info!("Configured clock: {}", sdmmc.clock().0);

    sdmmc.init_card(mhz(25)).await.unwrap();

    let clock = Clock::new();
    let device = SdmmcDevice::new(sdmmc);

    let mut volume_mgr = VolumeManager::new(device, clock);

    let mut volume0 = match volume_mgr.open_volume(VolumeIdx(0)).await {
        Ok(v) => v,
        Err(_) => panic!("Cannot find module"),
    };

    // info!("Volume 0: {:?}", volume0);
    // Open the root directory (mutably borrows from the volume).
    let mut root_dir = match volume0.open_root_dir() {
        Ok(d) => d,
        Err(_) => panic!("Cannot open root dir"),
    };
    // Open a file called "MY_FILE.TXT" in the root directory
    // This mutably borrows the directory.
    let mut my_file = match root_dir
        .open_file_in_dir("MY_FILE.TXT", Mode::ReadOnly)
        .await
    {
        Ok(f) => f,
        Err(_) => panic!("Cannot open file"),
    };
    // Print the contents of the file
    while !my_file.is_eof() {
        let mut buffer = [0u8; 32];
        match my_file.read(&mut buffer).await {
            Ok(num_bytes) => {
                for b in &buffer[0..num_bytes] {
                    info!("{}", *b as char);
                }
            }
            Err(_) => todo!(),
        }
    }

    loop {}
}
