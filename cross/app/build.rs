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
        timer: String,
    }

    impl PwmConfig{
        pub fn get_frequency(&self) -> u64 {
            self.frequency
        }
    
        pub fn get_timer(&self) -> Option<String> {
            get_string_value(self.timer.clone())
        }
    
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct PwmOutputConfig{
        output: String,
        channel: u8,
    }

    impl PwmOutputConfig{
        pub fn get_output(&self) -> Option<String> {
            get_string_value(self.output.clone())
        }
    
        pub fn get_channel(&self) -> u8 {
            self.channel
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
        pwm: PwmOutputConfig
    }

    impl ThermistorConfig{
        pub fn get_heater(&self) -> HeaterConfig {
            self.heater
        }

        pub fn get_adc(&self) -> AdcConfig {
            self.adc.clone()
        }

        pub fn get_pwm(&self) -> PwmOutputConfig {
            self.pwm.clone()
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct FanConfig{
        pub pwm: PwmOutputConfig
    }

    impl FanConfig{
        pub fn get_pwm(&self) -> PwmOutputConfig{
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
        pub pwm: PwmConfig,
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
    let steppers_x_step_pin = conf.steppers.get_x().get_step().get_pin().expect("Stepper X step pin is missing");
    let steppers_x_dir_pin = conf.steppers.get_x().get_dir().get_pin().expect("Stepper X dir pin is missing");
    let steppers_y_step_pin = conf.steppers.get_y().get_step().get_pin().expect("Stepper Y step pin is missing");
    let steppers_y_dir_pin = conf.steppers.get_y().get_dir().get_pin().expect("Stepper Y dir pin is missing");
    let steppers_z_step_pin = conf.steppers.get_z().get_step().get_pin().expect("Stepper Z step pin is missing");
    let steppers_z_dir_pin = conf.steppers.get_z().get_dir().get_pin().expect("Stepper Z dir pin is missing");
    let steppers_e_step_pin = conf.steppers.get_e().get_step().get_pin().expect("Stepper E step pin is missing");
    let steppers_e_dir_pin = conf.steppers.get_e().get_dir().get_pin().expect("Stepper E dir pin is missing");

    let pwm_timer = conf.pwm.get_timer().expect("PWM timer peripheral is missing");
    let pwm_frequency = conf.pwm.get_frequency();

    let uart_peripheral = conf.uart.get_peripheral().expect("UART peripheral is missing");
    let uart_baudrate = conf.uart.get_baudrate();
    let uart_rx_pin = conf.uart.get_rx().get_pin().expect("UART RX pin is missing");
    let uart_rx_dma = conf.uart.get_rx().get_dma().get_peripheral().expect("UART RX pin is missing");
    let uart_tx_pin = conf.uart.get_tx().get_pin().expect("UART TX pin is missing");
    let uart_tx_dma = conf.uart.get_tx().get_dma().get_peripheral().expect("UART TX pin is missing");

    let hotend_adc_peripheral = conf.hotend.get_adc().get_peripheral().expect("Hotend ADC peripheral is missing");
    let hotend_adc_input_pin = conf.hotend.get_adc().get_input().get_pin().expect("Hotend ADC input pin is missing");
    let hotend_adc_dma = conf.hotend.get_adc().get_dma().get_peripheral().expect("Hotend ADC DMA peripheral is missing");
    let hotend_pwm_output_pin = conf.hotend.get_pwm().get_output().expect("Hotend PWM output pin is missing");
    let hotend_pwm_output_channel = conf.hotend.get_pwm().get_channel();
    let hotend_heater_r0 = conf.hotend.get_heater().get_r0();
    let hotend_heater_r_series = conf.hotend.get_heater().get_r0();
    let hotend_heater_b = conf.hotend.get_heater().get_r_series();
    let hotend_heater_pid = conf.hotend.get_heater().get_pid();

    let heatbed_adc_peripheral = conf.heatbed.get_adc().get_peripheral().expect("Heatbed ADC peripheral is missing");
    let heatbed_adc_input_pin = conf.heatbed.get_adc().get_input().get_pin().expect("Heatbed ADC input pin is missing");
    let heatbed_adc_dma = conf.heatbed.get_adc().get_dma().get_peripheral().expect("Heatbed ADC DMA peripheral is missing");
    let heatbed_pwm_output_pin = conf.heatbed.get_pwm().get_output().expect("Heatbed PWM output pin is missing");
    let heatbed_pwm_output_channel = conf.heatbed.get_pwm().get_channel();
    let heatbed_heater_r0 = conf.heatbed.get_heater().get_r0();
    let heatbed_heater_r_series = conf.heatbed.get_heater().get_r0();
    let heatbed_heater_b = conf.heatbed.get_heater().get_r_series();
    let heatbed_heater_pid = conf.heatbed.get_heater().get_pid();

    let fan_pwm_output_pin = conf.fan.get_pwm().get_output().expect("Fan PWM output pin is missing");
    let fan_pwm_output_channel = conf.fan.get_pwm().get_channel();
    
    let sdcard_spi_peripheral = conf.sdcard.get_spi().get_peripheral().expect("SD-Card peripheral is missing");
    let sdcard_spi_timer = conf.sdcard.get_spi().get_timer().get_peripheral().expect("SD-Card SPI timer is missing");
    let sdcard_spi_mosi = conf.sdcard.get_spi().get_mosi().get_pin().expect("SD-Card SPI MOSI pin is missing");
    let sdcard_spi_miso = conf.sdcard.get_spi().get_miso().get_pin().expect("SD-Card SPI MISO pin is missing");
    let sdcard_spi_cs = conf.sdcard.get_spi().get_cs().get_pin().expect("SD-Card SPI CS pin is missing");

    let string = format!("

use embassy_stm32::Peripherals;
use embassy_stm32::peripherals::*;
use crate::config::*;

pub type XStepPin = {};
pub type XDirPin = {};
pub type YStepPin = {};
pub type YDirPin = {};
pub type ZStepPin = {};
pub type ZDirPin = {};
pub type EStepPin = {};
pub type EDirPin = {};
pub type PwmTimer = {};
pub type UartPeripheral = {};
pub type UartRxPin = {};
pub type UartRxDma = {};
pub type UartTxPin = {};
pub type UartTxDma = {};
pub type HotendAdcPeripheral = {};
pub type HotendAdcInputPin = {};
pub type HotendAdcDma = {};
pub type HotendPwmPin = {};
pub type HeatbedAdcPeripheral = {};
pub type HeatbedAdcInputPin = {};
pub type HeatbedAdcDma = {};
pub type HeatbedPwmPin = {};
pub type FanPwmPin = {};
pub type SdCardSpiPeripheral = {};
pub type SdCardSpiTimer = {};
pub type SdCardSpiMosiPin = {};
pub type SdCardSpiMisoPin = {};
pub type SdCardSpiCsPin = {};

pub fn peripherals_init(p: Peripherals) -> PrinterConfig<
    XStepPin,
    XDirPin,
    YStepPin,
    YDirPin,
    ZStepPin,
    ZDirPin,
    EStepPin,
    EDirPin,
    PwmTimer,
    UartPeripheral,
    UartRxPin,
    UartRxDma,
    UartTxPin,
    UartTxDma,
    HotendAdcPeripheral,
    HotendAdcInputPin,
    HotendAdcDma,
    HotendPwmPin,
    HeatbedAdcPeripheral,
    HeatbedAdcInputPin,
    HeatbedAdcDma,
    HeatbedPwmPin,
    FanPwmPin,
    SdCardSpiPeripheral,
    SdCardSpiTimer,
    SdCardSpiMosiPin,
    SdCardSpiMisoPin,
    SdCardSpiCsPin,
>{{
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
        }},
        pwm: PwmConfig{{
            frequency: {},
            timer: p.{}
        }},
        uart: UartConfig{{
            peripheral: p.{},
            baudrate: {},
            rx: UartPartConfig{{
                pin: p.{},
                dma: p.{}
            }},
            tx: UartPartConfig{{
                pin: p.{},
                dma: p.{}
            }}
        }},
        hotend: ThermistorConfig{{
            adc: AdcConfig {{
                peripheral: p.{},
                input: p.{},
                dma: p.{}
            }},
            pwm: PwmOutputConfig {{
                output: p.{},
                channel: {}
            }},
            heater: HeaterConfig {{
                r_series: {:.2},
                r0: {:.2},
                b: {:.2},
                pid: PidConfig{{
                    k_p: {:.2},
                    k_i: {:.2},
                    k_d: {:.2},
                }}
            }},
        }},
        heatbed: ThermistorConfig{{
            adc: AdcConfig {{
                peripheral: p.{},
                input: p.{},
                dma: p.{}
            }},
            pwm: PwmOutputConfig {{
                output: p.{},
                channel: {}
            }},
            heater: HeaterConfig {{
                r_series: {:.2},
                r0: {:.2},
                b: {:.2},
                pid: PidConfig{{
                    k_p: {:.2},
                    k_i: {:.2},
                    k_d: {:.2},
                }}
            }},
        }},
        fan: FanConfig{{
            pwm: PwmOutputConfig {{
                output: p.{},
                channel: {}
            }}
        }},
        sdcard: SdCardConfig {{
            spi: SpiConfig {{
                peripheral: p.{},
                timer: p.{},
                mosi: p.{},
                miso: p.{},
                cs: p.{},
            }}
        }}
    }}
}}

",
    steppers_x_step_pin,
    steppers_x_dir_pin,
    steppers_y_step_pin,
    steppers_y_dir_pin,
    steppers_z_step_pin,
    steppers_z_dir_pin,
    steppers_e_step_pin,
    steppers_e_dir_pin,
    pwm_timer,
    uart_peripheral,
    uart_rx_pin,
    uart_rx_dma,
    uart_tx_pin,
    uart_tx_dma,
    hotend_adc_peripheral,
    hotend_adc_input_pin,
    hotend_adc_dma,
    hotend_pwm_output_pin,
    heatbed_adc_peripheral,
    heatbed_adc_input_pin,
    heatbed_adc_dma,
    heatbed_pwm_output_pin,
    fan_pwm_output_pin,
    sdcard_spi_peripheral,
    sdcard_spi_timer,
    sdcard_spi_mosi,
    sdcard_spi_miso,
    sdcard_spi_cs,

    steppers_x_step_pin,
    steppers_x_dir_pin,
    steppers_y_step_pin,
    steppers_y_dir_pin,
    steppers_z_step_pin,
    steppers_z_dir_pin,
    steppers_e_step_pin,
    steppers_e_dir_pin,
    pwm_frequency,
    pwm_timer,
    uart_peripheral,
    uart_baudrate,
    uart_rx_pin,
    uart_rx_dma,
    uart_tx_pin,
    uart_tx_dma,
    hotend_adc_peripheral,
    hotend_adc_input_pin,
    hotend_adc_dma,
    hotend_pwm_output_pin,
    hotend_pwm_output_channel,
    hotend_heater_r0,
    hotend_heater_r_series,
    hotend_heater_b,
    hotend_heater_pid.get_k_p(),
    hotend_heater_pid.get_k_i(),
    hotend_heater_pid.get_k_d(),
    heatbed_adc_peripheral,
    heatbed_adc_input_pin,
    heatbed_adc_dma,
    heatbed_pwm_output_pin,
    heatbed_pwm_output_channel,
    heatbed_heater_r0,
    heatbed_heater_r_series,
    heatbed_heater_b,
    heatbed_heater_pid.get_k_p(),
    heatbed_heater_pid.get_k_i(),
    heatbed_heater_pid.get_k_d(),
    fan_pwm_output_pin,
    fan_pwm_output_channel,
    sdcard_spi_peripheral,
    sdcard_spi_timer,
    sdcard_spi_mosi,
    sdcard_spi_miso,
    sdcard_spi_cs,
);
    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("_abcd.rs").to_string_lossy().to_string();
    fs::write(&out_file, string.as_str()).unwrap();
}
