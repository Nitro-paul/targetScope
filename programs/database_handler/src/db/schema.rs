//! Migrations (doc/programs/database_handler.md#schema-migrations) and default settings.
//! `PRAGMA user_version` = number of applied migrations; each migration runs in its own transaction.

use rusqlite::Connection;

use crate::error::HandlerError;

pub const MIGRATIONS: &[&str] = &[include_str!("../../migrations/0001_initial.sql")];

pub fn migrate(c: &mut Connection) -> Result<(), HandlerError> {
    let _ = c;
    todo!()
}

/// Inserts every known setting that doesn't exist yet with its default value and active state.
pub fn insert_default_settings(
    c: &Connection,
    data_dir: &std::path::Path,
) -> Result<(), HandlerError> {
    let _ = (c, data_dir);
    todo!()
}
