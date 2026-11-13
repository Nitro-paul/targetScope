// doc/testing/analyzer.md#performance-performance

mod common;

use std::time::{Duration, Instant};

use analyzer::scoring::analyze;
use common::{shot, tt_ref, Rng, DEV_REF_DIAMETER};

fn timed(max_radius: f64) -> Duration {
    let mut rng = Rng(7);
    let shots: Vec<_> = (0..60).map(|_| shot(rng.next(), rng.next() * max_radius)).collect();
    let start = Instant::now();
    analyze(&shots, &tt_ref(), DEV_REF_DIAMETER);
    start.elapsed()
}

// AN-PRF-01
#[test]
#[ignore = "performance"]
fn perf_an_prf_01_typical_session() {
    let t = timed(15.0);
    assert!(t < Duration::from_millis(200), "{t:?}");
}

// AN-PRF-02
#[test]
#[ignore = "performance"]
fn perf_an_prf_02_worst_case() {
    let t = timed(90.0);
    assert!(t < Duration::from_secs(1), "{t:?}");
}
