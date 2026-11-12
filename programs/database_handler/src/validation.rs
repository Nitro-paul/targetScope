//! Validation rules per request (doc/programs/database_handler.md#validation-rules).

use crate::error::HandlerError;

// TODO: one function per request type, e.g.
/// create_session: at least one shot; shot_count = shots.len(); paper_no within 1..=paper_count,
/// at most one image per paper; notes at most 10 000 characters; … (doc/interfaces.md#create_session)
pub fn validate_create_session(payload: &serde_json::Value) -> Result<(), HandlerError> {
    let _ = payload;
    todo!()
}

/// A setting value against its value_rule (known_settings).
pub fn validate_setting_value(name: &str, value: &str, active: bool) -> Result<(), HandlerError> {
    let _ = (name, value, active);
    todo!()
}
