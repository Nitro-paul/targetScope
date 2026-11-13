// doc/programs/database_handler.md

pub mod db;
pub mod db_worker;
pub mod error;
pub mod filter_sql;
pub mod image_dir_change;
pub mod images;
pub mod known_settings;
pub mod maintenance;
pub mod stream_worker;
pub mod validation;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use serde_json::json;
use targetscope_common::bus::Bus;
use targetscope_common::clock::{Clock, SystemClock};
use targetscope_common::config::{expand_home, BootstrapConfig};
use targetscope_common::messages::database_handler::{ChangeAction, DataChanged, DatabaseHandlerEvent, Table, PROGRAM};
use targetscope_common::{failpoints, logging, program};

use crate::db::Database;
use crate::images::ImageStore;

// doc/shared_libraries.md#redis-bus
pub const CONCURRENT_REQUESTS: usize = 8;

// doc/programs/database_handler.md#startup
pub async fn run() -> ExitCode {
    failpoints::setup();
    let config = BootstrapConfig::from_env();
    let _log = logging::init(PROGRAM, &config);
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let db = match Database::open(&config.db_path, &config.data_dir) {
        Ok(db) => db,
        Err(e) => {
            tracing::error!(error = %e, path = %config.db_path.display(), "database could not be opened");
            return ExitCode::FAILURE;
        }
    };
    let folder = |name: &str, default: PathBuf| {
        db::settings::get(&db.conn, name).ok().flatten().map(|v| expand_home(&v)).unwrap_or(default)
    };
    let images = ImageStore {
        image_dir: folder("image_dir", config.data_dir.join("images")),
        staging_dir: folder("staging_dir", config.data_dir.join("staging")),
    };
    for dir in [images.image_dir.join("devices"), images.image_dir.join("sessions"), images.staging_dir.clone()] {
        if let Err(e) = std::fs::create_dir_all(&dir) {
            tracing::error!(error = %e, dir = %dir.display(), "folder could not be created");
            return ExitCode::FAILURE;
        }
    }
    let db_tx = db_worker::spawn_db_worker(db, images, config.data_dir.clone(), clock.clone());
    let bus = program::connect(&config.redis_url, PROGRAM).await;
    send_reload(&bus).await;
    tokio::spawn(reload_on_reconnect(bus.clone()));
    tokio::spawn(maintenance::run_periodically(db_tx.clone(), clock));
    program::stop_on_signal(&bus);
    let worker = Arc::new(stream_worker::StreamWorker {
        bus: bus.clone(),
        db: db_tx,
        data_dir: config.data_dir.clone(),
        changing_image_dir: AtomicBool::new(false),
    });
    tracing::info!("started");
    match bus.serve_concurrent(worker, CONCURRENT_REQUESTS).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "serve loop ended");
            ExitCode::FAILURE
        }
    }
}

// doc/programs/database_handler.md#error-handling
const RELOAD_ATTEMPTS: u32 = 10;

// doc/interfaces.md#data_changed
// doc/programs/database_handler.md#error-handling
async fn send_reload(bus: &Bus) {
    let event = DatabaseHandlerEvent::DataChanged(DataChanged {
        table: Table::All,
        key: json!(null),
        action: ChangeAction::Reload,
    });
    let mut delay = std::time::Duration::from_millis(100);
    for attempt in 1..=RELOAD_ATTEMPTS {
        match bus.publish(&event).await {
            Ok(_) => return,
            Err(e) if attempt == RELOAD_ATTEMPTS => tracing::warn!(error = %e, "reload event not sent"),
            Err(_) => {
                tokio::time::sleep(delay).await;
                delay = (delay * 2).min(std::time::Duration::from_secs(2));
            }
        }
    }
}

// doc/programs/database_handler.md#error-handling
async fn reload_on_reconnect(bus: Bus) {
    let mut reconnects = bus.reconnects();
    while reconnects.changed().await.is_ok() {
        send_reload(&bus).await;
    }
}
