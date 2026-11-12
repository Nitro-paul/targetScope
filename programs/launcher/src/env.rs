// doc/programs/launcher.md#bootstrap-environment

use crate::config::LauncherConfig;

pub const PROTECTED: [&str; 5] =
    ["TARGETSCOPE_REDIS_URL", "TARGETSCOPE_DATA_DIR", "TARGETSCOPE_DB_PATH", "TARGETSCOPE_LOG_DIR", "TARGETSCOPE_LOG"];

pub fn program_env(c: &LauncherConfig) -> Vec<(String, String)> {
    let mut env = vec![
        ("TARGETSCOPE_REDIS_URL".to_string(), c.redis_url()),
        ("TARGETSCOPE_DATA_DIR".to_string(), c.paths.data_dir.display().to_string()),
        ("TARGETSCOPE_DB_PATH".to_string(), c.paths.data_dir.join("targetscope.db").display().to_string()),
        ("TARGETSCOPE_LOG_DIR".to_string(), c.paths.log_dir.display().to_string()),
        ("TARGETSCOPE_LOG".to_string(), c.log.level.clone()),
    ];
    for (k, v) in &c.env {
        if PROTECTED.contains(&k.as_str()) {
            tracing::warn!(variable = %k, "[env] can't override a bootstrap variable; ignored");
        } else {
            env.push((k.clone(), v.clone()));
        }
    }
    env
}
