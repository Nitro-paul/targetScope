//! HandlerError (doc/programs/database_handler.md#errors).

use targetscope_common::messages::ErrorCode;

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
    /// e.g. an image folder change is already running
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
}

impl HandlerError {
    /// Error code sent in the response, see doc/interfaces.md#error-codes.
    pub fn code(&self) -> ErrorCode {
        match self {
            HandlerError::InvalidMessage(_) => ErrorCode::InvalidMessage,
            HandlerError::ValidationFailed(_) => ErrorCode::ValidationFailed,
            HandlerError::NotFound(_) => ErrorCode::NotFound,
            HandlerError::InUse(_) => ErrorCode::InUse,
            HandlerError::InvalidState(_) => ErrorCode::InvalidState,
            HandlerError::Io(_) => ErrorCode::IoError,
            // TODO: map trigger aborts ("record: …", "in_use: …") to their codes
            HandlerError::Database(_) => ErrorCode::DatabaseError,
        }
    }
}
