// doc/shared_libraries.md#clock-and-failpoints

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;

#[async_trait]
pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
    fn monotonic(&self) -> Instant;
    async fn sleep(&self, d: Duration);
}

pub struct SystemClock;

#[async_trait]
impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }

    fn monotonic(&self) -> Instant {
        Instant::now()
    }

    async fn sleep(&self, d: Duration) {
        tokio::time::sleep(d).await
    }
}

pub fn unix_s(t: SystemTime) -> i64 {
    unix_ms(t).div_euclid(1000)
}

pub fn unix_ms(t: SystemTime) -> i64 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as i64,
        Err(e) => -(e.duration().as_millis() as i64),
    }
}

// doc/shared_libraries.md#local-time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl std::fmt::Display for LocalDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

// doc/shared_libraries.md#local-time
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalDateTime {
    pub date: LocalDate,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub weekday_from_monday: u32,
    pub utc_offset_s: i64,
}

// doc/shared_libraries.md#local-time
pub fn local(unix_s: i64) -> LocalDateTime {
    let t = unix_s as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe { libc::localtime_r(&t, &mut tm) };
    LocalDateTime {
        date: LocalDate { year: tm.tm_year + 1900, month: (tm.tm_mon + 1) as u32, day: tm.tm_mday as u32 },
        hour: tm.tm_hour as u32,
        minute: tm.tm_min as u32,
        second: tm.tm_sec as u32,
        weekday_from_monday: ((tm.tm_wday + 6) % 7) as u32,
        utc_offset_s: tm.tm_gmtoff,
    }
}

// doc/shared_libraries.md#local-time
pub fn local_midnight(date: LocalDate) -> i64 {
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    tm.tm_year = date.year - 1900;
    tm.tm_mon = date.month as i32 - 1;
    tm.tm_mday = date.day as i32;
    tm.tm_isdst = -1;
    unsafe { libc::mktime(&mut tm) as i64 }
}

// doc/shared_libraries.md#local-time
pub fn rfc3339_local(unix_s: i64) -> String {
    let l = local(unix_s);
    let off = l.utc_offset_s;
    let sign = if off < 0 { '-' } else { '+' };
    let off = off.abs();
    format!(
        "{}T{:02}:{:02}:{:02}{}{:02}:{:02}",
        l.date,
        l.hour,
        l.minute,
        l.second,
        sign,
        off / 3600,
        (off % 3600) / 60
    )
}
