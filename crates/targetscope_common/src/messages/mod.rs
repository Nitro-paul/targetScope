// doc/interfaces.md#message-format
// doc/shared_libraries.md#messages

use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

// doc/interfaces.md#request-envelope
pub const VERSION: u32 = 1;

// doc/shared_libraries.md#messages
pub trait Request: Serialize + Send + Sync {
    type Response: DeserializeOwned + Send;
    const TYPE: &'static str;
    const STREAM: &'static str;
    // doc/interfaces.md#timeouts
    const TIMEOUT: Duration;
}

// doc/interfaces.md#request-envelope
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestEnvelope {
    pub r#type: String,
    pub version: u32,
    pub request_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    pub payload: Value,
}

impl RequestEnvelope {
    pub fn new<R: Request>(request_id: Uuid, reply_to: Option<String>, req: &R) -> Self {
        RequestEnvelope {
            r#type: R::TYPE.to_string(),
            version: VERSION,
            request_id,
            reply_to,
            payload: serde_json::to_value(req).unwrap_or(Value::Null),
        }
    }

    // doc/interfaces.md#error-codes
    pub fn decode<R: DeserializeOwned>(&self) -> Result<R, ServiceError> {
        if self.version != VERSION {
            return Err(ServiceError::new(ErrorCode::InvalidMessage, format!("unsupported version {} of {}", self.version, self.r#type)));
        }
        decode_tagged(&self.r#type, &self.payload).map_err(|e| {
            let msg = e.to_string();
            let unknown_type = msg.contains(&format!("unknown variant `{}`", self.r#type));
            let code = if unknown_type { ErrorCode::InvalidMessage } else { ErrorCode::ValidationFailed };
            ServiceError::new(code, msg)
        })
    }

    pub fn ok_response(&self, payload: Value) -> ResponseEnvelope {
        ResponseEnvelope {
            r#type: format!("{}_response", self.r#type),
            version: self.version,
            request_id: self.request_id,
            status: Status::Ok,
            error: None,
            payload: Some(payload),
        }
    }

    pub fn error_response(&self, error: &ServiceError) -> ResponseEnvelope {
        ResponseEnvelope {
            r#type: format!("{}_response", self.r#type),
            version: self.version,
            request_id: self.request_id,
            status: Status::Error,
            error: Some(error.body()),
            payload: None,
        }
    }
}

// doc/interfaces.md#response-envelope
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseEnvelope {
    pub r#type: String,
    pub version: u32,
    pub request_id: Uuid,
    pub status: Status,
    pub error: Option<ErrorBody>,
    pub payload: Option<Value>,
}

// doc/interfaces.md#event-envelope
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventEnvelope {
    pub r#type: String,
    pub version: u32,
    pub event_id: Uuid,
    pub source: String,
    pub timestamp_ms: i64,
    pub payload: Value,
}

impl EventEnvelope {
    pub fn new<E: Serialize>(source: &str, timestamp_ms: i64, event: &E) -> Result<Self, serde_json::Error> {
        let tagged = serde_json::to_value(event)?;
        let r#type = tagged.get("type").and_then(Value::as_str).unwrap_or_default().to_string();
        let payload = tagged.get("payload").cloned().unwrap_or_else(|| Value::Object(Default::default()));
        Ok(EventEnvelope { r#type, version: VERSION, event_id: Uuid::new_v4(), source: source.to_string(), timestamp_ms, payload })
    }

    // doc/interfaces.md#event-envelope
    pub fn decode<E: DeserializeOwned>(&self) -> Result<E, serde_json::Error> {
        if self.version != VERSION {
            return Err(serde::de::Error::custom(format!("unsupported version {} of {}", self.version, self.r#type)));
        }
        decode_tagged(&self.r#type, &self.payload)
    }
}

fn decode_tagged<T: DeserializeOwned>(r#type: &str, payload: &Value) -> Result<T, serde_json::Error> {
    let mut tagged = serde_json::Map::new();
    tagged.insert("type".into(), Value::String(r#type.to_string()));
    tagged.insert("payload".into(), payload.clone());
    serde_json::from_value(Value::Object(tagged))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Error,
}

// doc/interfaces.md#error-codes
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
    Timeout,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::InvalidMessage => "invalid_message",
            ErrorCode::ValidationFailed => "validation_failed",
            ErrorCode::NotFound => "not_found",
            ErrorCode::InUse => "in_use",
            ErrorCode::SettingNotSet => "setting_not_set",
            ErrorCode::InvalidState => "invalid_state",
            ErrorCode::Unavailable => "unavailable",
            ErrorCode::ProcessingFailed => "processing_failed",
            ErrorCode::IoError => "io_error",
            ErrorCode::DatabaseError => "database_error",
            ErrorCode::Timeout => "timeout",
        }
    }
}

// doc/interfaces.md#response-envelope
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

// doc/shared_libraries.md#redis-bus
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("{}: {message}", code.as_str())]
pub struct ServiceError {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<Value>,
}

impl ServiceError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        ServiceError { code, message: message.into(), details: None }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ValidationFailed, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidState, message)
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unavailable, message)
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::IoError, message)
    }

    // doc/interfaces.md#error-codes
    pub fn setting_not_set(setting: &str) -> Self {
        Self::new(ErrorCode::SettingNotSet, format!("setting {setting} is not set")).with_details(serde_json::json!({ "setting": setting }))
    }

    pub fn body(&self) -> ErrorBody {
        ErrorBody { code: self.code, message: self.message.clone(), details: self.details.clone() }
    }
}

impl From<ErrorBody> for ServiceError {
    fn from(b: ErrorBody) -> Self {
        ServiceError { code: b.code, message: b.message, details: b.details }
    }
}

// doc/interfaces.md#ping
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ping {}

// doc/interfaces.md#ping
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PingResponse {
    pub program: String,
    pub program_version: String,
    pub language: String,
    pub started_at: i64,
}

// doc/interfaces.md#message-format
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Empty {}

// doc/interfaces.md#streams-and-keys
pub fn request_stream(program: &str) -> String {
    format!("{program}:requests")
}

pub fn response_stream(program: &str) -> String {
    format!("{program}:responses")
}

pub fn event_stream(program: &str) -> String {
    format!("{program}:events")
}

macro_rules! request {
    ($req:ty, $resp:ty, $type:literal, $secs:expr) => {
        impl $crate::messages::Request for $req {
            type Response = $resp;
            const TYPE: &'static str = $type;
            const STREAM: &'static str = STREAM;
            const TIMEOUT: ::std::time::Duration = ::std::time::Duration::from_secs($secs);
        }
    };
}
pub(crate) use request;
