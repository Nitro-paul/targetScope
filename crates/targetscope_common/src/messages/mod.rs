//! Envelopes, error codes and the messages of every program (doc/interfaces.md#message-format).
//!
//! Every program has a request enum and, if it sends events, an event enum. Each request is
//! also its own struct implementing [`Request`], which links it to its response type, its
//! stream and its timeout.

use std::time::Duration;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

pub mod analyzer;
pub mod camera_reader;
pub mod database_handler;
pub mod exporter;
pub mod image_processor;
pub mod importer;
pub mod launcher;
pub mod session_manager;
pub mod statistics;

/// Interface version of all messages (doc/interfaces.md#request-envelope).
pub const VERSION: u32 = 1;

/// Links a request to its response type, the receiver's stream and the timeout of its type.
pub trait Request: Serialize {
    type Response: DeserializeOwned;
    /// e.g. "analyzer:requests"
    const STREAM: &'static str;
    /// Per request type, see doc/interfaces.md#timeouts.
    const TIMEOUT: Duration;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEnvelope<R> {
    pub version: u32,
    pub request_id: Uuid,
    pub reply_to: Option<String>,
    /// flattened: "type" + "payload"
    #[serde(flatten)]
    pub request: R,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    pub r#type: String,
    pub version: u32,
    pub request_id: Uuid,
    pub status: Status,
    pub error: Option<ErrorBody>,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<E> {
    pub version: u32,
    pub event_id: Uuid,
    pub source: String,
    pub timestamp_ms: i64,
    /// flattened: "type" + "payload"
    #[serde(flatten)]
    pub event: E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Error,
}

/// doc/interfaces.md#error-codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidMessage,
    ValidationFailed,
    NotFound,
    InUse,
    SettingNotSet,
    InvalidState,
    Unavailable,
    ProcessingFailed,
    IoError,
    DatabaseError,
    /// Set by the sender when no response arrived in time; never sent by a receiver.
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Error returned by a request handler; turned into an error response by the serve loop.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{code:?}: {message}")]
pub struct ServiceError {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl ServiceError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }
}

/// `ping` is understood by every program (doc/interfaces.md#ping).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ping {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PingResponse {
    pub program: String,
    pub program_version: String,
    pub language: String,
    pub started_at: i64,
}
