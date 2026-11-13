// doc/testing/statistics.md#periods-unit

use statistics::periods::period_start;
use targetscope_common::clock::{local, local_midnight, LocalDate};
use targetscope_common::messages::statistics::Period;

extern "C" {
    fn tzset();
}

fn berlin() {
    std::env::set_var("TZ", "Europe/Berlin");
    unsafe { tzset() };
}

fn at(year: i32, month: u32, day: u32, hour: i64, minute: i64) -> i64 {
    berlin();
    let midnight = local_midnight(LocalDate { year, month, day });
    let t = midnight + hour * 3600 + minute * 60;
    let shift = (local(midnight).utc_offset_s - local(t).utc_offset_s) as i64;
    t + shift
}

fn same(a: i64, b: i64, p: Period) -> bool {
    period_start(a, p) == period_start(b, p)
}

// ST-PER-01
#[test]
fn st_per_01_one_point_per_session() {
    let t = at(2026, 3, 2, 10, 0);
    assert_eq!(period_start(t, Period::Session), t);
}

// ST-PER-02
#[test]
fn st_per_02_days() {
    assert!(!same(at(2026, 3, 2, 23, 30), at(2026, 3, 3, 0, 30), Period::Day));
}

// ST-PER-03
#[test]
fn st_per_03_weeks_start_on_monday() {
    assert!(same(at(2026, 3, 2, 12, 0), at(2026, 3, 8, 12, 0), Period::Week));
    assert!(!same(at(2026, 3, 8, 12, 0), at(2026, 3, 9, 12, 0), Period::Week));
}

// ST-PER-04
#[test]
fn st_per_04_week_across_the_year() {
    let a = at(2026, 12, 31, 12, 0);
    assert!(same(a, at(2027, 1, 2, 12, 0), Period::Week));
    assert_eq!(local(period_start(a, Period::Week)).date, LocalDate { year: 2026, month: 12, day: 28 });
}

// ST-PER-05
#[test]
fn st_per_05_daylight_saving_time() {
    let (a, b, c) = (at(2026, 3, 29, 1, 30), at(2026, 3, 29, 3, 30), at(2026, 3, 30, 0, 30));
    assert_eq!(b - a, 3600);
    assert!(same(a, b, Period::Day));
    assert!(!same(b, c, Period::Day));
}

// ST-PER-06
#[test]
fn st_per_06_months() {
    assert!(!same(at(2026, 1, 31, 12, 0), at(2026, 2, 1, 12, 0), Period::Month));
}

// ST-PER-07
#[test]
fn st_per_07_empty_periods() {
    let starts: std::collections::BTreeSet<i64> =
        [at(2026, 1, 15, 12, 0), at(2026, 3, 15, 12, 0)].iter().map(|t| period_start(*t, Period::Month)).collect();
    assert_eq!(starts.len(), 2);
    assert!(starts.iter().all(|s| local(*s).date.month != 2));
}
