//! One supervisor task per program: waits for its child to exit and reports to the main loop.
//! Backoff doubles from initial_backoff_s up to max_backoff_s; resets after stable_after_s;
//! more than max_restarts within window_s → Failed (doc/programs/launcher.md#supervision).

use std::time::Duration;

use crate::config::RestartPolicy;

#[derive(Debug)]
pub enum SupervisorMsg {
    Exited {
        name: String,
        status: std::process::ExitStatus,
    },
}

/// Next backoff after a restart.
pub fn next_backoff(current: Duration, policy: &RestartPolicy) -> Duration {
    let _ = (current, policy);
    todo!()
}
