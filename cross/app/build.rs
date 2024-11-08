use std::{
    env,
    fs,
    path::{Path, PathBuf},
};

mod external {
    use std::ops::Not;

    use serde_derive::{Deserialize, Serialize};

    fn get_string_value(s: String) -> Option<String> {
        s.is_empty().not().then_some(s)
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct PinConfig {
        pin: String,
    }

    impl PinConfig {
        pub fn get_pin(&self) -> Option<String> {
            get_string_value(self.pin.clone())
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct PeripheralConfig {
        peripheral: String,
    }

    impl PeripheralConfig {
        pub fn get_peripheral(&self) -> Option<String> {
            get_string_value(self.peripheral.clone())
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone, Copy)]
    pub struct StepperBounds{
        pub min: f64,
        pub max: f64,
    }
    /* stepper */
    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct StepperConfig {
        step: PinConfig,
        dir: PinConfig,
        stepping_mode: String,
        distance_per_step: f64,
        steps_per_revolution: u64,
        bounds: StepperBounds,
        positive_direction: String
    }

    impl StepperConfig {
        pub fn get_step(&self) -> PinConfig {
            self.step.clone()
        }
        pub fn get_dir(&self) -> PinConfig {
            self.dir.clone()
        }
        pub fn get_stepping_mode(&self) -> String{
            self.stepping_mode.clone()
        }
        pub fn get_distance_per_step(&self) -> f64{
            self.distance_per_step
        }
        pub fn get_steps_per_revolution(&self) -> u64{
            self.steps_per_revolution
        }
        pub fn get_bounds(&self) -> StepperBounds{
            self.bounds
        }
        pub fn get_positive_direction(&self) -> String{
            self.positive_direction.clone()
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct StepperConfigs {
        x: StepperConfig,
        y: StepperConfig,
        z: StepperConfig,
        e: StepperConfig,
    }

    impl StepperConfigs {
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
    pub struct UartPartConfig {
        pin: String,
        dma: PeripheralConfig,
    }

    impl UartPartConfig {
        pub fn get_pin(&self) -> Option<String> {
            get_string_value(self.pin.clone())
        }

        pub fn get_dma(&self) -> PeripheralConfig {
            self.dma.clone()
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct UartConfig {
        peripheral: String,
        baudrate: u64,
        rx: UartPartConfig,
        tx: UartPartConfig,
    }

    impl UartConfig {
        pub fn get_peripheral(&self) -> Option<String> {
            get_string_value(self.peripheral.clone())
        }

        pub fn get_baudrate(&self) -> u64 {
            self.baudrate
        }

        pub fn get_tx(&self) -> UartPartConfig {
            self.tx.clone()
        }

        pub fn get_rx(&self) -> UartPartConfig {
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
    pub struct AdcConfig {
        pub peripheral: String,
        pub input: PinConfig,
        pub dma: PeripheralConfig,
    }

    impl AdcConfig {
        pub fn get_peripheral(&self) -> Option<String> {
            get_string_value(self.peripheral.clone())
        }

        pub fn get_input(&self) -> PinConfig {
            self.input.clone()
        }

        pub fn get_dma(&self) -> PeripheralConfig {
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
    pub struct PwmConfig {
        frequency: u64,
        timer: String,
        ch1: String,
        ch2: String,
        ch3: String,
    }

    impl PwmConfig {
        pub fn get_frequency(&self) -> u64 {
            self.frequency
        }

        pub fn get_timer(&self) -> Option<String> {
            get_string_value(self.timer.clone())
        }

        pub fn get_ch1(&self) -> Option<String>{
            get_string_value(self.ch1.clone())
        }

        pub fn get_ch2(&self) -> Option<String>{
            get_string_value(self.ch2.clone())
        }

        pub fn get_ch3(&self) -> Option<String>{
            get_string_value(self.ch3.clone())
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct PwmOutputConfig {
        channel: u8,
    }

    impl PwmOutputConfig {
        pub fn get_channel(&self) -> u8 {
            self.channel
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct SpiConfig {
        peripheral: String,
        clk: PeripheralConfig,
        mosi: PinConfig,
        miso: PinConfig,
        cs: PinConfig,
    }

    impl SpiConfig {
        pub fn get_peripheral(&self) -> Option<String> {
            get_string_value(self.peripheral.clone())
        }

        pub fn get_clk(&self) -> &PeripheralConfig {
            &self.clk
        }

        pub fn get_mosi(&self) -> &PinConfig {
            &self.mosi
        }
        pub fn get_miso(&self) -> &PinConfig {
            &self.miso
        }
        pub fn get_cs(&self) -> &PinConfig {
            &self.cs
        }
    }

    // [hotend.heater.pid]
    // k_p = 0
    // k_i = 0
    // k_d = 0
    #[derive(Default, Debug, Serialize, Deserialize, Clone, Copy)]
    pub struct PidConfig {
        k_p: f64,
        k_i: f64,
        k_d: f64,
    }

    impl PidConfig {
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
    pub struct HeaterConfig {
        r_series: f64,
        r0: f64,
        b: f64,
        pid: PidConfig,
    }

    impl HeaterConfig {
        pub fn get_r_series(&self) -> f64 {
            self.r_series
        }

        pub fn get_r0(&self) -> f64 {
            self.r0
        }

        pub fn get_b(&self) -> f64 {
            self.b
        }
        pub fn get_pid(&self) -> PidConfig {
            self.pid
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct ThermistorConfig {
        heater: HeaterConfig,
        adc: AdcConfig,
        pwm: PwmOutputConfig,
    }

    impl ThermistorConfig {
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
    pub struct FanConfig {
        pub pwm: PwmOutputConfig,
    }

    impl FanConfig {
        pub fn get_pwm(&self) -> PwmOutputConfig {
            self.pwm.clone()
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct SdCardConfig {
        pub spi: SpiConfig,
    }

    impl SdCardConfig {
        pub fn get_spi(&self) -> SpiConfig {
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

fn main() {
    println!("cargo::rerun-if-changed=config/config.toml");
    let path = Path::new("config/config.toml");
    let conf = confy::load_path::<external::MyConfig>(path).expect("Error reading config file");
    let steppers_x_step_pin = conf
        .steppers
        .get_x()
        .get_step()
        .get_pin()
        .expect("Stepper X step pin is missing");
    let steppers_x_dir_pin = conf
        .steppers
        .get_x()
        .get_dir()
        .get_pin()
        .expect("Stepper X dir pin is missing");
    let steppers_x_stepping_mode = conf.steppers.get_x().get_stepping_mode();
    let steppers_x_distance_per_step = conf.steppers.get_x().get_distance_per_step();
    let steppers_x_steps_per_revolution = conf.steppers.get_x().get_steps_per_revolution();
    let steppers_x_bounds = conf.steppers.get_x().get_bounds();
    let steppers_x_positive_direction = conf.steppers.get_x().get_positive_direction();

    let steppers_y_step_pin = conf
        .steppers
        .get_y()
        .get_step()
        .get_pin()
        .expect("Stepper Y step pin is missing");
    let steppers_y_dir_pin = conf
        .steppers
        .get_y()
        .get_dir()
        .get_pin()
        .expect("Stepper Y dir pin is missing");
    let steppers_y_stepping_mode = conf.steppers.get_y().get_stepping_mode();
    let steppers_y_distance_per_step = conf.steppers.get_y().get_distance_per_step();
    let steppers_y_steps_per_revolution = conf.steppers.get_y().get_steps_per_revolution();
    let steppers_y_bounds = conf.steppers.get_y().get_bounds();
    let steppers_y_positive_direction = conf.steppers.get_y().get_positive_direction();
    let steppers_z_step_pin = conf
        .steppers
        .get_z()
        .get_step()
        .get_pin()
        .expect("Stepper Z step pin is missing");
    let steppers_z_dir_pin = conf
        .steppers
        .get_z()
        .get_dir()
        .get_pin()
        .expect("Stepper Z dir pin is missing");
    let steppers_z_stepping_mode = conf.steppers.get_z().get_stepping_mode();
    let steppers_z_distance_per_step = conf.steppers.get_z().get_distance_per_step();
    let steppers_z_steps_per_revolution = conf.steppers.get_z().get_steps_per_revolution();
    let steppers_z_bounds = conf.steppers.get_z().get_bounds();
    let steppers_z_positive_direction = conf.steppers.get_z().get_positive_direction();
    let steppers_e_step_pin = conf
        .steppers
        .get_e()
        .get_step()
        .get_pin()
        .expect("Stepper E step pin is missing");
    let steppers_e_dir_pin = conf
        .steppers
        .get_e()
        .get_dir()
        .get_pin()
        .expect("Stepper E dir pin is missing");
    let steppers_e_stepping_mode = conf.steppers.get_e().get_stepping_mode();
    let steppers_e_distance_per_step = conf.steppers.get_e().get_distance_per_step();
    let steppers_e_steps_per_revolution = conf.steppers.get_e().get_steps_per_revolution();
    let steppers_e_bounds = conf.steppers.get_e().get_bounds();
    let steppers_e_positive_direction = conf.steppers.get_e().get_positive_direction();

    let pwm_timer = conf
        .pwm
        .get_timer()
        .expect("PWM timer peripheral is missing");
    let pwm_frequency = conf.pwm.get_frequency();
    let pwm_ch1 = conf.pwm.get_ch1().expect("PMW ch1 is missing");
    let pwm_ch2 = conf.pwm.get_ch2().expect("PMW ch2 is missing");
    let pwm_ch3 = conf.pwm.get_ch3().expect("PMW ch3 is missing");

    let uart_peripheral = conf
        .uart
        .get_peripheral()
        .expect("UART peripheral is missing");
    let uart_baudrate = conf.uart.get_baudrate();
    let uart_rx_pin = conf
        .uart
        .get_rx()
        .get_pin()
        .expect("UART RX pin is missing");
    let uart_rx_dma = conf
        .uart
        .get_rx()
        .get_dma()
        .get_peripheral()
        .expect("UART RX pin is missing");
    let uart_tx_pin = conf
        .uart
        .get_tx()
        .get_pin()
        .expect("UART TX pin is missing");
    let uart_tx_dma = conf
        .uart
        .get_tx()
        .get_dma()
        .get_peripheral()
        .expect("UART TX pin is missing");

    let hotend_adc_peripheral = conf
        .hotend
        .get_adc()
        .get_peripheral()
        .expect("Hotend ADC peripheral is missing");
    let hotend_adc_input_pin = conf
        .hotend
        .get_adc()
        .get_input()
        .get_pin()
        .expect("Hotend ADC input pin is missing");
    let hotend_adc_dma = conf
        .hotend
        .get_adc()
        .get_dma()
        .get_peripheral()
        .expect("Hotend ADC DMA peripheral is missing");
    let hotend_pwm_output_channel = conf.hotend.get_pwm().get_channel();
    let hotend_heater_r0 = conf.hotend.get_heater().get_r0();
    let hotend_heater_r_series = conf.hotend.get_heater().get_r_series();
    let hotend_heater_b = conf.hotend.get_heater().get_b();
    let hotend_heater_pid = conf.hotend.get_heater().get_pid();

    let heatbed_adc_peripheral = conf
        .heatbed
        .get_adc()
        .get_peripheral()
        .expect("Heatbed ADC peripheral is missing");
    let heatbed_adc_input_pin = conf
        .heatbed
        .get_adc()
        .get_input()
        .get_pin()
        .expect("Heatbed ADC input pin is missing");
    let heatbed_adc_dma = conf
        .heatbed
        .get_adc()
        .get_dma()
        .get_peripheral()
        .expect("Heatbed ADC DMA peripheral is missing");
    let heatbed_pwm_output_channel = conf.heatbed.get_pwm().get_channel();
    let heatbed_heater_r0 = conf.heatbed.get_heater().get_r0();
    let heatbed_heater_r_series = conf.heatbed.get_heater().get_r0();
    let heatbed_heater_b = conf.heatbed.get_heater().get_r_series();
    let heatbed_heater_pid = conf.heatbed.get_heater().get_pid();

    let fan_pwm_output_channel = conf.fan.get_pwm().get_channel();

    let sdcard_spi_peripheral = conf
        .sdcard
        .get_spi()
        .get_peripheral()
        .expect("SD-Card peripheral is missing");
    let sdcard_spi_timer = conf
        .sdcard
        .get_spi()
        .get_clk()
        .get_peripheral()
        .expect("SD-Card SPI timer is missing");
    let sdcard_spi_mosi = conf
        .sdcard
        .get_spi()
        .get_mosi()
        .get_pin()
        .expect("SD-Card SPI MOSI pin is missing");
    let sdcard_spi_miso = conf
        .sdcard
        .get_spi()
        .get_miso()
        .get_pin()
        .expect("SD-Card SPI MISO pin is missing");
    let sdcard_spi_cs = conf
        .sdcard
        .get_spi()
        .get_cs()
        .get_pin()
        .expect("SD-Card SPI CS pin is missing");

    if hotend_pwm_output_channel < 1 || hotend_pwm_output_channel > 4{
        panic!("Hotend PWM channel must be between 1 and 4");
    }
    if heatbed_pwm_output_channel < 1 || heatbed_pwm_output_channel > 4{
        panic!("Heatbed PWM channel must be between 1 and 4");
    }
    if fan_pwm_output_channel < 1 || fan_pwm_output_channel > 4{
        panic!("Fan PWM channel must be between 1 and 4");
    }


    let string = format!(
        "

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
pub type PwmCh1Pin = {};
pub type PwmCh2Pin = {};
pub type PwmCh3Pin = {};
pub type UartPeripheral = {};
pub type UartRxPin = {};
pub type UartRxDma = {};
pub type UartTxPin = {};
pub type UartTxDma = {};
pub type HotendAdcPeripheral = {};
pub type HotendAdcInputPin = {};
pub type HotendAdcDma = {};
pub type HeatbedAdcPeripheral = {};
pub type HeatbedAdcInputPin = {};
pub type HeatbedAdcDma = {};
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
    PwmCh1Pin,
    PwmCh2Pin,
    PwmCh3Pin,
    UartPeripheral,
    UartRxPin,
    UartRxDma,
    UartTxPin,
    UartTxDma,
    HotendAdcPeripheral,
    HotendAdcInputPin,
    HotendAdcDma,
    HeatbedAdcPeripheral,
    HeatbedAdcInputPin,
    HeatbedAdcDma,
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
                stepping_mode: \"{}\",
                distance_per_step: {:.2},
                steps_per_revolution: {},
                bounds: ({:.2}, {:.2}),
                positive_direction: \"{}\"
            }},
            y: StepperConfig{{
                step_pin: p.{},
                dir_pin: p.{},
                stepping_mode: \"{}\",
                distance_per_step: {:.2},
                steps_per_revolution: {},
                bounds: ({:.2}, {:.2}),
                positive_direction: \"{}\"
            }},
            z: StepperConfig{{
                step_pin: p.{},
                dir_pin: p.{},
                stepping_mode: \"{}\",
                distance_per_step: {:.2},
                steps_per_revolution: {},
                bounds: ({:.2}, {:.2}),
                positive_direction: \"{}\"
            }},
            e: StepperConfig{{
                step_pin: p.{},
                dir_pin: p.{},
                stepping_mode: \"{}\",
                distance_per_step: {:.2},
                steps_per_revolution: {},
                bounds: ({:.2}, {:.2}),
                positive_direction: \"{}\"
            }}
        }},
        pwm: PwmConfig{{
            frequency: {},
            timer: p.{},
            ch1: p.{},
            ch2: p.{},
            ch3: p.{},
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
                channel: {}
            }}
        }},
        sdcard: SdCardConfig {{
            spi: SpiConfig {{
                peripheral: p.{},
                clk: p.{},
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
        pwm_ch1,
        pwm_ch2,
        pwm_ch3,
        uart_peripheral,
        uart_rx_pin,
        uart_rx_dma,
        uart_tx_pin,
        uart_tx_dma,
        hotend_adc_peripheral,
        hotend_adc_input_pin,
        hotend_adc_dma,
        heatbed_adc_peripheral,
        heatbed_adc_input_pin,
        heatbed_adc_dma,
        sdcard_spi_peripheral,
        sdcard_spi_timer,
        sdcard_spi_mosi,
        sdcard_spi_miso,
        sdcard_spi_cs,
        steppers_x_step_pin,
        steppers_x_dir_pin,
        steppers_x_stepping_mode.as_str(),
        steppers_x_distance_per_step,
        steppers_x_steps_per_revolution,
        steppers_x_bounds.min,
        steppers_x_bounds.max,
        steppers_x_positive_direction,
        steppers_y_step_pin,
        steppers_y_dir_pin,
        steppers_y_stepping_mode.as_str(),
        steppers_y_distance_per_step,
        steppers_y_steps_per_revolution,
        steppers_y_bounds.min,
        steppers_y_bounds.max,
        steppers_y_positive_direction,
        steppers_z_step_pin,
        steppers_z_dir_pin,
        steppers_z_stepping_mode.as_str(),
        steppers_z_distance_per_step,
        steppers_z_steps_per_revolution,
        steppers_z_bounds.min,
        steppers_z_bounds.max,
        steppers_z_positive_direction,
        steppers_e_step_pin,
        steppers_e_dir_pin,
        steppers_e_stepping_mode.as_str(),
        steppers_e_distance_per_step,
        steppers_e_steps_per_revolution,
        steppers_e_bounds.min,
        steppers_e_bounds.max,
        steppers_e_positive_direction,
        pwm_frequency,
        pwm_timer,
        pwm_ch1,
        pwm_ch2,
        pwm_ch3,
        uart_peripheral,
        uart_baudrate,
        uart_rx_pin,
        uart_rx_dma,
        uart_tx_pin,
        uart_tx_dma,
        hotend_adc_peripheral,
        hotend_adc_input_pin,
        hotend_adc_dma,
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
        heatbed_pwm_output_channel,
        heatbed_heater_r0,
        heatbed_heater_r_series,
        heatbed_heater_b,
        heatbed_heater_pid.get_k_p(),
        heatbed_heater_pid.get_k_i(),
        heatbed_heater_pid.get_k_d(),
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
