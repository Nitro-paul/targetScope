//! `FakeClock`: controllable time; it only moves when a test advances it.

use std::time::{Duration, Instant, SystemTime};

use async_trait::async_trait;
use targetscope_common::clock::Clock;

pub struct FakeClock {
    // TODO: current wall time, current monotonic offset, sleepers
}

impl FakeClock {
    pub fn new(start: SystemTime) -> Self {
        let _ = start;
        todo!()
    }

    /// Moves both clocks forward and wakes due sleepers.
    pub fn advance(&self, d: Duration) {
        let _ = d;
        todo!()
    }
}

#[async_trait]
impl Clock for FakeClock {
    fn now(&self) -> SystemTime {
        todo!()
    }

    fn monotonic(&self) -> Instant {
        todo!()
    }

    async fn sleep(&self, d: Duration) {
        let _ = d;
        todo!()
    }
}
