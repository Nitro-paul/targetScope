// doc/programs/importer.md

pub mod archive;
pub mod consistency;
pub mod export_file;
pub mod handler;
pub mod records;

use std::process::ExitCode;

use targetscope_common::messages::importer::PROGRAM;
use targetscope_common::program;
use targetscope_common::settings::SettingsCache;

pub async fn run() -> ExitCode {
    let p = program::start(PROGRAM).await;
    let settings = loop {
        match SettingsCache::load(&p.bus).await {
            Ok(s) => break s,
            Err(e) => {
                tracing::warn!(error = %e, "settings not available yet");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    };
    program::stop_on_signal(&p.bus);
    match p.bus.serve(handler::ImporterHandler { bus: p.bus.clone(), settings }).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "serve loop ended");
            ExitCode::FAILURE
        }
    }
}
