//! Switches the named failpoints of doc/testing/overview.md#failpoints on.
//! In-process failpoints use the `fail` crate; for a program under test they are passed
//! through the `FAILPOINTS` environment variable.

pub const BUS_BEFORE_RESPONSE: &str = "bus.before_response";
pub const BUS_BEFORE_ACK: &str = "bus.before_ack";
pub const SETTINGS_AFTER_NOTE_ID: &str = "settings.after_note_id";
pub const DB_BEFORE_COMMIT: &str = "db.before_commit";
pub const DB_AFTER_COMMIT: &str = "db.after_commit";
pub const DB_IMAGE_DIR_COPY: &str = "db.image_dir_copy";
pub const SM_BEFORE_EXEC: &str = "sm.before_exec";
pub const SM_AFTER_CREATE_SESSION_SENT: &str = "sm.after_create_session_sent";
pub const EXPORTER_BEFORE_RENAME: &str = "exporter.before_rename";
pub const IMPORTER_AFTER_RECORD: &str = "importer.after_record";

pub enum Action {
    ReturnError,
    Sleep(std::time::Duration),
    Abort,
}

/// Value of the FAILPOINTS environment variable for a program under test.
pub fn env_value(points: &[(&str, Action)]) -> String {
    let _ = points;
    todo!()
}
