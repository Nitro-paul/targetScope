//! `FakeProgram`: serves `<program>:requests` with scripted answers per request type
//! (response, error code, delay, no answer) and records every request it gets.

use std::time::Duration;

use targetscope_common::messages::ErrorCode;

#[derive(Debug, Clone)]
pub enum Answer {
    Ok(serde_json::Value),
    Error(ErrorCode, String),
    Delayed(Duration, Box<Answer>),
    NoAnswer,
}

pub struct FakeProgram {
    pub name: String,
    // TODO: script map type -> answers, recorded requests, serve task
}

impl FakeProgram {
    pub async fn start(redis_url: &str, name: &str) -> Self {
        let _ = (redis_url, name);
        todo!()
    }

    /// Answer every request of this type like this (until changed).
    pub fn on(&self, r#type: &str, answer: Answer) {
        let _ = (r#type, answer);
        todo!()
    }

    /// All requests received so far: (type, payload).
    pub fn requests(&self) -> Vec<(String, serde_json::Value)> {
        todo!()
    }
}
