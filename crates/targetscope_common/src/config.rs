//! Bootstrap configuration from the environment set by the launcher
//! (doc/shared_libraries.md#bootstrap-configuration).

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapConfig {
    /// TARGETSCOPE_REDIS_URL, default redis://127.0.0.1:6379/
    pub redis_url: String,
    /// TARGETSCOPE_DATA_DIR, default ~/.local/share/targetscope
    pub data_dir: PathBuf,
    /// TARGETSCOPE_DB_PATH (database_handler only), default <data_dir>/targetscope.db
    pub db_path: PathBuf,
    /// TARGETSCOPE_LOG_DIR, default ~/.local/state/targetscope/logs
    pub log_dir: PathBuf,
    /// TARGETSCOPE_LOG, default "info" (tracing EnvFilter syntax)
    pub log_level: String,
}

impl BootstrapConfig {
    pub fn from_env() -> Self {
        todo!()
    }
}
