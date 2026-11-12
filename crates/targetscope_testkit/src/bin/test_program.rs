// doc/testing/overview.md#test-kit
// doc/testing/launcher.md#setup

use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;
use targetscope_common::bus::{Bus, RequestHandler};
use targetscope_common::messages::{ErrorCode, ServiceError};
use uuid::Uuid;

struct NoRequests;

#[async_trait]
impl RequestHandler<Value> for NoRequests {
    async fn handle(&self, _id: Uuid, _req: Value) -> Result<Value, ServiceError> {
        Err(ServiceError::new(ErrorCode::InvalidMessage, "test program"))
    }
}

fn args() -> Vec<String> {
    let argv0 = std::env::args().next().unwrap_or_default();
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if let Ok(extra) = std::fs::read_to_string(format!("{argv0}.args")) {
        args.extend(extra.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string));
    }
    args
}

#[tokio::main]
async fn main() {
    let name = PathBuf::from(std::env::args().next().unwrap_or_default())
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let args = args();
    let value = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
    let values = |flag: &str| {
        args.iter().enumerate().filter(|(_, a)| *a == flag).filter_map(|(i, _)| args.get(i + 1).cloned()).collect::<Vec<_>>()
    };

    if args.iter().any(|a| a == "--ignore-sigterm") {
        unsafe { libc::signal(libc::SIGTERM, libc::SIG_IGN) };
    }
    if args.iter().any(|a| a == "--print-env") {
        for (k, v) in std::env::vars().filter(|(k, _)| k.starts_with("TARGETSCOPE_") || k.starts_with("TEST_")) {
            println!("{k}={v}");
        }
    }
    for line in values("--print-stdout") {
        println!("{line}");
    }
    for line in values("--print-stderr") {
        eprintln!("{line}");
    }
    let _ = std::io::stdout().flush();

    if let Some(ms) = value("--exit-after-ms") {
        let code: i32 = value("--exit-code").and_then(|c| c.parse().ok()).unwrap_or(0);
        let ms: u64 = ms.parse().unwrap_or(0);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(ms)).await;
            std::process::exit(code);
        });
    }

    if !args.iter().any(|a| a == "--no-ping") {
        let delay: u64 = value("--ping-delay-ms").and_then(|d| d.parse().ok()).unwrap_or(0);
        tokio::time::sleep(Duration::from_millis(delay)).await;
        let url = std::env::var("TARGETSCOPE_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into());
        if let Ok(bus) = Bus::connect(&url, &name).await {
            tokio::spawn(async move {
                let _ = bus.serve(NoRequests).await;
            });
        }
    }
    std::future::pending::<()>().await;
}
