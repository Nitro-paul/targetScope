// doc/testing/overview.md#test-kit

use std::time::Duration;

use serde_json::Value;
use targetscope_common::bus::Bus;
use targetscope_common::messages::{request_stream, RequestEnvelope, ResponseEnvelope, VERSION};
use uuid::Uuid;

pub struct TestBus {
    pub bus: Bus,
    pub timeout: Duration,
}

impl TestBus {
    pub async fn connect(redis_url: &str) -> Self {
        let bus = Bus::connect(redis_url, &format!("test_{}", Uuid::new_v4().simple())).await.expect("redis");
        TestBus { bus, timeout: Duration::from_secs(10) }
    }

    pub async fn send(&self, program: &str, r#type: &str, payload: Value) -> ResponseEnvelope {
        self.send_with_id(program, Uuid::new_v4(), r#type, payload).await
    }

    pub async fn send_with_id(&self, program: &str, request_id: Uuid, r#type: &str, payload: Value) -> ResponseEnvelope {
        self.try_send(program, request_id, r#type, payload, self.timeout).await.expect("response")
    }

    pub async fn try_send(
        &self,
        program: &str,
        request_id: Uuid,
        r#type: &str,
        payload: Value,
        timeout: Duration,
    ) -> Option<ResponseEnvelope> {
        let env = RequestEnvelope { r#type: r#type.into(), version: VERSION, request_id, reply_to: None, payload };
        self.bus.send_envelope(&request_stream(program), env, timeout).await.ok()
    }

    pub async fn send_raw(&self, program: &str, data: &str) -> String {
        let mut c = self.bus.connection();
        redis::cmd("XADD").arg(request_stream(program)).arg("*").arg("data").arg(data).query_async(&mut c).await.unwrap()
    }

    pub fn response_stream(&self) -> String {
        targetscope_common::messages::response_stream(self.bus.program())
    }
}
