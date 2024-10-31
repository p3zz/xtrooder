#![cfg_attr(not(test), no_std)]

use core::time::Duration;

pub mod motion;
pub mod stepper;
pub mod planner;

pub trait TimerTrait {
    async fn after(duration: Duration);
}
