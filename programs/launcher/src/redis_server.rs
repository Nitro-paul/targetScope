//! Redis as a child process (doc/programs/launcher.md#redis):
//! `redis-server --port <port> --bind 127.0.0.1 --dir <data_dir>/redis --appendonly yes
//! --appendfsync everysec --save ""`. Refuses to use a port that is already in use.

use std::time::Duration;

use crate::config::LauncherConfig;

pub const READY_TIMEOUT: Duration = Duration::from_secs(10);

pub struct RedisProcess {
    pub child: tokio::process::Child,
}

/// Starts Redis and waits until PING answers (at most READY_TIMEOUT).
pub async fn start(config: &LauncherConfig) -> Result<RedisProcess, String> {
    let _ = config;
    todo!()
}

impl RedisProcess {
    /// SHUTDOWN (writes the AOF), then waits for the process.
    pub async fn shutdown(self) -> Result<(), String> {
        todo!()
    }
}
