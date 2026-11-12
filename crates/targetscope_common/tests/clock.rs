// doc/testing/shared_libraries.md#clock-unit

// LIB-CLK-01
#[test]
#[ignore = "not implemented"]
fn lib_clk_01_fake_time() {
    todo!("LIB-CLK-01")
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
