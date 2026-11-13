// doc/programs/launcher.md#supervision

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::config::RestartPolicy;

pub fn next_backoff(current: Duration, policy: &RestartPolicy) -> Duration {
    (current * 2).min(Duration::from_secs(policy.max_backoff_s))
}

pub fn initial_backoff(policy: &RestartPolicy) -> Duration {
    Duration::from_secs(policy.initial_backoff_s)
}

// doc/programs/launcher.md#supervision
pub fn too_many_restarts(restarts: &mut VecDeque<Instant>, now: Instant, policy: &RestartPolicy) -> bool {
    let window = Duration::from_secs(policy.window_s);
    restarts.push_back(now);
    while restarts.front().is_some_and(|t| now.duration_since(*t) > window) {
        restarts.pop_front();
    }
    restarts.len() > policy.max_restarts as usize
}
