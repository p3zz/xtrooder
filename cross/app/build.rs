use std::{env, fmt::Display, fs, path::{Path, PathBuf}, str::FromStr};
use external::{stringify_pin, PinConfig};
use proc_macro2::TokenStream;
use quote::quote;

mod external{
    use std::ops::Not;

    use serde_derive::{Serialize, Deserialize};

    fn get_string_value(s: String) -> Option<String> {
        s.is_empty().not().then(|| s)
    }

    pub fn stringify_pin(pin: Option<PinConfig>) -> String{
        match pin{
            Some(p) => {
                match p.get_pin(){
                    Some(s) => format!(", {}", s),
                    None => String::new(),
                }
            },
            None => String::new(),
        }
    }

    pub fn stringify_peripheral(peripheral: Option<PeripheralConfig>) -> String{
        match peripheral{
            Some(p) => {
                match p.get_peripheral(){
                    Some(s) => format!(", {}", s),
                    None => String::new(),
                }
            },
            None => String::new(),
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize ,Clone)]
    pub struct PinConfig{
        pin: String
    }

    impl PinConfig{
        pub fn get_pin(&self) -> Option<String> {
            get_string_value(self.pin.clone())
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct PeripheralConfig{
        peripheral: String
    }

    impl PeripheralConfig{
        pub fn get_peripheral(&self) -> Option<String> {
            get_string_value(self.peripheral.clone())
        }
    }

    /* stepper */
    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct StepperConfig{
        step: PinConfig,
        dir: PinConfig,
    }

    impl StepperConfig{
        pub fn get_step(&self) -> PinConfig {
            self.step.clone()
        }
        pub fn get_dir(&self) -> PinConfig {
            self.dir.clone()
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct StepperConfigs{
        x: StepperConfig,
        y: StepperConfig,
        z: StepperConfig,
        e: StepperConfig,
    }

    impl StepperConfigs{
        pub fn get_x(&self) -> StepperConfig {
            self.x.clone()
        }
        pub fn get_y(&self) -> StepperConfig {
            self.y.clone()
        }
        pub fn get_z(&self) -> StepperConfig {
            self.z.clone()
        }
        pub fn get_e(&self) -> StepperConfig {
            self.e.clone()
        }
    }

    /* UART */
    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct UartPartConfig{
        pin: String,
        dma: PeripheralConfig,
    }

    impl UartPartConfig{
        pub fn get_pin(&self) -> Option<String>{
            get_string_value(self.pin.clone())
        }

        pub fn get_dma(&self) -> PeripheralConfig{
            self.dma.clone()
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct UartConfig{
        peripheral: String,
        baudrate: u64,
        rx: UartPartConfig,
        tx: UartPartConfig
    }

    impl UartConfig{
        pub fn get_peripheral(&self) -> Option<String>{
            get_string_value(self.peripheral.clone())
        }

        pub fn get_baudrate(&self) -> u64{
            self.baudrate
        }

        pub fn get_tx(&self) -> UartPartConfig{
            self.tx.clone()
        }

        pub fn get_rx(&self) -> UartPartConfig{
            self.rx.clone()
        }
    }

    /* ADC */
    // [hotend.adc]
    // peripheral = "ADC1"

    // [hotend.adc.input]
    // pin = "PA1"

    // [hotend.adc.dma]
    // peripheral = ""
    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct AdcConfig{
        pub peripheral: String,
        pub input: PinConfig,
        pub dma: PeripheralConfig,
    }

    impl AdcConfig{
        pub fn get_peripheral(&self) -> Option<String>{
            get_string_value(self.peripheral.clone())
        }
    
        pub fn get_input(&self) -> PinConfig {
            self.input.clone()
        }
    
        pub fn get_dma(&self) -> PeripheralConfig{
            self.dma.clone()
        }
    }

    // [hotend.pwm]
    // frequency=0

    // [hotend.pwm.timer]
    // peripheral = ""

    // [hotend.pwm.channel0]
    // pin = ""

    // [hotend.pwm.channel1]
    // pin = ""

    // [hotend.pwm.channel2]
    // pin = ""

    // [hotend.pwm.channel3]
    // pin = ""
    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct PwmConfig{
        frequency: u64,
        timer: PeripheralConfig,
        channel0: Option<PinConfig>,
        channel1: Option<PinConfig>,
        channel2: Option<PinConfig>,
        channel3: Option<PinConfig>,
    }

    impl PwmConfig{
        pub fn get_frequency(&self) -> u64 {
            self.frequency
        }
    
        pub fn get_timer(&self) -> PeripheralConfig {
            self.timer.clone()
        }
    
        pub fn get_channel0(&self) -> Option<PinConfig>{
            self.channel0.clone()
        }
        pub fn get_channel1(&self) -> Option<PinConfig>{
            self.channel1.clone()
        }
        pub fn get_channel2(&self) -> Option<PinConfig>{
            self.channel2.clone()
        }
        pub fn get_channel3(&self) -> Option<PinConfig>{
            self.channel3.clone()
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct SpiConfig{
        peripheral: String,
        timer: PeripheralConfig,
        mosi: PinConfig,
        miso: PinConfig,
        cs: PinConfig,
    }

    impl SpiConfig{
        pub fn get_peripheral(&self) -> Option<String> {
            get_string_value(self.peripheral.clone())
        }
    
        pub fn get_timer(&self) -> &PeripheralConfig {
            &self.timer
        }
    
        pub fn get_mosi(&self) -> &PinConfig{
            &self.mosi
        }
        pub fn get_miso(&self) -> &PinConfig{
            &self.miso
        }
        pub fn get_cs(&self) -> &PinConfig{
            &self.cs
        }
    }

    // [hotend.heater.pid]
    // k_p = 0
    // k_i = 0
    // k_d = 0
    #[derive(Default, Debug, Serialize, Deserialize, Clone, Copy)]
    pub struct PidConfig{
        k_p: f64,
        k_i: f64,
        k_d: f64,
    }

    impl PidConfig{
        pub fn get_k_p(&self) -> f64 {
            self.k_p
        }

        pub fn get_k_i(&self) -> f64 {
            self.k_i
        }
        pub fn get_k_d(&self) -> f64 {
            self.k_d
        }
    }

    // [hotend.heater]
    // r_series=0
    // r0=0
    // b = 0
    #[derive(Default, Debug, Serialize, Deserialize, Clone, Copy)]
    pub struct HeaterConfig{
        r_series: u64,
        r0: u64,
        b: u64,
        pid: PidConfig
    }


    impl HeaterConfig{
        pub fn get_r_series(&self) -> u64 {
            self.r_series
        }

        pub fn get_r0(&self) -> u64 {
            self.r0
        }

        pub fn get_b(&self) -> u64 {
            self.b
        }
        pub fn get_pid(&self) -> PidConfig {
            self.pid
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct ThermistorConfig{
        heater: HeaterConfig,
        adc: AdcConfig,
        pwm: PwmConfig
    }

    impl ThermistorConfig{
        pub fn get_heater(&self) -> HeaterConfig {
            self.heater
        }

        pub fn get_adc(&self) -> AdcConfig {
            self.adc.clone()
        }

        pub fn get_pwm(&self) -> PwmConfig {
            self.pwm.clone()
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct FanConfig{
        pub pwm: PwmConfig
    }

    impl FanConfig{
        pub fn get_pwm(&self) -> PwmConfig{
            self.pwm.clone()
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct SdCardConfig{
        pub spi: SpiConfig
    }

    impl SdCardConfig{
        pub fn get_spi(&self) -> SpiConfig{
            self.spi.clone()
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct MyConfig {
        pub steppers: StepperConfigs,
        pub uart: UartConfig,
        pub hotend: ThermistorConfig,
        pub heatbed: ThermistorConfig,
        pub fan: FanConfig,
        pub sdcard: SdCardConfig,
    }
}

fn main() -> () {
    println!("cargo::rerun-if-changed=config/config.toml");
    let path = Path::new("config/config.toml");
    let conf = confy::load_path::<external::MyConfig>(path).expect("Error reading config file");
    let mut string = String::new();
    let steppers_imports = format!("{}, {}, {}, {}, {}, {}, {}, {}",
        conf.steppers.get_x().get_step().get_pin().expect("Stepper X step pin is missing"),
        conf.steppers.get_x().get_dir().get_pin().expect("Stepper X dir pin is missing"),
        conf.steppers.get_y().get_step().get_pin().expect("Stepper Y step pin is missing"),
        conf.steppers.get_y().get_dir().get_pin().expect("Stepper Y dir pin is missing"),
        conf.steppers.get_z().get_step().get_pin().expect("Stepper Z step pin is missing"),
        conf.steppers.get_z().get_dir().get_pin().expect("Stepper Z dir pin is missing"),
        conf.steppers.get_e().get_step().get_pin().expect("Stepper E step pin is missing"),
        conf.steppers.get_e().get_dir().get_pin().expect("Stepper E dir pin is missing"),
    );
    let uart_imports = format!("{}, {}, {}, {}, {}", 
        conf.uart.get_peripheral().expect("UART peripheral is missing"),
        conf.uart.get_rx().get_pin().expect("UART RX pin is missing"),
        conf.uart.get_rx().get_dma().get_peripheral().expect("UART RX DMA peripheral is missing"),
        conf.uart.get_tx().get_pin().expect("UART TX pin is missing"),
        conf.uart.get_tx().get_dma().get_peripheral().expect("UART TX DMA peripheral is missing"),
    );
    
    if (conf.heatbed.get_pwm().get_channel0().is_none() || conf.heatbed.get_pwm().get_channel0().unwrap().get_pin().is_none()) &&
        (conf.heatbed.get_pwm().get_channel1().is_none() || conf.heatbed.get_pwm().get_channel1().unwrap().get_pin().is_none()) &&
        (conf.heatbed.get_pwm().get_channel2().is_none() || conf.heatbed.get_pwm().get_channel2().unwrap().get_pin().is_none()) &&
        (conf.heatbed.get_pwm().get_channel3().is_none() || conf.heatbed.get_pwm().get_channel3().unwrap().get_pin().is_none()) {
        panic!("Heatbed is missing a valid PWM channel");
    }
    let mut heatbed_imports = format!("{}, {}, {}, {}", 
        conf.heatbed.get_adc().get_peripheral().expect("Heatbed ADC peripheral is missing"),
        conf.heatbed.get_adc().get_input().get_pin().expect("Heatbed ADC input pin is missing"),
        conf.heatbed.get_adc().get_dma().get_peripheral().expect("Heatbed ADC DMA peripheral is missing"),
        conf.heatbed.get_pwm().get_timer().get_peripheral().expect("Heatbed PWM timer is missing"),
    );
    
    heatbed_imports += stringify_pin(conf.heatbed.get_pwm().get_channel0()).as_str();
    heatbed_imports += stringify_pin(conf.heatbed.get_pwm().get_channel1()).as_str();
    heatbed_imports += stringify_pin(conf.heatbed.get_pwm().get_channel2()).as_str();
    heatbed_imports += stringify_pin(conf.heatbed.get_pwm().get_channel3()).as_str();

    if (conf.hotend.get_pwm().get_channel0().is_none() || conf.hotend.get_pwm().get_channel0().unwrap().get_pin().is_none()) &&
        (conf.hotend.get_pwm().get_channel1().is_none() || conf.hotend.get_pwm().get_channel1().unwrap().get_pin().is_none()) &&
        (conf.hotend.get_pwm().get_channel2().is_none() || conf.hotend.get_pwm().get_channel2().unwrap().get_pin().is_none()) &&
        (conf.hotend.get_pwm().get_channel3().is_none() || conf.hotend.get_pwm().get_channel3().unwrap().get_pin().is_none()) {
        panic!("Hotend is missing a valid PWM channel");
    }

    let mut hotend_imports = format!("{}, {}, {}, {}", 
        conf.hotend.get_adc().get_peripheral().expect("Hotend ADC peripheral is missing"),
        conf.hotend.get_adc().get_input().get_pin().expect("Hotend ADC input pin is missing"),
        conf.hotend.get_adc().get_dma().get_peripheral().expect("Hotend ADC DMA peripheral is missing"),
        conf.hotend.get_pwm().get_timer().get_peripheral().expect("Hotend PWM timer is missing"),
    );

    hotend_imports += stringify_pin(conf.hotend.get_pwm().get_channel0()).as_str();
    hotend_imports += stringify_pin(conf.hotend.get_pwm().get_channel1()).as_str();
    hotend_imports += stringify_pin(conf.hotend.get_pwm().get_channel2()).as_str();
    hotend_imports += stringify_pin(conf.hotend.get_pwm().get_channel3()).as_str();
    
    let mut fan_imports = format!("{}", 
        conf.fan.get_pwm().get_timer().get_peripheral().expect("Fan PWM timer peripheral is missing"),
    );

    fan_imports += stringify_pin(conf.fan.get_pwm().get_channel0()).as_str();
    fan_imports += stringify_pin(conf.fan.get_pwm().get_channel1()).as_str();
    fan_imports += stringify_pin(conf.fan.get_pwm().get_channel2()).as_str();
    fan_imports += stringify_pin(conf.fan.get_pwm().get_channel3()).as_str();

    let sdcard_imports = format!("{}, {}, {}, {}, {}", 
        conf.sdcard.get_spi().get_peripheral().expect("SD-Card peripheral is missing"),
        conf.sdcard.get_spi().get_timer().get_peripheral().expect("SD-Card SPI timer is missing"),
        conf.sdcard.get_spi().get_mosi().get_pin().expect("SD-Card SPI MOSI pin is missing"),
        conf.sdcard.get_spi().get_miso().get_pin().expect("SD-Card SPI MISO pin is missing"),
        conf.sdcard.get_spi().get_cs().get_pin().expect("SD-Card SPI CS pin is missing"),
    );

    let imports = format!("{}, {}, {}, {}, {}", steppers_imports, uart_imports, hotend_imports, heatbed_imports, sdcard_imports);
        
    string += format!("

use embassy_stm32::Peripherals;
use embassy_stm32::peripherals::*;
use crate::config::*;

pub fn peripherals_init(p: Peripherals) -> PrinterConfig<{}>{{
    PrinterConfig{{
        steppers: SteppersConfig{{
            x: StepperConfig{{
                step_pin: p.{},
                dir_pin: p.{},
            }},
            y: StepperConfig{{
                step_pin: p.{},
                dir_pin: p.{},
            }},
            z: StepperConfig{{
                step_pin: p.{},
                dir_pin: p.{},
            }},
            e: StepperConfig{{
                step_pin: p.{},
                dir_pin: p.{},
            }}
        }}
    }}
}}

",
    imports,
    conf.steppers.get_x().get_step().get_pin().unwrap(),
    conf.steppers.get_x().get_dir().get_pin().unwrap(),
    conf.steppers.get_y().get_step().get_pin().unwrap(),
    conf.steppers.get_y().get_dir().get_pin().unwrap(),
    conf.steppers.get_z().get_step().get_pin().unwrap(),
    conf.steppers.get_z().get_dir().get_pin().unwrap(),
    conf.steppers.get_e().get_step().get_pin().unwrap(),
    conf.steppers.get_e().get_dir().get_pin().unwrap()
).as_str();
    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("_abcd.rs").to_string_lossy().to_string();
    fs::write(&out_file, string.as_str()).unwrap();
}
