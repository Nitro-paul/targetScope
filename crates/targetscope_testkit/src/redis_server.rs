// doc/testing/overview.md#test-kit

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub struct RedisTestServer {
    pub port: u16,
    pub dir: PathBuf,
    child: Option<Child>,
    _tmp: tempfile::TempDir,
}

pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr()).map(|a| a.port()).expect("free port")
}

impl RedisTestServer {
    pub async fn start() -> Self {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut server = RedisTestServer { port: free_port(), dir: tmp.path().to_path_buf(), child: None, _tmp: tmp };
        server.spawn().await;
        server
    }

    pub fn url(&self) -> String {
        format!("redis://127.0.0.1:{}/", self.port)
    }

    async fn spawn(&mut self) {
        let child = Command::new("redis-server")
            .args(["--port", &self.port.to_string(), "--bind", "127.0.0.1", "--appendonly", "yes"])
            .args(["--appendfsync", "everysec", "--save", ""])
            .arg("--dir")
            .arg(&self.dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("redis-server in PATH");
        self.child = Some(child);
        let client = redis::Client::open(self.url()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Ok(mut c) = client.get_multiplexed_async_connection().await {
                let pong: Result<String, _> = redis::cmd("PING").query_async(&mut c).await;
                if pong.is_ok() {
                    return;
                }
            }
            assert!(Instant::now() < deadline, "redis-server did not start");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    pub async fn stop(&mut self) {
        if let Ok(mut c) = redis::Client::open(self.url()).unwrap().get_multiplexed_async_connection().await {
            let _: Result<(), _> = redis::cmd("SHUTDOWN").query_async(&mut c).await;
        }
        if let Some(mut child) = self.child.take() {
            let _ = child.wait();
        }
    }

    pub async fn restart(&mut self) {
        self.stop().await;
        self.spawn().await;
    }

    pub async fn flush(&self) {
        let mut c = self.connection().await;
        let _: () = redis::cmd("FLUSHALL").query_async(&mut c).await.unwrap();
    }

    pub async fn connection(&self) -> redis::aio::MultiplexedConnection {
        redis::Client::open(self.url()).unwrap().get_multiplexed_async_connection().await.unwrap()
    }
}

impl Drop for RedisTestServer {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
