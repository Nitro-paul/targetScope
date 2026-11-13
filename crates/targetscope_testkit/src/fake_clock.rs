// doc/shared_libraries.md#clock-and-failpoints

use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

use async_trait::async_trait;
use targetscope_common::clock::Clock;
use tokio::sync::watch;

pub struct FakeClock {
    start_wall: SystemTime,
    start_mono: Instant,
    offset: Mutex<Duration>,
    tick: watch::Sender<Duration>,
}

impl FakeClock {
    pub fn new(start: SystemTime) -> Self {
        let (tick, _) = watch::channel(Duration::ZERO);
        FakeClock { start_wall: start, start_mono: Instant::now(), offset: Mutex::new(Duration::ZERO), tick }
    }

    pub fn advance(&self, d: Duration) {
        let mut offset = self.offset.lock().unwrap();
        *offset += d;
        self.tick.send_replace(*offset);
    }

    fn offset(&self) -> Duration {
        *self.offset.lock().unwrap()
    }
}

#[async_trait]
impl Clock for FakeClock {
    fn now(&self) -> SystemTime {
        self.start_wall + self.offset()
    }

    fn monotonic(&self) -> Instant {
        self.start_mono + self.offset()
    }

    async fn sleep(&self, d: Duration) {
        let target = self.offset() + d;
        let mut rx = self.tick.subscribe();
        while *rx.borrow_and_update() < target {
            if rx.changed().await.is_err() {
                return;
            }
        }
    }
}
