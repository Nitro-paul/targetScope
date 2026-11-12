//! Command line (doc/programs/launcher.md#usage).

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Start everything with the given (or default) configuration file.
    Run { config: Option<PathBuf> },
    /// Send reset_settings for admin_password_hash to the running TargetScope and exit.
    ResetAdminPassword { config: Option<PathBuf> },
}

pub fn parse(args: &[String]) -> Result<Command, String> {
    let _ = args;
    todo!()
}
