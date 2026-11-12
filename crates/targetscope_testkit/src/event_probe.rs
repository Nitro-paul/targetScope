//! `EventProbe`: reads a `<program>:events` stream and waits for expected events, with a
//! timeout; can also check that no event came.

use std::time::Duration;

pub struct EventProbe {
    // TODO: reader of <program>:events starting at the current end
}

impl EventProbe {
    pub async fn start(redis_url: &str, program: &str) -> Self {
        let _ = (redis_url, program);
        todo!()
    }

    /// Waits for the next event of this type; panics after the timeout.
    pub async fn expect(&mut self, r#type: &str, timeout: Duration) -> serde_json::Value {
        let _ = (r#type, timeout);
        todo!()
    }

    /// Panics if any event arrives within the duration.
    pub async fn expect_none(&mut self, within: Duration) {
        let _ = within;
        todo!()
    }
}
