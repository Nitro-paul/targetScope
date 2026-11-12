//! `TestBus`: a bus for the test itself; sends requests to the program under test,
//! with a chosen `request_id` if needed, and returns the full response envelope.

use targetscope_common::messages::ResponseEnvelope;
use uuid::Uuid;

pub struct TestBus {
    // TODO: targetscope_common::bus::Bus as program "test"
}

impl TestBus {
    pub async fn connect(redis_url: &str) -> Self {
        let _ = redis_url;
        todo!()
    }

    /// Sends `{type, payload}` to `<program>:requests` and waits for the response envelope.
    pub async fn send(
        &self,
        program: &str,
        r#type: &str,
        payload: serde_json::Value,
    ) -> ResponseEnvelope {
        self.send_with_id(program, Uuid::new_v4(), r#type, payload)
            .await
    }

    pub async fn send_with_id(
        &self,
        program: &str,
        request_id: Uuid,
        r#type: &str,
        payload: serde_json::Value,
    ) -> ResponseEnvelope {
        let _ = (program, request_id, r#type, payload);
        todo!()
    }

    /// Sends raw data (for invalid_message tests).
    pub async fn send_raw(&self, program: &str, data: &str) -> ResponseEnvelope {
        let _ = (program, data);
        todo!()
    }
}
