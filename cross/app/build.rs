use std::{env, fs, path::{Path, PathBuf}, str::FromStr};
use proc_macro2::TokenStream;
use quote::quote;
use serde_derive::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize)]
struct StepperConfig{
    step: String,
    dir: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct SteppersConfig{
    x: StepperConfig,
    y: StepperConfig,
    z: StepperConfig,
    e: StepperConfig,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct MyConfig {
    // version: u8,
    steppers: SteppersConfig,
}

// fn main() -> () {
    // let to_write = "abcd".as_bytes();
    // let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    // let out_file = out_dir.join("_abcd.rs").to_string_lossy().to_string();
    // fs::write(&out_file, to_write).unwrap();
//     // ========
//     // Write generated.rs
//     // let path = Path::new("config/config.toml");
//     // let cfg: MyConfig = confy::load_path(path).expect("Config file not found");
//     // dbg!(cfg);
//     // Ok(())
// }

fn main() -> () {
    println!("cargo::rerun-if-changed=config/config.toml");
    let path = Path::new("config/config.toml");
    let conf = confy::load_path::<MyConfig>(path).expect("Error reading config file");
    let mut string = String::new();
    string += format!("use embassy_stm32::peripherals::{};\n", conf.steppers.x.step).as_str();
    string += "use embassy_stm32::Config;\n";
    string += "pub struct PrinterConfig{\n";
    string += format!("\tpub step_pin: {},\n", conf.steppers.x.step).as_str();
    string += "}\n";
    string += "\n";
    string += "pub fn peripherals_init() -> PrinterConfig{\n";
    string += "\tlet p = embassy_stm32::init(Config::default());\n";
    string += "\tPrinterConfig {\n";
    string += format!("\t\t step_pin: p.{},\n", conf.steppers.x.step).as_str();
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
