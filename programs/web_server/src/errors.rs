// doc/programs/web_server.md#error-mapping

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};
use targetscope_common::bus::BusError;
use targetscope_common::messages::{ErrorCode, ServiceError};

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &str, message: impl Into<String>) -> Self {
        ApiError { status, code: code.into(), message: message.into(), details: None }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "validation_failed", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", message)
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

// doc/programs/web_server.md#error-mapping
pub fn status_for(code: ErrorCode) -> StatusCode {
    match code {
        ErrorCode::InvalidMessage | ErrorCode::ValidationFailed => StatusCode::BAD_REQUEST,
        ErrorCode::NotFound => StatusCode::NOT_FOUND,
        ErrorCode::InUse | ErrorCode::InvalidState | ErrorCode::SettingNotSet => StatusCode::CONFLICT,
        ErrorCode::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        ErrorCode::ProcessingFailed => StatusCode::UNPROCESSABLE_ENTITY,
        ErrorCode::Timeout => StatusCode::GATEWAY_TIMEOUT,
        ErrorCode::IoError | ErrorCode::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl From<ServiceError> for ApiError {
    fn from(e: ServiceError) -> Self {
        ApiError { status: status_for(e.code), code: e.code.as_str().into(), message: e.message, details: e.details }
    }
}

impl From<BusError> for ApiError {
    fn from(e: BusError) -> Self {
        e.to_service_error().into()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut body = json!({ "code": self.code, "message": self.message });
        if let Some(d) = self.details {
            body["details"] = d;
        }
        (self.status, Json(body)).into_response()
    }
}

pub type ApiResult<T = Json<Value>> = Result<T, ApiError>;
