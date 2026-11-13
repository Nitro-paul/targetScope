// doc/programs/launcher.md#configuration-launchertoml

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use targetscope_common::config::expand_home;

pub const DEFAULT_CONFIG: &str = include_str!("../../../config/launcher.toml");

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LauncherConfig {
    pub paths: Paths,
    pub redis: RedisConfig,
    pub log: LogConfig,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(rename = "program", default)]
    pub programs: Vec<ProgramSpec>,
    pub restart: RestartPolicy,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Paths {
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
    pub bin_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedisConfig {
    pub binary: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgramSpec {
    pub name: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub binary: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestartPolicy {
    pub initial_backoff_s: u64,
    pub max_backoff_s: u64,
    pub stable_after_s: u64,
    pub max_restarts: u32,
    pub window_s: u64,
}

fn default_true() -> bool {
    true
}

impl LauncherConfig {
    pub fn binary(&self, p: &ProgramSpec) -> PathBuf {
        p.binary.as_ref().map(|b| expand_home(&b.to_string_lossy())).unwrap_or_else(|| self.paths.bin_dir.join(&p.name))
    }

    pub fn redis_url(&self) -> String {
        format!("redis://127.0.0.1:{}/", self.redis.port)
    }
}

// doc/programs/launcher.md#configuration-launchertoml
pub fn load(path: &Path) -> Result<LauncherConfig, String> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(path, DEFAULT_CONFIG).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut c: LauncherConfig = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    for p in [&mut c.paths.data_dir, &mut c.paths.log_dir, &mut c.paths.bin_dir] {
        *p = expand_home(&p.to_string_lossy());
    }
    crate::order::start_order(&c.programs)?;
    Ok(c)
}

pub fn default_path() -> PathBuf {
    expand_home("~/.config/targetscope/launcher.toml")
}
