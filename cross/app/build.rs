use std::{env, fs, path::{Path, PathBuf}, str::FromStr};
use proc_macro2::TokenStream;
use quote::quote;
use serde_derive::{Serialize, Deserialize};


#[derive(Default, Debug, Serialize, Deserialize)]
struct PinConfig{
    pin: String
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct PeripheralConfig{
    peripheral: String
}

/* stepper */
#[derive(Default, Debug, Serialize, Deserialize)]
struct StepperConfig{
    step: PinConfig,
    dir: PinConfig,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct StepperConfigs{
    x: StepperConfig,
    y: StepperConfig,
    z: StepperConfig,
    e: StepperConfig,
}

/* UART */
#[derive(Default, Debug, Serialize, Deserialize)]
struct UartPartConfig{
    pin: String,
    dma: PeripheralConfig,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct UartConfig{
    peripheral: String,
    baudrate: u64,
    rx: UartPartConfig,
    tx: UartPartConfig
}

/* ADC */
// [hotend.adc]
// peripheral = "ADC1"

// [hotend.adc.input]
// pin = "PA1"

// [hotend.adc.dma]
// peripheral = ""
#[derive(Default, Debug, Serialize, Deserialize)]
struct AdcConfig{
    peripheral: String,
    input: PinConfig,
    dma: PeripheralConfig,
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
#[derive(Default, Debug, Serialize, Deserialize)]
struct PwmConfig{
    frequency: u64,
    timer: PeripheralConfig,
    channel0: PinConfig,
    channel1: PinConfig,
    channel2: PinConfig,
    channel3: PinConfig,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct SpiConfig{
    peripheral: String,
    timer: PeripheralConfig,
    mosi: PinConfig,
    miso: PinConfig,
    cs: PinConfig,
}

// [hotend.heater.pid]
// k_p = 0
// k_i = 0
// k_d = 0
#[derive(Default, Debug, Serialize, Deserialize)]
struct PidConfig{
    k_p: f64,
    k_i: f64,
    k_d: f64,
}

// [hotend.heater]
// r_series=0
// r0=0
// b = 0
#[derive(Default, Debug, Serialize, Deserialize)]
struct HeaterConfig{
    r_series: u64,
    r0: u64,
    b: u64,
    pid: PidConfig
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct ThermistorConfig{
    heater: HeaterConfig,
    adc: AdcConfig,
    pwm: PwmConfig
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct FanConfig{
    pwm: PwmConfig
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct SdCardConfig{
    spi: SpiConfig
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct MyConfig {
    // version: u8,
    steppers: StepperConfigs,
    uart: UartConfig,
    hotend: ThermistorConfig,
    heatbed: ThermistorConfig,
    fan: FanConfig,
    sdcard: SdCardConfig,
}

fn main() -> () {
    println!("cargo::rerun-if-changed=config/config.toml");
    let path = Path::new("config/config.toml");
    let conf = confy::load_path::<MyConfig>(path).expect("Error reading config file");
    let mut string = String::new();
    string += format!("use embassy_stm32::peripherals::{};\n", conf.steppers.x.step.pin).as_str();
    string += "use embassy_stm32::Config;\n";
    string += "pub struct PrinterConfig{\n";
    string += format!("\tpub step_pin: {},\n", conf.steppers.x.step.pin).as_str();
    string += "}\n";
    string += "\n";
    string += "pub fn peripherals_init() -> PrinterConfig{\n";
    string += "\tlet p = embassy_stm32::init(Config::default());\n";
    string += "\tPrinterConfig {\n";
    string += format!("\t\t step_pin: p.{},\n", conf.steppers.x.step.pin).as_str();
    string += "\t}\n";
    string += "}\n";
    // format!(string, "{}");
    // g.extend(quote!{
    //     use embassy_stm32::peripherals::#step_pin;
    // });
    // let bytes = fs::read(path).expect("File not found");
    // let str = String::from_utf8(bytes).expect("Invalid bytes");
    // println!("{}", str);
    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("_abcd.rs").to_string_lossy().to_string();
    fs::write(&out_file, string.as_str()).unwrap();
}
