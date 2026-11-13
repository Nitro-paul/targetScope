// doc/shared_libraries.md#bootstrap-configuration

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapConfig {
    pub redis_url: String,
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub log_dir: PathBuf,
    pub log_level: String,
}

impl BootstrapConfig {
    pub fn from_env() -> Self {
        Self::from_vars(|name| std::env::var(name).ok().filter(|v| !v.is_empty()))
    }

    pub fn from_vars(var: impl Fn(&str) -> Option<String>) -> Self {
        let data_dir = expand_home(&var("TARGETSCOPE_DATA_DIR").unwrap_or_else(|| "~/.local/share/targetscope".into()));
        let db_path = var("TARGETSCOPE_DB_PATH").map(|p| expand_home(&p)).unwrap_or_else(|| data_dir.join("targetscope.db"));
        BootstrapConfig {
            redis_url: var("TARGETSCOPE_REDIS_URL").unwrap_or_else(|| "redis://127.0.0.1:6379/".into()),
            db_path,
            log_dir: expand_home(&var("TARGETSCOPE_LOG_DIR").unwrap_or_else(|| "~/.local/state/targetscope/logs".into())),
            log_level: var("TARGETSCOPE_LOG").unwrap_or_else(|| "info".into()),
            data_dir,
        }
    }
}

// doc/shared_libraries.md#settings-cache
pub fn expand_home(path: &str) -> PathBuf {
    let home = || std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    if path == "~" {
        home()
    } else if let Some(rest) = path.strip_prefix("~/") {
        home().join(rest)
    } else {
        Path::new(path).to_path_buf()
    }
}
