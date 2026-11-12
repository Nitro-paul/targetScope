//! `RedisTestServer`: `redis-server` on a free port with its own temporary folder and AOF on.
//! Can be stopped and started again on the same folder (Redis restart tests).

use std::path::PathBuf;

pub struct RedisTestServer {
    pub port: u16,
    pub dir: PathBuf,
    // TODO: child process, tempdir guard
}

impl RedisTestServer {
    pub async fn start() -> Self {
        todo!()
    }

    pub fn url(&self) -> String {
        format!("redis://127.0.0.1:{}/", self.port)
    }

    pub async fn stop(&mut self) {
        todo!()
    }

    /// Starts again on the same folder and port.
    pub async fn restart(&mut self) {
        todo!()
    }

    /// FLUSHALL, for tests sharing one server.
    pub async fn flush(&self) {
        todo!()
    }
}
