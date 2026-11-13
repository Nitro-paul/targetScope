// doc/testing/overview.md#test-kit

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use targetscope_common::bus::{Bus, RequestHandler};
use targetscope_common::messages::{ErrorCode, ServiceError};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Answer {
    Ok(Value),
    Error(ErrorCode, String),
    Delayed(Duration, Box<Answer>),
    NoAnswer,
    Computed(Responder),
}

// doc/testing/overview.md#test-kit
#[derive(Clone)]
pub struct Responder(pub Arc<dyn Fn(&Value) -> Answer + Send + Sync>);

impl Responder {
    pub fn answer(f: impl Fn(&Value) -> Answer + Send + Sync + 'static) -> Answer {
        Answer::Computed(Responder(Arc::new(f)))
    }
}

impl std::fmt::Debug for Responder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Responder")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawRequest {
    pub r#type: String,
    pub payload: Value,
}

#[derive(Default)]
struct Script {
    answers: HashMap<String, Answer>,
    received: Vec<(Uuid, String, Value)>,
}

struct Handler(Arc<Mutex<Script>>);

#[async_trait]
impl RequestHandler<RawRequest> for Handler {
    async fn handle(&self, request_id: Uuid, req: RawRequest) -> Result<Value, ServiceError> {
        let answer = {
            let mut s = self.0.lock().unwrap();
            s.received.push((request_id, req.r#type.clone(), req.payload.clone()));
            s.answers.get(&req.r#type).cloned()
        };
        let mut answer = answer.unwrap_or(Answer::Error(ErrorCode::InvalidMessage, format!("unscripted {}", req.r#type)));
        loop {
            match answer {
                Answer::Ok(v) => return Ok(v),
                Answer::Error(code, msg) => return Err(ServiceError::new(code, msg)),
                Answer::Delayed(d, inner) => {
                    tokio::time::sleep(d).await;
                    answer = *inner;
                }
                Answer::NoAnswer => std::future::pending::<()>().await,
                Answer::Computed(Responder(f)) => answer = f(&req.payload),
            }
        }
    }
}

pub struct FakeProgram {
    pub name: String,
    pub bus: Bus,
    script: Arc<Mutex<Script>>,
    task: tokio::task::JoinHandle<()>,
}

impl FakeProgram {
    pub async fn start(redis_url: &str, name: &str) -> Self {
        let bus = Bus::connect(redis_url, name).await.expect("redis");
        let script = Arc::new(Mutex::new(Script::default()));
        let handler = Arc::new(Handler(script.clone()));
        let serving = bus.clone();
        let task = tokio::spawn(async move {
            let _ = serving.serve_concurrent(handler, 64).await;
        });
        FakeProgram { name: name.to_string(), bus, script, task }
    }

    pub fn on(&self, r#type: &str, answer: Answer) {
        self.script.lock().unwrap().answers.insert(r#type.to_string(), answer);
    }

    pub fn requests(&self) -> Vec<(String, Value)> {
        self.script.lock().unwrap().received.iter().map(|(_, t, p)| (t.clone(), p.clone())).collect()
    }

    pub fn requests_with_ids(&self) -> Vec<(Uuid, String, Value)> {
        self.script.lock().unwrap().received.clone()
    }

    pub fn requests_of(&self, r#type: &str) -> Vec<Value> {
        self.requests().into_iter().filter(|(t, _)| t == r#type).map(|(_, p)| p).collect()
    }
}

impl Drop for FakeProgram {
    fn drop(&mut self) {
        self.task.abort();
    }
}
