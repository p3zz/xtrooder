use chrono::Timelike;
use embassy_time::{Duration, Instant};
use fs::filesystem::timestamp::{TimeSource, Timestamp};

#[derive(Clone, Copy)]
pub struct Clock {
    last_ticks: u64,
}

impl Clock {
    pub fn new() -> Clock {
        Clock { last_ticks: 0 }
    }

    pub fn start(&mut self) {
        self.last_ticks = Instant::now().as_ticks();
    }

    pub fn measure(&self) -> Duration {
        let current_ticks = Instant::now().as_ticks();
        Duration::from_ticks(current_ticks - self.last_ticks)
    }
}

impl TimeSource for Clock{
    fn get_timestamp(&self) -> fs::filesystem::timestamp::Timestamp {
        Timestamp{
            year_since_1970: 0,
            zero_indexed_day: 0,
            zero_indexed_month: 0,
            hours: 0,
            minutes: 0,
            seconds: 0
        }
    }
}