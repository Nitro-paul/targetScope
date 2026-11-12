// doc/shared_libraries.md#program-startup

use std::time::Duration;

use crate::bus::Bus;
use crate::config::BootstrapConfig;
use crate::failpoints;
use crate::logging::{self, LogGuard};

pub struct Program {
    pub name: &'static str,
    pub config: BootstrapConfig,
    pub bus: Bus,
    _log: LogGuard,
}

pub async fn start(name: &'static str) -> Program {
    failpoints::setup();
    let config = BootstrapConfig::from_env();
    let log = logging::init(name, &config);
    let bus = connect(&config.redis_url, name).await;
    tracing::info!(program = name, "started");
    Program { name, config, bus, _log: log }
}

pub async fn connect(url: &str, name: &str) -> Bus {
    let mut backoff = Duration::from_millis(100);
    loop {
        match Bus::connect(url, name).await {
            Ok(bus) => return bus,
            Err(e) => {
                tracing::warn!(error = %e, "redis not reachable");
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(5));
            }
        }
    }
}

// doc/shared_libraries.md#program-startup
pub async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};
    let mut term = signal(SignalKind::terminate()).expect("SIGTERM handler");
    let mut int = signal(SignalKind::interrupt()).expect("SIGINT handler");
    tokio::select! {
        _ = term.recv() => {}
        _ = int.recv() => {}
    }
    tracing::info!("stopping");
}
