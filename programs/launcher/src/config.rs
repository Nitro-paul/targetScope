//! `launcher.toml` (doc/programs/launcher.md#configuration-launchertoml).
//! If the file doesn't exist, the default (config/launcher.toml) is written and used.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The default configuration, written when no file exists.
pub const DEFAULT_CONFIG: &str = include_str!("../../../config/launcher.toml");

#[derive(Debug, Clone, Deserialize)]
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
pub struct Paths {
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
    pub bin_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub binary: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProgramSpec {
    pub name: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Default: <bin_dir>/<name>
    pub binary: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read {0}: {1}")]
    Io(PathBuf, std::io::Error),
    #[error("invalid configuration: {0}")]
    Invalid(String),
}

/// Loads the file (writing the default first if it doesn't exist) and expands "~".
pub fn load(path: &Path) -> Result<LauncherConfig, ConfigError> {
    let _ = path;
    todo!()
}

/// Default path: ~/.config/targetscope/launcher.toml
pub fn default_path() -> PathBuf {
    todo!()
}
