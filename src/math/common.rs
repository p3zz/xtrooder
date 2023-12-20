use micromath::F32Ext;
use embassy_time::{driver::now, Duration};

pub fn abs(value: f64) -> f64 {
    (value as f32).abs() as f64
}

pub fn sqrt(value: f64) -> f64 {
    (value as f32).sqrt() as f64
}

pub struct StopWatch{
    last_ticks: u64
}

impl StopWatch{
    pub fn new() -> StopWatch {
        StopWatch {last_ticks: 0}
    }

    pub fn start(&mut self){
        self.last_ticks = now();
    }

    pub fn measure(&self) -> Duration {
        let current_ticks = now();
        Duration::from_ticks(current_ticks - self.last_ticks)
    }
}