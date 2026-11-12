//! statistics (doc/programs/statistics.md). Calculations are plain Rust in `calc`, testable
//! without Redis. Nothing is cached: every request reads the selected sessions again.

pub mod calc;
pub mod grouping;
pub mod handler;
pub mod periods;
pub mod reading;

use std::process::ExitCode;

pub async fn run() -> ExitCode {
    todo!()
}
