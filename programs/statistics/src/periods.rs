//! Trend periods: one point per session, or calendar day/week/month in local time,
//! weeks starting on Monday.

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Period {
    Session,
    Day,
    Week,
    Month,
}

/// Unix seconds of the start of the period containing `saved_at`.
pub fn period_start(saved_at: i64, period: Period) -> i64 {
    let _ = (saved_at, period);
    todo!()
}
