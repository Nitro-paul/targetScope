//! Maintenance at startup and once an hour: purge processed_requests older than
//! processed_requests_retention_days; delete staging files (incl. .part) older than staging_max_age_hours.

pub async fn run_periodically() {
    todo!()
}
