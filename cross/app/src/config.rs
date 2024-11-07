pub struct StepperConfig<S, D> {
    pub step_pin: S,
    pub dir_pin: D,
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
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    V,
    W,
    X,
    Y,
    Z,
    AA,
    AB,
    AC,
    AD,
    AE,
> {
    pub steppers: SteppersConfig<A, B, C, D, E, F, G, H>,
    pub pwm: PwmConfig<I>,
    pub uart: UartConfig<J, K, L, M, N>,
    pub hotend: ThermistorConfig<O, P, Q, R>,
    pub heatbed: ThermistorConfig<V, W, X, Y>,
    pub fan: FanConfig<Z>,
    pub sdcard: SdCardConfig<AA, AB, AC, AD, AE>,
}

pub struct AdcConfig<P, I, D> {
    pub peripheral: P,
    pub input: I,
    pub dma: D,
}

pub struct PwmConfig<T> {
    pub frequency: u64,
    pub timer: T,
}

pub struct PwmOutputConfig<O> {
    pub output: O,
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
    pub r_series: u64,
    pub r0: u64,
    pub b: u64,
    pub pid: PidConfig,
}

pub struct ThermistorConfig<ADCP, ADCI, ADCD, PWMO> {
    pub heater: HeaterConfig,
    pub adc: AdcConfig<ADCP, ADCI, ADCD>,
    pub pwm: PwmOutputConfig<PWMO>,
}

pub struct FanConfig<PWMO> {
    pub pwm: PwmOutputConfig<PWMO>,
}

pub struct SdCardConfig<SPIP, SPIT, SPIMO, SPIMI, SPICS> {
    pub spi: SpiConfig<SPIP, SPIT, SPIMO, SPIMI, SPICS>,
}
