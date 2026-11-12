//! session_manager (doc/programs/session_manager.md).

pub mod actor;
pub mod analysis;
pub mod events;
pub mod persistence;
pub mod saving;
pub mod state;
pub mod transitions;

use std::process::ExitCode;

/// Startup: logging, bus, settings cache, recovery from `session_manager:current`,
/// event readers (image_processor:events with consumer group, database_handler:events plain),
/// then serve requests.
pub async fn run() -> ExitCode {
    todo!()
}
