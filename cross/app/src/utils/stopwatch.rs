use embassy_time::{Duration, Instant};
use fs::filesystem::timestamp::{TimeSource, Timestamp};

#[derive(Clone, Copy)]
pub struct Clock {
    start_ticks: u64,
    stop_ticks: u64,
    elapsed_ticks: u64,
    running: bool,
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock {
    pub fn new() -> Clock {
        Clock { start_ticks: 0, stop_ticks: 0, elapsed_ticks: 0, running: false }
    }

    fn now() -> Instant{
        Instant::now()
    }

    pub fn start(&mut self) {
        if !self.running{
            self.start_ticks = Clock::now().as_ticks();
        }
    }

    pub fn stop(&mut self) {
        if self.running{
            self.stop_ticks = Clock::now().as_ticks();
            self.elapsed_ticks += self.stop_ticks - self.start_ticks;
            self.running = false;
        }
    }

    pub fn measure(&self) -> Duration {
        let elapsed_ticks = if self.running {
            self.elapsed_ticks + Clock::now().as_ticks() - self.start_ticks
        }
        else{
            self.elapsed_ticks
        };
        Duration::from_ticks(elapsed_ticks)
    }

    pub fn reset(&mut self){
        self.start_ticks = 0;
        self.stop_ticks = 0;
        self.elapsed_ticks = 0;
    }
}

impl TimeSource for Clock {
    fn get_timestamp(&self) -> fs::filesystem::timestamp::Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_day: 0,
            zero_indexed_month: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}
