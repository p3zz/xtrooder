use defmt::info;
use embassy_stm32::{time::Hertz, timer::{simple_pwm::SimplePwm, CaptureCompare16bitInstance, Channel}};
use embassy_time::Duration;
use math::temperature::Temperature;
use micromath::F32Ext;
use pid_lite::Controller;

pub struct Heater<'s, S> {
    out: SimplePwm<'s, S>,
    ch: Channel,
    pid: Controller,
}

impl<'s, S> Heater<'s, S>
where
    S: CaptureCompare16bitInstance,
{
    pub fn new(mut out: SimplePwm<'s, S>, ch: Channel) -> Heater<'s, S> {
        let pid = Controller::new(Temperature::from_celsius(25.0).to_celsius(), 1.0, 0.01, 0.0);
        out.set_frequency(Hertz::khz(1));
        out.set_duty(ch, 0);
        out.enable(ch);
        Heater { out, ch, pid }
    }

    pub fn set_target_temperature(&mut self, temperature: Temperature) {
        self.pid.set_target(temperature.to_celsius());
    }

    pub fn update(&mut self, tmp: Temperature, dt: Duration) {
        let mut duty_cycle = self.pid.update_elapsed(
            tmp.to_celsius(),
            core::time::Duration::from_millis(dt.as_millis()),
        );

        info!("duty cycle real value {}", duty_cycle);

        let min = 0f64;
        let max = f64::from(self.out.get_max_duty());

        if duty_cycle > max {
            duty_cycle = max;
        }
        if duty_cycle < min {
            duty_cycle = min;
        }

        let duty_cycle = (duty_cycle as f32).trunc() as u16;

        info!("duty cycle set to {}", duty_cycle);
        self.out.set_duty(self.ch, duty_cycle);
    }
}
