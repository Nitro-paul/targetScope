//! Database: connection and pragmas. All query functions take a `&Connection`
//! (a Transaction derefs to it), so the DbWorker controls the transaction.

pub mod devices;
pub mod processed_requests;
pub mod schema;
pub mod sessions;
pub mod settings;
pub mod shooters;
pub mod target_types;

use std::path::Path;

use crate::error::HandlerError;

pub struct Database {
    pub conn: rusqlite::Connection,
}

impl Database {
    /// Creates file and parent directory if missing; PRAGMA foreign_keys = ON, journal_mode = WAL,
    /// busy_timeout = 5000; runs migrations; inserts default settings.
    pub fn open(path: &Path) -> Result<Self, HandlerError> {
        let _ = path;
        todo!()
    }

    pub fn transaction(&mut self) -> Result<rusqlite::Transaction<'_>, HandlerError> {
        Ok(self.conn.transaction()?)
    }
}
