pub struct StepperConfig<S, D>{
    pub step_pin: S,
    pub dir_pin: D,
}

pub struct UartPartConfig<P, D>{
    pub pin: P,
    pub dma: D,
}

pub struct UartConfig<P, RXP, RXD, TXP, TXD>{
    pub peripheral: P,
    pub baudrate: u64,
    pub rx: UartPartConfig<RXP, RXD>,
    pub tx: UartPartConfig<TXP, TXD>
}

pub struct SteppersConfig<XP, XD, YP, YD, ZP, ZD, EP, ED>{
    pub x: StepperConfig<XP, XD>,
    pub y: StepperConfig<YP, YD>,
    pub z: StepperConfig<ZP, ZD>,
    pub e: StepperConfig<EP, ED>,
}

pub struct PrinterConfig<A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z,AA,AB,AC,AD,AE,AF,AG,AH,AI,AJ,AK,AL,AM>{
    pub steppers: SteppersConfig<A,B,C,D,E,F,G,H>,
    pub uart: UartConfig<I,J,K,L,M>,
    pub hotend: ThermistorConfig<N,O,P,Q,R,S,T,U>,
    pub heatbed: ThermistorConfig<V,W,X,Y,Z,AA,AB,AC>,
    pub fan: FanConfig<AD,AE,AF,AG,AH>,
    pub sdcard: SdCardConfig<AI,AJ,AK,AL,AM>
}

pub struct AdcConfig<P,I,D>{
    pub peripheral: P,
    pub input: I,
    pub dma: D,
}

pub struct PwmConfig<T,A,B,C,D>{
    pub frequency: u64,
    pub timer: T,
    pub channel0: Option<A>,
    pub channel1: Option<B>,
    pub channel2: Option<C>,
    pub channel3: Option<D>,
}

pub struct SpiConfig<P,T,MO,MI,CS>{
    pub peripheral: P,
    pub timer: T,
    pub mosi: MO,
    pub miso: MI,
    pub cs: CS,
}
pub struct PidConfig{
    pub k_p: f64,
    pub k_i: f64,
    pub k_d: f64,
}

pub struct HeaterConfig{
    pub r_series: u64,
    pub r0: u64,
    pub b: u64,
    pub pid: PidConfig
}

pub struct ThermistorConfig<ADCP,ADCI,ADCD,PWMT, PWMA, PWMB, PWMC, PWMD>{
    pub heater: HeaterConfig,
    pub adc: AdcConfig<ADCP,ADCI,ADCD>,
    pub pwm: PwmConfig<PWMT, PWMA, PWMB, PWMC, PWMD>
}

pub struct FanConfig<PWMT, PWMA, PWMB, PWMC, PWMD>{
    pub pwm: PwmConfig<PWMT, PWMA, PWMB, PWMC, PWMD>
}

pub struct SdCardConfig<SPIP,SPIT,SPIMO,SPIMI,SPICS>{
    pub spi: SpiConfig<SPIP,SPIT,SPIMO,SPIMI,SPICS>
}
