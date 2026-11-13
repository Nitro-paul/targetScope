// doc/programs/statistics.md#calculation-flow-compute_statistics
// doc/shared_libraries.md#local-time

use targetscope_common::clock::{local, local_midnight, LocalDate};
use targetscope_common::messages::statistics::Period;

pub fn period_start(saved_at: i64, period: Period) -> i64 {
    let l = local(saved_at);
    match period {
        Period::Session => saved_at,
        Period::Day => local_midnight(l.date),
        Period::Week => local_midnight(add_days(l.date, -(l.weekday_from_monday as i64))),
        Period::Month => local_midnight(LocalDate { day: 1, ..l.date }),
    }
}

fn add_days(d: LocalDate, days: i64) -> LocalDate {
    civil_from_days(days_from_civil(d) + days)
}

fn days_from_civil(d: LocalDate) -> i64 {
    let (m, y) = (d.month as i64, d.year as i64 - (d.month <= 2) as i64);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d.day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> LocalDate {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = (yoe + era * 400 + (month <= 2) as i64) as i32;
    LocalDate { year, month, day }
}
