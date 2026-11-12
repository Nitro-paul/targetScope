//! Logging setup (doc/shared_libraries.md#logging): stderr (collected by the launcher) and a
//! daily rotated file `<log_dir>/<program>.log`. Every log line of a request has its request_id.

use crate::config::BootstrapConfig;

/// Keeps the file writer alive; drop it at the end of `main`.
pub struct LogGuard {
    // TODO: tracing_appender::non_blocking::WorkerGuard
}

pub fn init(program: &str, config: &BootstrapConfig) -> LogGuard {
    let _ = (program, config);
    todo!()
}
