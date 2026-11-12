//! `--reset-admin-password`: connects to the running TargetScope's Redis, sends reset_settings
//! for admin_password_hash and exits 0; exits 1 if TargetScope isn't running.

use std::process::ExitCode;

use crate::config::LauncherConfig;

pub async fn reset_admin_password(config: &LauncherConfig) -> ExitCode {
    let _ = config;
    todo!()
}
