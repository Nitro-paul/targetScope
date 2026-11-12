// doc/programs/exporter.md

pub mod collect;
pub mod files;
pub mod formats;
pub mod handler;

use std::process::ExitCode;
use std::sync::Arc;

use targetscope_common::messages::exporter::PROGRAM;
use targetscope_common::program;
use targetscope_common::settings::SettingsCache;

// doc/shared_libraries.md#redis-bus
pub const CONCURRENT_REQUESTS: usize = 2;

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
    if let Some(dir) = settings.get_path("export_dir") {
        files::remove_part_files(&dir);
    }
    program::stop_on_signal(&p.bus);
    let handler = Arc::new(handler::ExporterHandler { bus: p.bus.clone(), settings });
    match p.bus.serve_concurrent(handler, CONCURRENT_REQUESTS).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "serve loop ended");
            ExitCode::FAILURE
        }
    }
}
