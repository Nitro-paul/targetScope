// doc/testing/overview.md#test-kit

use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use targetscope_common::bus::{Bus, EventReader, ReadMode};
use targetscope_common::messages::EventEnvelope;
use uuid::Uuid;

pub struct EventProbe {
    reader: EventReader<Value>,
}

impl EventProbe {
    pub async fn start(redis_url: &str, program: &str) -> Self {
        let bus = Bus::connect(redis_url, &format!("probe_{}", Uuid::new_v4().simple())).await.expect("redis");
        let after = bus.newest_event_id(program).await.expect("newest id");
        EventProbe { reader: bus.events::<Value>(program, ReadMode::After(after)) }
    }

    pub async fn next(&mut self, timeout: Duration) -> Option<EventEnvelope> {
        tokio::time::timeout(timeout, self.reader.next()).await.ok().and_then(Result::ok).map(|e| e.envelope)
    }

    pub async fn expect(&mut self, r#type: &str, timeout: Duration) -> EventEnvelope {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let left = deadline.saturating_duration_since(tokio::time::Instant::now());
            match self.next(left).await {
                Some(env) if env.r#type == r#type => return env,
                Some(_) => continue,
                None => panic!("no {type} event within {timeout:?}"),
            }
        }
    }

    pub async fn expect_none(&mut self, within: Duration) {
        if let Some(env) = self.next(within).await {
            panic!("unexpected event {}", env.r#type);
        }
    }
}

// doc/testing/overview.md#test-kit
pub async fn publish_as<E: Serialize + Sync>(redis_url: &str, source: &str, event: &E) -> EventEnvelope {
    let bus = Bus::connect(redis_url, source).await.expect("redis");
    bus.publish(event).await.expect("publish")
}
