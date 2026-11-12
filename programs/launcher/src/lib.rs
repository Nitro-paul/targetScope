// doc/programs/launcher.md

pub mod cli;
pub mod config;
pub mod env;
pub mod launcher;
pub mod lock;
pub mod logs;
pub mod order;
pub mod program;
pub mod redis_server;
pub mod requests;
pub mod reset_password;
pub mod supervisor;

use std::process::ExitCode;

use targetscope_common::bus::Bus;
use targetscope_common::config::BootstrapConfig;
use targetscope_common::messages::launcher::PROGRAM;
use targetscope_common::program as common_program;
use targetscope_common::{failpoints, frames, logging};
use tokio::sync::mpsc;

use crate::launcher::{watch_redis, Event, Launcher};

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("targetscope: {msg}");
    tracing::error!("{msg}");
    ExitCode::FAILURE
}

// doc/programs/launcher.md#startup-sequence
pub async fn run(args: Vec<String>) -> ExitCode {
    failpoints::setup();
    let command = match cli::parse(&args) {
        Ok(c) => c,
        Err(e) => return fail(e),
    };
    let (config_path, reset) = match command {
        cli::Command::Run { config } => (config, false),
        cli::Command::ResetAdminPassword { config } => (config, true),
    };
    let config = match config::load(&config_path.unwrap_or_else(config::default_path)) {
        Ok(c) => c,
        Err(e) => return fail(e),
    };
    if reset {
        return reset_password::reset_admin_password(&config).await;
    }
    let _lock = match lock::acquire() {
        Ok(l) => l,
        Err(e) => return fail(e),
    };
    for dir in [&config.paths.data_dir, &config.paths.log_dir] {
        if let Err(e) = std::fs::create_dir_all(dir) {
            return fail(format!("{}: {e}", dir.display()));
        }
    }
    let bootstrap = BootstrapConfig::from_vars(|name| {
        env::program_env(&config).into_iter().find(|(k, _)| k == name).map(|(_, v)| v)
    });
    let _log = logging::init(PROGRAM, &bootstrap);
    if let Err(e) = frames::cleanup_dead_nodes() {
        tracing::warn!(error = %e, "stale shared memory could not be removed");
    }
    if redis_server::port_in_use(config.redis.port) {
        return fail(format!("port {} is already in use; set another port in [redis]", config.redis.port));
    }
    let child = match redis_server::spawn(&config) {
        Ok(c) => c,
        Err(e) => return fail(e),
    };
    let (tx, rx) = mpsc::channel(256);
    let redis_pid = child.id();
    if let Err(e) = redis_server::wait_ready(&config.redis_url(), redis_server::READY_TIMEOUT).await {
        if let Some(pid) = redis_pid {
            let _ = nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid as i32), nix::sys::signal::Signal::SIGKILL);
        }
        return fail(e);
    }
    let redis_pid = watch_redis(child, tx.clone());
    let bus = match Bus::connect(&config.redis_url(), PROGRAM).await {
        Ok(b) => b,
        Err(e) => return fail(e),
    };
    let serving = bus.clone();
    let handler = requests::LauncherRequestHandler { tx: tx.clone() };
    tokio::spawn(async move {
        let _ = serving.serve(handler).await;
    });
    let signals = tx.clone();
    tokio::spawn(async move {
        common_program::shutdown_signal().await;
        let _ = signals.send(Event::Shutdown).await;
    });
    match Launcher::new(config, bus, tx, redis_pid) {
        Ok(l) => l.run(rx).await,
        Err(e) => return fail(e),
    }
    ExitCode::SUCCESS
}
