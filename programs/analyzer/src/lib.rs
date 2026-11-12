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
    program::stop_on_signal(&p.bus);
    match p.bus.serve_concurrent(Arc::new(handler::AnalyzerHandler), CONCURRENT_REQUESTS).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "serve loop ended");
            ExitCode::FAILURE
        }
    }
}
