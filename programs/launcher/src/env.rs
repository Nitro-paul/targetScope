//! Bootstrap environment for every program (doc/programs/launcher.md#bootstrap-environment).
//! The five TARGETSCOPE_* variables can't be overridden by `[env]` (ignored with a warning).

use crate::config::LauncherConfig;

pub const PROTECTED: [&str; 5] = [
    "TARGETSCOPE_REDIS_URL",
    "TARGETSCOPE_DATA_DIR",
    "TARGETSCOPE_DB_PATH",
    "TARGETSCOPE_LOG_DIR",
    "TARGETSCOPE_LOG",
];

/// The variables to set for every program: the five bootstrap variables plus `[env]`.
pub fn program_env(config: &LauncherConfig) -> Vec<(String, String)> {
    let _ = config;
    todo!()
}
