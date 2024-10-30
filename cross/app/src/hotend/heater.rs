use embassy_stm32::{
    time::Hertz,
    timer::{simple_pwm::SimplePwm, Channel, GeneralInstance4Channel},
};
use embassy_time::Duration;
use math::measurements::Temperature;
use micromath::F32Ext;
use pid_lite::Controller;

pub struct Heater<'s, T: GeneralInstance4Channel> {
    out: SimplePwm<'s, T>,
    ch: Channel,
    pid: Controller,
    target_temperature: Option<Temperature>,
}

impl<'s, T: GeneralInstance4Channel> Heater<'s, T> {
    pub fn new(out: SimplePwm<'s, T>, ch: Channel) -> Heater<'s, T> {
        let pid = Controller::new(
            Temperature::from_celsius(30.0).as_celsius(),
            20.0,
            0.02,
            0.0,
        );
        let mut out = out;
        out.set_frequency(Hertz::hz(100));
        Heater {
            out,
            ch,
            pid,
            target_temperature: None,
        }
    }

    pub fn enable(&mut self){
        self.out.enable(self.ch);
    }

    pub fn disable(&mut self){
        self.out.disable(self.ch);
    }

    pub fn reset_target_temperature(&mut self) {
        self.target_temperature = None;
    }

    pub fn set_target_temperature(&mut self, temperature: Temperature) {
        self.target_temperature = Some(temperature);
        self.pid
            .set_target(self.target_temperature.unwrap().as_celsius());
    }

    pub fn update(&mut self, tmp: Temperature, dt: Duration) -> Result<u32, ()> {
        if self.target_temperature.is_none() {
            return Err(());
        }
        
        let duty_cycle = self.pid.update_elapsed(
            tmp.as_celsius(),
            core::time::Duration::from_millis(dt.as_millis()),
        );

        // info!("duty cycle real value {}", duty_cycle);
        let duty_cycle = duty_cycle.max(0f64).min(f64::from(self.out.get_max_duty()));

        let duty_cycle = (duty_cycle as f32).trunc() as u32;

        // info!("duty cycle set to {}", duty_cycle);
        self.out.set_duty(self.ch, duty_cycle);

        Ok(duty_cycle)
    }
}
