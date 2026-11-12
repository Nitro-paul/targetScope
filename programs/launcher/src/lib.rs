//! Launcher library (doc/programs/launcher.md). `main.rs` only calls [`run`].

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

/// Parses the command line and runs the launcher or `--reset-admin-password`.
pub async fn run(args: Vec<String>) -> ExitCode {
    let _ = args;
    todo!()
}
