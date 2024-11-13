#![cfg_attr(not(test), no_std)]

use core::{future::Future, time::Duration};

pub mod motion;
pub mod stepper;
pub mod planner;

pub trait TimerTrait {
    fn after(duration: Duration) -> impl Future<Output=()>;
}
