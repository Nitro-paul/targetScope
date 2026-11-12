//! Starts `targetscope` (the launcher) with a temporary HOME and stops it.

use std::path::PathBuf;
use std::time::Duration;

pub struct System {
    pub home: tempfile::TempDir,
    pub web_url: String,
    // TODO: launcher child process, redis port
}

impl System {
    /// Starts the launcher and waits until all programs are running (/api/system).
    pub async fn start() -> Self {
        todo!()
    }

    pub fn data_dir(&self) -> PathBuf {
        todo!()
    }

    /// kill -9 one program (by name), e.g. session_manager; "redis" for Redis.
    pub fn kill(&self, program: &str) {
        let _ = program;
        todo!()
    }

    /// SIGTERM to the launcher; returns its exit code.
    pub async fn stop(self, timeout: Duration) -> i32 {
        let _ = timeout;
        todo!()
    }
}
