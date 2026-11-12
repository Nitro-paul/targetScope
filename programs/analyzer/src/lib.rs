//! analyzer (doc/programs/analyzer.md). The calculation is plain Rust in `scoring`, testable
//! without Redis; `handler` serves `analyze_shots`.

pub mod handler;
pub mod scoring;

use std::process::ExitCode;

pub async fn run() -> ExitCode {
    todo!()
}
