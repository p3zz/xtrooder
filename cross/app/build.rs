use std::{
    env,
    fs,
    path::{Path, PathBuf},
};

use math::{common::RotationDirection, measurements::{Distance, Length, Resistance, Speed}};
use proc_macro2::Span;
use stepper::{motion::Positioning, stepper::SteppingMode};
use quote::quote;
use syn::Ident;

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

//     [motion]
// arc_unit_length = 0.0
// feedrate = 0.0
// positioning = "absolute"

// [motion.retraction]
// feedrate = 0.0
// length = 0.0
// z_lift = 0.0

// [motion.recover]
// feedrate = 0.0
// length = 0.0


    #[derive(Default, Debug, Serialize, Deserialize, Clone, Copy)]
    pub struct RecoverMotionConfig{
        feedrate: f64,
        length: f64,
    }

    impl RecoverMotionConfig{
        pub fn get_feedrate(&self) -> f64 {
            self.feedrate
        }

        pub fn get_length(&self) -> f64 {
            self.length
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone, Copy)]
    pub struct RetractionMotionConfig{
        feedrate: f64,
        length: f64,
        z_lift: f64,
    }

    impl RetractionMotionConfig{
        pub fn get_feedrate(&self) -> f64 {
            self.feedrate
        }

        pub fn get_length(&self) -> f64 {
            self.length
        }

        pub fn get_zlift(&self) -> f64 {
            self.z_lift
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct EndstopPartConfig{
        pin: String,
        exti: String
    }

    impl EndstopPartConfig{
        pub fn get_pin(&self) -> Option<String>{
            get_string_value(self.pin.clone())
        }

        pub fn get_exti(&self) -> Option<String>{
            get_string_value(self.exti.clone())
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct EndstopsConfig{
        x: EndstopPartConfig,
        y: EndstopPartConfig,
        z: EndstopPartConfig,
    }

    impl EndstopsConfig{
        pub fn get_x(&self) -> EndstopPartConfig{
            self.x.clone()
        }
        
        pub fn get_y(&self) -> EndstopPartConfig{
            self.y.clone()
        }

        pub fn get_z(&self) -> EndstopPartConfig{
            self.z.clone()
        }
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone)]
    pub struct MotionConfig{
        arc_unit_length: f64,
        feedrate: f64,
        positioning: String,
        retraction: RetractionMotionConfig,
        recover: RecoverMotionConfig,
        endstops: EndstopsConfig
    }

    impl MotionConfig{
        pub fn get_arc_unit_length(&self) -> f64 {
            self.arc_unit_length
        }

        pub fn get_feedrate(&self) -> f64 {
            self.feedrate
        }

        pub fn get_positioning(&self) -> Option<String> {
            get_string_value(self.positioning.clone())
        }

        pub fn get_retraction(&self) -> RetractionMotionConfig{
            self.retraction
        }

        pub fn get_recover(&self) -> RecoverMotionConfig{
            self.recover
        }

        pub fn get_endstops(&self) -> EndstopsConfig{
            self.endstops.clone()
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
        pub motion: MotionConfig
    }
}

fn main() {
    println!("cargo::rerun-if-changed=config/config.toml");
    let path = Path::new("config/config.toml");
    let conf = confy::load_path::<external::MyConfig>(path).expect("Error reading config file");
    
    let motion_arc_unit_len = conf.motion.get_arc_unit_length();
    let motion_feedrate = conf.motion.get_feedrate();
    let motion_positioning = conf.motion.get_positioning().expect("Motion positioning is missing");
    let motion_positioning = motion_positioning.as_str();
    let _ = Positioning::from(motion_positioning);

    let motion_retraction_z_lift = conf.motion.get_retraction().get_zlift();
    let motion_retraction_feedrate = conf.motion.get_retraction().get_feedrate();
    let motion_retraction_len = conf.motion.get_retraction().get_length();
    let motion_recover_feedrate = conf.motion.get_recover().get_feedrate();
    let motion_recover_len = conf.motion.get_recover().get_length();

    let motion_endstop_x = conf.motion.get_endstops().get_x().get_pin().expect("Endstop x axis is missing");
    let motion_endstop_x = Ident::new(motion_endstop_x.as_str(), Span::call_site());

    let motion_endstop_x_exti = conf.motion.get_endstops().get_x().get_exti().expect("Endstop x EXTI is missing");
    let motion_endstop_x_exti = Ident::new(motion_endstop_x_exti.as_str(), Span::call_site());

    let motion_endstop_y = conf.motion.get_endstops().get_y().get_pin().expect("Endstop y axis is missing");
    let motion_endstop_y = Ident::new(motion_endstop_y.as_str(), Span::call_site());

    let motion_endstop_y_exti = conf.motion.get_endstops().get_y().get_exti().expect("Endstop y EXTI is missing");
    let motion_endstop_y_exti = Ident::new(motion_endstop_y_exti.as_str(), Span::call_site());

    let motion_endstop_z = conf.motion.get_endstops().get_z().get_pin().expect("Endstop z axis is missing");
    let motion_endstop_z = Ident::new(motion_endstop_z.as_str(), Span::call_site());

    let motion_endstop_z_exti = conf.motion.get_endstops().get_z().get_exti().expect("Endstop z EXTI is missing");
    let motion_endstop_z_exti = Ident::new(motion_endstop_z_exti.as_str(), Span::call_site());

    let steppers_x_step_pin = conf
        .steppers
        .get_x()
        .get_step()
        .get_pin()
        .expect("Stepper X step pin is missing");
    let steppers_x_step_pin = Ident::new(steppers_x_step_pin.as_str(), Span::call_site());

    let steppers_x_dir_pin = conf
        .steppers
        .get_x()
        .get_dir()
        .get_pin()
        .expect("Stepper X dir pin is missing");
    let steppers_x_dir_pin = Ident::new(steppers_x_dir_pin.as_str(), Span::call_site());
    let steppers_x_stepping_mode = conf.steppers.get_x().get_stepping_mode();
    let steppers_x_stepping_mode = steppers_x_stepping_mode.as_str();
    let _ = SteppingMode::from(steppers_x_stepping_mode);
    
    let steppers_x_distance_per_step = conf.steppers.get_x().get_distance_per_step();
    let steppers_x_steps_per_revolution = conf.steppers.get_x().get_steps_per_revolution();
    let steppers_x_bounds = conf.steppers.get_x().get_bounds();
    let steppers_x_bounds_min = steppers_x_bounds.min;
    let steppers_x_bounds_max = steppers_x_bounds.max;
    let steppers_x_positive_direction = conf.steppers.get_x().get_positive_direction();
    let steppers_x_positive_direction = steppers_x_positive_direction.as_str();
    let _ = RotationDirection::from(steppers_x_positive_direction);

    let steppers_y_step_pin = conf
        .steppers
        .get_y()
        .get_step()
        .get_pin()
        .expect("Stepper Y step pin is missing");
    let steppers_y_step_pin = Ident::new(steppers_y_step_pin.as_str(), Span::call_site());

    let steppers_y_dir_pin = conf
        .steppers
        .get_y()
        .get_dir()
        .get_pin()
        .expect("Stepper Y dir pin is missing");
    let steppers_y_dir_pin = Ident::new(steppers_y_dir_pin.as_str(), Span::call_site());

    let steppers_y_stepping_mode = conf.steppers.get_y().get_stepping_mode();
    let steppers_y_stepping_mode = steppers_y_stepping_mode.as_str();
    let _ = SteppingMode::from(steppers_y_stepping_mode);
    
    let steppers_y_distance_per_step = conf.steppers.get_y().get_distance_per_step();
    let steppers_y_steps_per_revolution = conf.steppers.get_y().get_steps_per_revolution();
    let steppers_y_bounds = conf.steppers.get_y().get_bounds();
    let steppers_y_bounds_min = steppers_y_bounds.min;
    let steppers_y_bounds_max = steppers_y_bounds.max;
    let steppers_y_positive_direction = conf.steppers.get_y().get_positive_direction();
    let steppers_y_positive_direction = steppers_y_positive_direction.as_str();
    let _ = RotationDirection::from(steppers_y_positive_direction);

    let steppers_z_step_pin = conf
        .steppers
        .get_z()
        .get_step()
        .get_pin()
        .expect("Stepper Z step pin is missing");
    let steppers_z_step_pin = Ident::new(steppers_z_step_pin.as_str(), Span::call_site());

    let steppers_z_dir_pin = conf
        .steppers
        .get_z()
        .get_dir()
        .get_pin()
        .expect("Stepper Z dir pin is missing");
    let steppers_z_dir_pin = Ident::new(steppers_z_dir_pin.as_str(), Span::call_site());
    let steppers_z_stepping_mode = conf.steppers.get_z().get_stepping_mode();
    let steppers_z_stepping_mode = steppers_z_stepping_mode.as_str();
    let _ = SteppingMode::from(steppers_z_stepping_mode);
    let steppers_z_distance_per_step = conf.steppers.get_z().get_distance_per_step();
    let steppers_z_steps_per_revolution = conf.steppers.get_z().get_steps_per_revolution();
    let steppers_z_bounds = conf.steppers.get_z().get_bounds();
    let steppers_z_bounds_min = steppers_z_bounds.min;
    let steppers_z_bounds_max = steppers_z_bounds.max;
    let steppers_z_positive_direction = conf.steppers.get_z().get_positive_direction();
    let steppers_z_positive_direction = steppers_z_positive_direction.as_str();
    let _ = RotationDirection::from(steppers_z_positive_direction);

    let steppers_e_step_pin = conf
        .steppers
        .get_e()
        .get_step()
        .get_pin()
        .expect("Stepper E step pin is missing");
    let steppers_e_step_pin = Ident::new(steppers_e_step_pin.as_str(), Span::call_site());

    let steppers_e_dir_pin = conf
        .steppers
        .get_e()
        .get_dir()
        .get_pin()
        .expect("Stepper E dir pin is missing");
    let steppers_e_dir_pin = Ident::new(steppers_e_dir_pin.as_str(), Span::call_site());

    let steppers_e_stepping_mode = conf.steppers.get_e().get_stepping_mode();
    let steppers_e_stepping_mode = steppers_e_stepping_mode.as_str();
    let _ = SteppingMode::from(steppers_e_stepping_mode);
    let steppers_e_distance_per_step = conf.steppers.get_e().get_distance_per_step();
    let steppers_e_steps_per_revolution = conf.steppers.get_e().get_steps_per_revolution();
    let steppers_e_bounds = conf.steppers.get_e().get_bounds();
    let steppers_e_bounds_min = steppers_e_bounds.min;
    let steppers_e_bounds_max = steppers_e_bounds.max;
    let steppers_e_positive_direction = conf.steppers.get_e().get_positive_direction();
    let steppers_e_positive_direction = steppers_e_positive_direction.as_str();
    let _ = RotationDirection::from(steppers_e_positive_direction);
    
    let pwm_timer = conf
        .pwm
        .get_timer()
        .expect("PWM timer peripheral is missing");
    let pwm_timer = Ident::new(pwm_timer.as_str(), Span::call_site());
    
    let pwm_frequency = conf.pwm.get_frequency();
    let pwm_ch1 = conf.pwm.get_ch1().expect("PMW ch1 is missing");
    let pwm_ch1 = Ident::new(pwm_ch1.as_str(), Span::call_site());
    let pwm_ch2 = conf.pwm.get_ch2().expect("PMW ch2 is missing");
    let pwm_ch2 = Ident::new(pwm_ch2.as_str(), Span::call_site());
    let pwm_ch3 = conf.pwm.get_ch3().expect("PMW ch3 is missing");
    let pwm_ch3 = Ident::new(pwm_ch3.as_str(), Span::call_site());

    let uart_peripheral = conf
        .uart
        .get_peripheral()
        .expect("UART peripheral is missing");
    let uart_peripheral = Ident::new(uart_peripheral.as_str(), Span::call_site());

    let uart_baudrate = conf.uart.get_baudrate();
    let uart_rx_pin = conf
        .uart
        .get_rx()
        .get_pin()
        .expect("UART RX pin is missing");
    let uart_rx_pin = Ident::new(uart_rx_pin.as_str(), Span::call_site());

    let uart_rx_dma = conf
        .uart
        .get_rx()
        .get_dma()
        .get_peripheral()
        .expect("UART RX pin is missing");
    let uart_rx_dma = Ident::new(uart_rx_dma.as_str(), Span::call_site());

    let uart_tx_pin = conf
        .uart
        .get_tx()
        .get_pin()
        .expect("UART TX pin is missing");
    let uart_tx_pin = Ident::new(uart_tx_pin.as_str(), Span::call_site());

    let uart_tx_dma = conf
        .uart
        .get_tx()
        .get_dma()
        .get_peripheral()
        .expect("UART TX pin is missing");
    let uart_tx_dma = Ident::new(uart_tx_dma.as_str(), Span::call_site());

    let hotend_adc_peripheral = conf
        .hotend
        .get_adc()
        .get_peripheral()
        .expect("Hotend ADC peripheral is missing");
    let hotend_adc_peripheral = Ident::new(hotend_adc_peripheral.as_str(), Span::call_site());

    let hotend_adc_input_pin = conf
        .hotend
        .get_adc()
        .get_input()
        .get_pin()
        .expect("Hotend ADC input pin is missing");
    let hotend_adc_input_pin = Ident::new(hotend_adc_input_pin.as_str(), Span::call_site());

    let hotend_adc_dma = conf
        .hotend
        .get_adc()
        .get_dma()
        .get_peripheral()
        .expect("Hotend ADC DMA peripheral is missing");
    let hotend_adc_dma = Ident::new(hotend_adc_dma.as_str(), Span::call_site());

    let hotend_pwm_output_channel = conf.hotend.get_pwm().get_channel();
    let hotend_heater_r0 = conf.hotend.get_heater().get_r0();
    let hotend_heater_r_series = conf.hotend.get_heater().get_r_series();
    let hotend_heater_b = conf.hotend.get_heater().get_b();
    let hotend_heater_pid = conf.hotend.get_heater().get_pid();
    let hotend_heater_pid_kp = hotend_heater_pid.get_k_p();
    let hotend_heater_pid_ki = hotend_heater_pid.get_k_i();
    let hotend_heater_pid_kd = hotend_heater_pid.get_k_d();

    let heatbed_adc_peripheral = conf
        .heatbed
        .get_adc()
        .get_peripheral()
        .expect("Heatbed ADC peripheral is missing");
    let heatbed_adc_peripheral = Ident::new(heatbed_adc_peripheral.as_str(), Span::call_site());

    let heatbed_adc_input_pin = conf
        .heatbed
        .get_adc()
        .get_input()
        .get_pin()
        .expect("Heatbed ADC input pin is missing");
    let heatbed_adc_input_pin = Ident::new(heatbed_adc_input_pin.as_str(), Span::call_site());

    let heatbed_adc_dma = conf
        .heatbed
        .get_adc()
        .get_dma()
        .get_peripheral()
        .expect("Heatbed ADC DMA peripheral is missing");
    let heatbed_adc_dma = Ident::new(heatbed_adc_dma.as_str(), Span::call_site());

    let heatbed_pwm_output_channel = conf.heatbed.get_pwm().get_channel();
    let heatbed_heater_r0 = conf.heatbed.get_heater().get_r0();
    let heatbed_heater_r_series = conf.heatbed.get_heater().get_r0();
    let heatbed_heater_b = conf.heatbed.get_heater().get_r_series();
    let heatbed_heater_pid = conf.heatbed.get_heater().get_pid();
    let heatbed_heater_pid_kp = heatbed_heater_pid.get_k_p();
    let heatbed_heater_pid_ki = heatbed_heater_pid.get_k_i();
    let heatbed_heater_pid_kd = heatbed_heater_pid.get_k_d();

    let fan_pwm_output_channel = conf.fan.get_pwm().get_channel();

    let sdcard_spi_peripheral = conf
        .sdcard
        .get_spi()
        .get_peripheral()
        .expect("SD-Card peripheral is missing");
    let sdcard_spi_peripheral = Ident::new(sdcard_spi_peripheral.as_str(), Span::call_site());

    let sdcard_spi_timer = conf
        .sdcard
        .get_spi()
        .get_clk()
        .get_peripheral()
        .expect("SD-Card SPI timer is missing");
    let sdcard_spi_timer = Ident::new(sdcard_spi_timer.as_str(), Span::call_site());

    let sdcard_spi_mosi = conf
        .sdcard
        .get_spi()
        .get_mosi()
        .get_pin()
        .expect("SD-Card SPI MOSI pin is missing");
    let sdcard_spi_mosi = Ident::new(sdcard_spi_mosi.as_str(), Span::call_site());

    let sdcard_spi_miso = conf
        .sdcard
        .get_spi()
        .get_miso()
        .get_pin()
        .expect("SD-Card SPI MISO pin is missing");
    let sdcard_spi_miso = Ident::new(sdcard_spi_miso.as_str(), Span::call_site());

    let sdcard_spi_cs = conf
        .sdcard
        .get_spi()
        .get_cs()
        .get_pin()
        .expect("SD-Card SPI CS pin is missing");
    let sdcard_spi_cs = Ident::new(sdcard_spi_cs.as_str(), Span::call_site());

    if hotend_pwm_output_channel < 1 || hotend_pwm_output_channel > 4{
        panic!("Hotend PWM channel must be between 1 and 4");
    }
    if heatbed_pwm_output_channel < 1 || heatbed_pwm_output_channel > 4{
        panic!("Heatbed PWM channel must be between 1 and 4");
    }
    if fan_pwm_output_channel < 1 || fan_pwm_output_channel > 4{
        panic!("Fan PWM channel must be between 1 and 4");
    }

    let tokens = quote! {
        use embassy_stm32::Peripherals;
        use embassy_stm32::peripherals::*;
        use math::measurements::{Speed, Length, Distance, Resistance, Temperature};
        use math::common::RotationDirection;
        use stepper::motion::Positioning;
        use stepper::stepper::SteppingMode;
        use stepper::planner::{MotionConfig, RecoverMotionConfig, RetractionMotionConfig};
        use crate::config::*;

        pub type XStepPin = #steppers_x_step_pin;
        pub type XDirPin = #steppers_x_dir_pin;
        pub type YStepPin = #steppers_y_step_pin;
        pub type YDirPin = #steppers_y_dir_pin;
        pub type ZStepPin = #steppers_z_step_pin;
        pub type ZDirPin = #steppers_z_dir_pin;
        pub type EStepPin = #steppers_e_step_pin;
        pub type EDirPin = #steppers_e_dir_pin;
        pub type PwmTimer = #pwm_timer;
        pub type PwmCh1Pin = #pwm_ch1;
        pub type PwmCh2Pin = #pwm_ch2;
        pub type PwmCh3Pin = #pwm_ch3;
        pub type UartPeripheral = #uart_peripheral;
        pub type UartRxPin = #uart_rx_pin;
        pub type UartRxDma = #uart_rx_dma;
        pub type UartTxPin = #uart_tx_pin;
        pub type UartTxDma = #uart_tx_dma;
        pub type HotendAdcPeripheral = #hotend_adc_peripheral;
        pub type HotendAdcInputPin = #hotend_adc_input_pin;
        pub type HotendAdcDma = #hotend_adc_dma;
        pub type HeatbedAdcPeripheral = #heatbed_adc_peripheral;
        pub type HeatbedAdcInputPin = #heatbed_adc_input_pin;
        pub type HeatbedAdcDma = #heatbed_adc_dma;
        pub type SdCardSpiPeripheral = #sdcard_spi_peripheral;
        pub type SdCardSpiTimer = #sdcard_spi_timer;
        pub type SdCardSpiMosiPin = #sdcard_spi_mosi;
        pub type SdCardSpiMisoPin = #sdcard_spi_miso;
        pub type SdCardSpiCsPin = #sdcard_spi_cs;
        pub type XEndstopPin = #motion_endstop_x;
        pub type XEndstopExti = #motion_endstop_x_exti;
        pub type YEndstopPin = #motion_endstop_y;
        pub type YEndstopExti = #motion_endstop_y_exti;
        pub type ZEndstopPin = #motion_endstop_z;
        pub type ZEndstopExti = #motion_endstop_z_exti;

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
            XEndstopPin,
            XEndstopExti,
            YEndstopPin,
            YEndstopExti,
            ZEndstopPin,
            ZEndstopExti,
        >{
            PrinterConfig{
                motion: MotionConfig{
                    arc_unit_length: Length::from_millimeters(#motion_arc_unit_len),
                    feedrate: Speed::from_meters_per_second(#motion_feedrate / 1000.0),
                    positioning: Positioning::from(#motion_positioning),
                    retraction: RetractionMotionConfig{
                        feedrate: Speed::from_meters_per_second(#motion_retraction_feedrate),
                        length: Length::from_meters(#motion_retraction_len * 1000.0),
                        z_lift: Length::from_meters(#motion_retraction_z_lift * 1000.0),
                    },
                    recover: RecoverMotionConfig{
                        feedrate: Speed::from_meters_per_second(#motion_recover_feedrate),
                        length: Length::from_meters(#motion_recover_len * 1000.0),
                    },
                },
                endstops: EndstopsConfig{
                    x: EndstopPartConfig {
                        pin: p.#motion_endstop_x,
                        exti: p.#motion_endstop_x_exti,
                    },
                    y: EndstopPartConfig {
                        pin: p.#motion_endstop_y,
                        exti: p.#motion_endstop_y_exti,
                    },
                    z: EndstopPartConfig {
                        pin: p.#motion_endstop_z,
                        exti: p.#motion_endstop_z_exti,
                    },
                },
                steppers: SteppersConfig{
                    x: StepperConfig{
                        step_pin: p.#steppers_x_step_pin,
                        dir_pin: p.#steppers_x_dir_pin,
                        stepping_mode: SteppingMode::from(#steppers_x_stepping_mode),
                        distance_per_step: Distance::from_millimeters(#steppers_x_distance_per_step),
                        steps_per_revolution: #steppers_x_steps_per_revolution,
                        bounds: (#steppers_x_bounds_min, #steppers_x_bounds_max),
                        positive_direction: RotationDirection::from(#steppers_x_positive_direction),
                    },
                    y: StepperConfig{
                        step_pin: p.#steppers_y_step_pin,
                        dir_pin: p.#steppers_y_dir_pin,
                        stepping_mode: SteppingMode::from(#steppers_y_stepping_mode),
                        distance_per_step: Distance::from_millimeters(#steppers_y_distance_per_step),
                        steps_per_revolution: #steppers_y_steps_per_revolution,
                        bounds: (#steppers_y_bounds_min, #steppers_y_bounds_max),
                        positive_direction: RotationDirection::from(#steppers_y_positive_direction),
                    },
                    z: StepperConfig{
                        step_pin: p.#steppers_z_step_pin,
                        dir_pin: p.#steppers_z_dir_pin,
                        stepping_mode: SteppingMode::from(#steppers_z_stepping_mode),
                        distance_per_step: Distance::from_millimeters(#steppers_z_distance_per_step),
                        steps_per_revolution: #steppers_z_steps_per_revolution,
                        bounds: (#steppers_z_bounds_min, #steppers_z_bounds_max),
                        positive_direction: RotationDirection::from(#steppers_z_positive_direction),
                    },
                    e: StepperConfig{
                        step_pin: p.#steppers_e_step_pin,
                        dir_pin: p.#steppers_e_dir_pin,
                        stepping_mode: SteppingMode::from(#steppers_e_stepping_mode),
                        distance_per_step: Distance::from_millimeters(#steppers_e_distance_per_step),
                        steps_per_revolution: #steppers_e_steps_per_revolution,
                        bounds: (#steppers_e_bounds_min, #steppers_e_bounds_max),
                        positive_direction: RotationDirection::from(#steppers_e_positive_direction),
                    },
                },
                pwm: PwmConfig{
                    frequency: #pwm_frequency,
                    timer: p.#pwm_timer,
                    ch1: p.#pwm_ch1,
                    ch2: p.#pwm_ch2,
                    ch3: p.#pwm_ch3,
                },
                uart: UartConfig{
                    peripheral: p.#uart_peripheral,
                    baudrate: #uart_baudrate,
                    rx: UartPartConfig{
                        pin: p.#uart_rx_pin,
                        dma: p.#uart_rx_dma,
                    },
                    tx: UartPartConfig{
                        pin: p.#uart_tx_pin,
                        dma: p.#uart_tx_dma,
                    }
                },
                hotend: ThermistorConfig{
                    adc: AdcConfig {
                        peripheral: p.#hotend_adc_peripheral,
                        input: p.#hotend_adc_input_pin,
                        dma: p.#hotend_adc_dma,
                    },
                    pwm: PwmOutputConfig {
                        channel: #hotend_pwm_output_channel,
                    },
                    heater: HeaterConfig {
                        r_series: Resistance::from_ohms(#hotend_heater_r_series),
                        r0: Resistance::from_ohms(#hotend_heater_r0),
                        b: Temperature::from_celsius(#hotend_heater_b),
                        pid: PidConfig{
                            k_p: #hotend_heater_pid_kp,
                            k_i: #hotend_heater_pid_ki,
                            k_d: #hotend_heater_pid_kd,
                        }
                    },
                },
                heatbed: ThermistorConfig{
                    adc: AdcConfig {
                        peripheral: p.#heatbed_adc_peripheral,
                        input: p.#heatbed_adc_input_pin,
                        dma: p.#heatbed_adc_dma,
                    },
                    pwm: PwmOutputConfig {
                        channel: #heatbed_pwm_output_channel,
                    },
                    heater: HeaterConfig {
                        r_series: Resistance::from_ohms(#heatbed_heater_r_series),
                        r0: Resistance::from_ohms(#heatbed_heater_r0),
                        b: Temperature::from_celsius(#heatbed_heater_b),
                        pid: PidConfig{
                            k_p: #heatbed_heater_pid_kp,
                            k_i: #heatbed_heater_pid_ki,
                            k_d: #heatbed_heater_pid_kd,
                        }
                    },
                },
                fan: FanConfig{
                    pwm: PwmOutputConfig {
                        channel: #fan_pwm_output_channel,
                    }
                },
                sdcard: SdCardConfig {
                    spi: SpiConfig {
                        peripheral: p.#sdcard_spi_peripheral,
                        clk: p.#sdcard_spi_timer,
                        mosi: p.#sdcard_spi_mosi,
                        miso: p.#sdcard_spi_miso,
                        cs: p.#sdcard_spi_cs,
                    }
                }
            }
        }
    };

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("_abcd.rs").to_string_lossy().to_string();
    fs::write(&out_file, tokens.to_string().as_str()).unwrap();
}
