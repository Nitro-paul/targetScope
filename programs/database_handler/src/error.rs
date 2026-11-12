// doc/programs/database_handler.md#errors

use targetscope_common::messages::{ErrorCode, ServiceError};

#[derive(thiserror::Error, Debug)]
pub enum HandlerError {
    #[error("invalid message: {0}")]
    InvalidMessage(String),
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("in use: {0}")]
    InUse(String),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
}

impl HandlerError {
    // doc/interfaces.md#error-codes
    pub fn code(&self) -> ErrorCode {
        match self {
            HandlerError::InvalidMessage(_) => ErrorCode::InvalidMessage,
            HandlerError::ValidationFailed(_) => ErrorCode::ValidationFailed,
            HandlerError::NotFound(_) => ErrorCode::NotFound,
            HandlerError::InUse(_) => ErrorCode::InUse,
            HandlerError::InvalidState(_) => ErrorCode::InvalidState,
            HandlerError::Io(_) => ErrorCode::IoError,
            HandlerError::Database(e) if e.to_string().starts_with("in_use:") => ErrorCode::InUse,
            HandlerError::Database(_) => ErrorCode::DatabaseError,
        }
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        HandlerError::ValidationFailed(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        HandlerError::NotFound(msg.into())
    }
}

impl From<HandlerError> for ServiceError {
    fn from(e: HandlerError) -> Self {
        let message = match &e {
            HandlerError::InvalidMessage(m)
            | HandlerError::ValidationFailed(m)
            | HandlerError::NotFound(m)
            | HandlerError::InUse(m)
            | HandlerError::InvalidState(m) => m.clone(),
            other => other.to_string(),
        };
        ServiceError::new(e.code(), message)
    }
}

pub type Result<T> = std::result::Result<T, HandlerError>;
