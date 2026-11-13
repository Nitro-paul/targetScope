// doc/testing/system.md#setup

use std::path::PathBuf;
use std::time::Duration;

pub struct System {
    pub home: tempfile::TempDir,
    pub web_url: String,
}

impl System {
    pub async fn start() -> Self {
        todo!()
    }

    pub fn data_dir(&self) -> PathBuf {
        todo!()
    }

    pub fn kill(&self, program: &str) {
        let _ = program;
        todo!()
    }

    pub async fn stop(self, timeout: Duration) -> i32 {
        let _ = timeout;
        todo!()
    }
}
