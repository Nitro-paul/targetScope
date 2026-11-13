// doc/testing/shared_libraries.md#clock-unit

// LIB-CLK-01
#[tokio::test]
async fn lib_clk_01_fake_time() {
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};
    use targetscope_common::clock::Clock;
    let t = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let clock = Arc::new(targetscope_testkit::FakeClock::new(t));
    let sleeper = clock.clone();
    let task = tokio::spawn(async move { sleeper.sleep(Duration::from_secs(10)).await });
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(!task.is_finished());
    clock.advance(Duration::from_secs(10));
    tokio::time::timeout(Duration::from_secs(1), task).await.unwrap().unwrap();
    assert_eq!(clock.now(), t + Duration::from_secs(10));
}

// LIB-CLK-02
#[test]
fn lib_clk_02_real_clock() {
    let clock = targetscope_common::clock::SystemClock;
    let a = targetscope_common::clock::Clock::now(&clock);
    std::thread::sleep(std::time::Duration::from_millis(100));
    let b = targetscope_common::clock::Clock::now(&clock);
    let d = b.duration_since(a).unwrap().as_millis() as i64;
    assert!((d - 100).abs() <= 20, "{d}");
}
