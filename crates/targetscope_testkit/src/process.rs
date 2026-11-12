// doc/testing/overview.md#test-levels

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use targetscope_common::bus::Bus;
use uuid::Uuid;

pub struct ProgramUnderTest {
    pub name: String,
    child: Option<Child>,
}

impl ProgramUnderTest {
    pub async fn start(binary: &Path, name: &str, env: &[(String, String)]) -> Self {
        let p = Self::spawn(binary, name, env);
        let url = env.iter().find(|(k, _)| k == "TARGETSCOPE_REDIS_URL").map(|(_, v)| v.clone()).expect("redis url");
        wait_ready(&url, name, Duration::from_secs(20)).await;
        p
    }

    pub fn spawn(binary: &Path, name: &str, env: &[(String, String)]) -> Self {
        let child = Command::new(binary)
            .envs(env.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            .stdin(Stdio::null())
            .spawn()
            .unwrap_or_else(|e| panic!("start {}: {e}", binary.display()));
        ProgramUnderTest { name: name.to_string(), child: Some(child) }
    }

    pub fn pid(&self) -> u32 {
        self.child.as_ref().map(|c| c.id()).unwrap_or(0)
    }

    pub fn kill(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }

    pub fn terminate(&mut self, timeout: Duration) -> Option<std::process::ExitStatus> {
        let mut c = self.child.take()?;
        unsafe { libc::kill(c.id() as i32, libc::SIGTERM) };
        let deadline = Instant::now() + timeout;
        loop {
            if let Ok(Some(status)) = c.try_wait() {
                return Some(status);
            }
            if Instant::now() > deadline {
                let _ = c.kill();
                return c.wait().ok();
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn wait_exit(&mut self, timeout: Duration) -> Option<std::process::ExitStatus> {
        let c = self.child.as_mut()?;
        let deadline = Instant::now() + timeout;
        loop {
            if let Ok(Some(status)) = c.try_wait() {
                self.child = None;
                return Some(status);
            }
            if Instant::now() > deadline {
                return None;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for ProgramUnderTest {
    fn drop(&mut self) {
        self.kill();
    }
}

// doc/interfaces.md#ping
pub async fn wait_ready(redis_url: &str, program: &str, timeout: Duration) {
    let bus = Bus::connect(redis_url, &format!("ready_{}", Uuid::new_v4().simple())).await.expect("redis");
    let deadline = Instant::now() + timeout;
    while bus.ping(program, Duration::from_millis(500)).await.is_err() {
        assert!(Instant::now() < deadline, "{program} not ready");
    }
}
