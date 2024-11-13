#![cfg_attr(not(test), no_std)]

use core::{future::Future, time::Duration};

pub mod motion;
pub mod planner;
pub mod stepper;

pub trait TimerTrait {
    fn after(duration: Duration) -> impl Future<Output = ()>;
}
