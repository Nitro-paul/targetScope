// doc/programs/launcher.md#redis

use std::process::Stdio;
use std::time::{Duration, Instant};

use tokio::process::{Child, Command};

use crate::config::LauncherConfig;

pub const READY_TIMEOUT: Duration = Duration::from_secs(10);

pub fn port_in_use(port: u16) -> bool {
    std::net::TcpStream::connect_timeout(&([127, 0, 0, 1], port).into(), Duration::from_millis(300)).is_ok()
}

// doc/programs/launcher.md#redis
pub fn spawn(c: &LauncherConfig) -> Result<Child, String> {
    let dir = c.paths.data_dir.join("redis");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    Command::new(&c.redis.binary)
        .args(["--port", &c.redis.port.to_string(), "--bind", "127.0.0.1"])
        .arg("--dir")
        .arg(&dir)
        .args(["--appendonly", "yes", "--appendfsync", "everysec", "--save", ""])
        .arg("--logfile")
        .arg(c.paths.log_dir.join("redis.log"))
        .stdin(Stdio::null())
        .kill_on_drop(false)
        .spawn()
        .map_err(|e| format!("{} could not be started: {e}", c.redis.binary))
}

pub async fn wait_ready(url: &str, timeout: Duration) -> Result<(), String> {
    let client = redis::Client::open(url).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + timeout;
    loop {
        let ok = match tokio::time::timeout(Duration::from_millis(500), client.get_multiplexed_async_connection()).await {
            Ok(Ok(mut conn)) => {
                matches!(tokio::time::timeout(Duration::from_millis(500), redis::cmd("PING").query_async::<String>(&mut conn)).await, Ok(Ok(_)))
            }
            _ => false,
        };
        if ok {
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err(format!("Redis doesn't answer PING within {} s", timeout.as_secs()));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

// doc/programs/launcher.md#shutdown
pub async fn shutdown(url: &str) {
    if let Ok(client) = redis::Client::open(url) {
        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
            let _: Result<(), _> = redis::cmd("SHUTDOWN").query_async(&mut conn).await;
        }
    }
}
