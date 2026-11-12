//! Time source (doc/shared_libraries.md#clock-and-failpoints). All time-dependent code reads
//! time through a `Clock`, so tests can use the `FakeClock` of the test kit.

use std::time::{Duration, Instant, SystemTime};

use async_trait::async_trait;

#[async_trait]
pub trait Clock: Send + Sync {
    /// Wall clock (timestamps, saved_at, periods).
    fn now(&self) -> SystemTime;
    /// Durations and timeouts.
    fn monotonic(&self) -> Instant;
    async fn sleep(&self, d: Duration);
}

/// The real clock, used in production.
pub struct SystemClock;

#[async_trait]
impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }

    fn monotonic(&self) -> Instant {
        Instant::now()
    }

    async fn sleep(&self, d: Duration) {
        tokio::time::sleep(d).await
    }
}
