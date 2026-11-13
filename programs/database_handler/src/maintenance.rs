// doc/programs/database_handler.md#startup

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use targetscope_common::clock::Clock;
use tokio::sync::{mpsc, oneshot};

use crate::db_worker::DbCommand;

pub const PERIOD: Duration = Duration::from_secs(3600);

// doc/interfaces.md#staging-folder
pub fn clean_staging(staging_dir: &Path, max_age_hours: u64, now: SystemTime) {
    let Ok(entries) = fs::read_dir(staging_dir) else { return };
    let max_age = Duration::from_secs(max_age_hours * 3600);
    for e in entries.flatten() {
        let Ok(meta) = e.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let old = meta.modified().ok().and_then(|m| now.duration_since(m).ok()).is_some_and(|age| age > max_age);
        if old {
            match fs::remove_file(e.path()) {
                Ok(()) => tracing::info!(file = %e.path().display(), "old staged file deleted"),
                Err(err) => tracing::warn!(file = %e.path().display(), error = %err, "staged file could not be deleted"),
            }
        }
    }
}

pub async fn run_once(db: &mpsc::Sender<DbCommand>) {
    let (reply, rx) = oneshot::channel();
    if db.send(DbCommand::Maintenance { reply }).await.is_err() {
        return;
    }
    match rx.await {
        Ok(Ok(purged)) => tracing::info!(purged, "maintenance done"),
        Ok(Err(e)) => tracing::warn!(error = %e, "maintenance failed"),
        Err(_) => {}
    }
}

// doc/programs/database_handler.md#startup
pub async fn run_periodically(db: mpsc::Sender<DbCommand>, clock: Arc<dyn Clock>) {
    loop {
        run_once(&db).await;
        clock.sleep(PERIOD).await;
    }
}
