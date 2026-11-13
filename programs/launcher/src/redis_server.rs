// doc/programs/launcher.md#redis

use std::path::PathBuf;
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
    let binary = find_binary(&c.redis.binary).ok_or_else(|| {
        format!(
            "{} not found (searched the PATH, $TARGETSCOPE_DEPS/bin and ~/.local/opt/targetscope-deps/bin); \
             install Redis or Valkey, or set [redis] binary in the launcher configuration to its full path",
            c.redis.binary
        )
    })?;
    Command::new(&binary)
        .args(["--port", &c.redis.port.to_string(), "--bind", "127.0.0.1"])
        .arg("--dir")
        .arg(&dir)
        .args(["--appendonly", "yes", "--appendfsync", "everysec", "--save", ""])
        .arg("--logfile")
        .arg(c.paths.log_dir.join("redis.log"))
        .stdin(Stdio::null())
        .kill_on_drop(false)
        .spawn()
        .map_err(|e| format!("{} could not be started: {e}", binary.display()))
}

// doc/programs/launcher.md#redis
pub fn find_binary(binary: &str) -> Option<PathBuf> {
    find_binary_in(binary, std::env::var_os("PATH"), std::env::var_os("TARGETSCOPE_DEPS"), std::env::var_os("HOME"))
}

// doc/programs/launcher.md#redis
pub fn find_binary_in(
    binary: &str,
    path: Option<std::ffi::OsString>,
    deps: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
) -> Option<PathBuf> {
    if binary.contains('/') {
        let path = targetscope_common::config::expand_home(binary);
        return path.is_file().then_some(path);
    }
    let path_dirs: Vec<PathBuf> = path.map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    let deps_dirs = deps
        .map(PathBuf::from)
        .into_iter()
        .chain(home.map(|h| PathBuf::from(h).join(".local/opt/targetscope-deps")))
        .map(|d| d.join("bin"));
    path_dirs.into_iter().chain(deps_dirs).map(|d| d.join(binary)).find(|p| p.is_file())
}

pub async fn wait_ready(url: &str, timeout: Duration) -> Result<(), String> {
    let client = redis::Client::open(url).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + timeout;
    loop {
        let ok = match tokio::time::timeout(Duration::from_millis(500), client.get_multiplexed_async_connection()).await {
            Ok(Ok(mut conn)) => {
                matches!(
                    tokio::time::timeout(Duration::from_millis(500), redis::cmd("PING").query_async::<String>(&mut conn)).await,
                    Ok(Ok(_))
                )
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
