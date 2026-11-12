// doc/programs/analyzer.md

pub mod handler;
pub mod scoring;

use std::process::ExitCode;
use std::sync::Arc;

use targetscope_common::messages::analyzer::PROGRAM;
use targetscope_common::program;

// doc/programs/analyzer.md#interface
pub const CONCURRENT_REQUESTS: usize = 16;

pub async fn run() -> ExitCode {
    let p = program::start(PROGRAM).await;
    tokio::select! {
        r = p.bus.serve_concurrent(Arc::new(handler::AnalyzerHandler), CONCURRENT_REQUESTS) => {
            tracing::error!(error = ?r, "serve loop ended");
            ExitCode::FAILURE
        }
        _ = program::shutdown_signal() => ExitCode::SUCCESS,
    }
}
