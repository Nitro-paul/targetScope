//! database_handler (doc/programs/database_handler.md): the only program with access to the
//! database and write access to the image folder; protects records.
//!
//! Startup (doc/programs/database_handler.md#startup):
//! 1. logging and bootstrap configuration, 2. `Database::open` (pragmas, migrations, default
//! settings), 3. image and staging folders, 4. DbWorker thread, 5. Redis consumer group,
//! `data_changed all/reload`, StreamWorker, 6. maintenance hourly, 7. exit on SIGTERM/SIGINT.

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

use std::process::ExitCode;

pub async fn run() -> ExitCode {
    todo!()
}
