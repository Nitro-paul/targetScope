// doc/testing/overview.md#failpoints
// doc/shared_libraries.md#clock-and-failpoints

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

#[derive(Debug, Clone)]
pub enum Action {
    ReturnError,
    Sleep(std::time::Duration),
    Abort,
}

impl Action {
    fn spec(&self) -> String {
        match self {
            Action::ReturnError => "return".into(),
            Action::Sleep(d) => format!("sleep({})", d.as_millis()),
            Action::Abort => "return(abort)".into(),
        }
    }
}

pub fn env_value(points: &[(&str, Action)]) -> String {
    points.iter().map(|(name, a)| format!("{name}={}", a.spec())).collect::<Vec<_>>().join(";")
}

pub fn set(name: &str, action: Action) {
    fail::cfg(name, &action.spec()).expect("failpoint");
}

pub fn clear(name: &str) {
    fail::remove(name);
}
