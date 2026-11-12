//! `targetscope`: the launcher. Started by hand; starts Redis and all programs.
//!
//! ```text
//! targetscope                       # = launcher with ~/.config/targetscope/launcher.toml
//! targetscope --config <file>
//! targetscope --reset-admin-password
//! ```

#[tokio::main]
async fn main() -> std::process::ExitCode {
    launcher::run(std::env::args().collect()).await
}
