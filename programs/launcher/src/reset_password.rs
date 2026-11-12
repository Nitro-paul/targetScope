// doc/programs/launcher.md#usage

use std::process::ExitCode;
use std::time::Duration;

use targetscope_common::bus::Bus;
use targetscope_common::messages::database_handler::ResetSettings;

use crate::config::LauncherConfig;
use crate::redis_server;

pub async fn reset_admin_password(config: &LauncherConfig) -> ExitCode {
    let not_running = || {
        eprintln!("TargetScope isn't running");
        ExitCode::FAILURE
    };
    if redis_server::wait_ready(&config.redis_url(), Duration::from_secs(2)).await.is_err() {
        return not_running();
    }
    let Ok(bus) = Bus::connect(&config.redis_url(), &format!("launcher_cli_{}", std::process::id())).await else {
        return not_running();
    };
    let req = ResetSettings { names: Some(vec!["admin_password_hash".into()]), move_images: false };
    match bus.request(req).await {
        Ok(_) => {
            println!("The admin password was removed.");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("TargetScope isn't running ({e})");
            ExitCode::FAILURE
        }
    }
}
