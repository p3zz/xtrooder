use math::{
    common::RotationDirection,
    measurements::{AngularVelocity, Length, Resistance, Temperature},
};
pub use stepper::planner::MotionConfig;
use stepper::stepper::SteppingMode;

pub struct EndstopPartConfig<P, E> {
    pub pin: P,
    pub exti: E,
}

pub struct EndstopsConfig<XP, XE, YP, YE, ZP, ZE> {
    pub x: EndstopPartConfig<XP, XE>,
    pub y: EndstopPartConfig<YP, YE>,
    pub z: EndstopPartConfig<ZP, ZE>,
}

pub struct StepperConfig<S, D> {
    pub step_pin: S,
    pub dir_pin: D,
    pub stepping_mode: SteppingMode,
    pub distance_per_step: Length,
    pub steps_per_revolution: u64,
    pub bounds: (f64, f64),
    pub positive_direction: RotationDirection,
}

pub struct UartPartConfig<P, D> {
    pub pin: P,
    pub dma: D,
}

pub struct UartConfig<P, RXP, RXD, TXP, TXD> {
    pub peripheral: P,
    pub baudrate: u64,
    pub rx: UartPartConfig<RXP, RXD>,
    pub tx: UartPartConfig<TXP, TXD>,
}

pub struct SteppersConfig<XP, XD, YP, YD, ZP, ZD, EP, ED> {
    pub x: StepperConfig<XP, XD>,
    pub y: StepperConfig<YP, YD>,
    pub z: StepperConfig<ZP, ZD>,
    pub e: StepperConfig<EP, ED>,
}

pub struct PrinterConfig<
    XP,
    XD,
    YP,
    YD,
    ZP,
    ZD,
    EP,
    ED,
    PWMT,
    CH1,
    CH2,
    CH3,
    UP,
    RXP,
    RXD,
    TXP,
    TXD,
    HOP,
    HOI,
    HOD,
    HEP,
    HEI,
    HED,
    SPIP,
    SPIT,
    SPIMO,
    SPIMI,
    SPICS,
    XEP,
    XEE,
    YEP,
    YEE,
    ZEP,
    ZEE,
> {
    pub steppers: SteppersConfig<XP, XD, YP, YD, ZP, ZD, EP, ED>,
    pub pwm: PwmConfig<PWMT, CH1, CH2, CH3>,
    pub uart: UartConfig<UP, RXP, RXD, TXP, TXD>,
    pub hotend: ThermistorConfig<HOP, HOI, HOD>,
    pub heatbed: ThermistorConfig<HEP, HEI, HED>,
    pub fan: FanConfig,
    pub sdcard: SdCardConfig<SPIP, SPIT, SPIMO, SPIMI, SPICS>,
    pub motion: MotionConfig,
    pub endstops: EndstopsConfig<XEP, XEE, YEP, YEE, ZEP, ZEE>,
}

pub struct AdcConfig<P, I, D> {
    pub peripheral: P,
    pub input: I,
    pub dma: D,
}

pub struct PwmConfig<T, CH1, CH2, CH3> {
    pub frequency: u64,
    pub timer: T,
    pub ch1: CH1,
    pub ch2: CH2,
    pub ch3: CH3,
}

pub struct PwmOutputConfig {
    pub channel: u8,
}

pub struct SpiConfig<P, C, MO, MI, CS> {
    pub peripheral: P,
    pub clk: C,
    pub mosi: MO,
    pub miso: MI,
    pub cs: CS,
}
pub struct PidConfig {
    pub k_p: f64,
    pub k_i: f64,
    pub k_d: f64,
}

pub struct HeaterConfig {
    pub r_series: Resistance,
    pub r0: Resistance,
    pub b: Temperature,
    pub pid: PidConfig,
    pub max_temperature_limit: Temperature,
    pub min_temperature_limit: Temperature,
}

pub struct ThermistorConfig<ADCP, ADCI, ADCD> {
    pub heater: HeaterConfig,
    pub adc: AdcConfig<ADCP, ADCI, ADCD>,
    pub pwm: PwmOutputConfig,
}

pub struct FanConfig {
    pub max_speed: AngularVelocity,
    pub pwm_output_channel: PwmOutputConfig,
}

pub struct SdCardConfig<SPIP, SPIT, SPIMO, SPIMI, SPICS> {
    pub spi: SpiConfig<SPIP, SPIT, SPIMO, SPIMI, SPICS>,
}
