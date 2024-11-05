pub struct StepperConfig<S, D>{
    pub step_pin: S,
    pub dir_pin: D,
}

pub struct UartPartConfig<P, D>{
    pin: P,
    dma: D,
}

pub struct UartConfig<P, RXP, RXD, TXP, TXD>{
    peripheral: P,
    baudrate: u64,
    rx: UartPartConfig<RXP, RXD>,
    tx: UartPartConfig<TXP, TXD>
}

pub struct PrinterConfig<XP, XD, YP, YD, ZP, ZD, EP, ED>{
    pub x_stepper: StepperConfig<XP, XD>,
    pub y_stepper: StepperConfig<YP, YD>,
    pub z_stepper: StepperConfig<ZP, ZD>,
    pub e_stepper: StepperConfig<EP, ED>,
}
