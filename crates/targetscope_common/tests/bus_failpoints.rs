// doc/testing/shared_libraries.md#redis-bus-integration

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use targetscope_common::bus::{Bus, RequestHandler};
use targetscope_common::messages::{Request, ServiceError};
use targetscope_testkit::failpoints::{self, Action};
use targetscope_testkit::RedisTestServer;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
enum TestRequest {
    Echo(Echo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Echo {
    value: i64,
}

impl Request for Echo {
    type Response = Value;
    const TYPE: &'static str = "echo";
    const STREAM: &'static str = "b:requests";
    const TIMEOUT: Duration = Duration::from_millis(500);
}

struct Handler(Arc<Mutex<Vec<i64>>>);

#[async_trait]
impl RequestHandler<TestRequest> for Handler {
    async fn handle(&self, _id: Uuid, req: TestRequest) -> Result<Value, ServiceError> {
        let TestRequest::Echo(e) = req;
        self.0.lock().unwrap().push(e.value);
        Ok(json!({ "value": e.value }))
    }
}

// LIB-BUS-12
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_12_pending_requests_first() {
    let redis = RedisTestServer::start().await;
    let a = Bus::connect(&redis.url(), "a").await.unwrap();
    let b = Bus::connect(&redis.url(), "b").await.unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    failpoints::set("bus.before_ack", Action::ReturnError);
    let serving = b.clone();
    let h = Handler(seen.clone());
    let first = tokio::spawn(async move { serving.serve(h).await });
    let _ = a.request(Echo { value: 1 }).await;
    b.stop();
    let _ = first.await;
    failpoints::clear("bus.before_ack");
    let a2 = a.clone();
    let queued = tokio::spawn(async move { a2.request(Echo { value: 2 }).await });
    tokio::time::sleep(Duration::from_millis(100)).await;
    let b2 = Bus::connect(&redis.url(), "b").await.unwrap();
    let serving = b2.clone();
    let h = Handler(seen.clone());
    tokio::spawn(async move { serving.serve(h).await });
    let _ = queued.await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let order = seen.lock().unwrap().clone();
    assert_eq!(&order[..], &[1, 1, 2], "{order:?}");
}
