use embassy_time::{Duration, Instant};
use micromath::F32Ext;

pub fn abs(value: f64) -> f64 {
    let mut v = value;
    if value.is_sign_negative() {
        v = -value;
    }
    v
}

pub fn sqrt(value: f64) -> f64 {
    (value as f32).sqrt() as f64
}

pub struct StopWatch {
    last_ticks: u64,
}

impl StopWatch {
    pub fn new() -> StopWatch {
        StopWatch { last_ticks: 0 }
    }

    pub fn start(&mut self) {
        self.last_ticks = Instant::now().as_ticks();
    }

    pub fn measure(&self) -> Duration {
        let current_ticks = Instant::now().as_ticks();
        Duration::from_ticks(current_ticks - self.last_ticks)
    }
}
