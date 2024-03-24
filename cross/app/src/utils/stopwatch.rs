use embassy_time::{Instant, Duration};

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
