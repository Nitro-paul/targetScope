// doc/programs/statistics.md

pub mod calc;
pub mod grouping;
pub mod handler;
pub mod periods;
pub mod reading;

use std::process::ExitCode;
use std::sync::Arc;

use targetscope_common::messages::statistics::PROGRAM;
use targetscope_common::program;

// doc/shared_libraries.md#redis-bus
pub const CONCURRENT_REQUESTS: usize = 4;

pub async fn run() -> ExitCode {
    let p = program::start(PROGRAM).await;
    program::stop_on_signal(&p.bus);
    let handler = Arc::new(handler::StatisticsHandler { bus: p.bus.clone() });
    match p.bus.serve_concurrent(handler, CONCURRENT_REQUESTS).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "serve loop ended");
            ExitCode::FAILURE
        }
    }
}
