// doc/programs/database_handler.md#database-and-queries

pub mod devices;
pub mod processed_requests;
pub mod schema;
pub mod sessions;
pub mod settings;
pub mod shooters;
pub mod target_types;

use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension};

use crate::error::Result;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    // doc/programs/database_handler.md#startup
    pub fn open(path: &Path, data_dir: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut conn = Connection::open(path)?;
        conn.busy_timeout(Duration::from_millis(5000))?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        schema::migrate(&mut conn, schema::MIGRATIONS)?;
        schema::insert_default_settings(&conn, data_dir)?;
        Ok(Database { conn })
    }

    pub fn open_in_memory(data_dir: &Path) -> Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        schema::migrate(&mut conn, schema::MIGRATIONS)?;
        schema::insert_default_settings(&conn, data_dir)?;
        Ok(Database { conn })
    }
}

pub fn exists(c: &Connection, table: &str, id: i64) -> Result<bool> {
    Ok(c.query_row(&format!("SELECT 1 FROM {table} WHERE id = ?1"), [id], |_| Ok(())).optional()?.is_some())
}

pub fn used_by_session(c: &Connection, column: &str, id: i64) -> Result<bool> {
    Ok(c.query_row(&format!("SELECT 1 FROM sessions WHERE {column} = ?1 LIMIT 1"), [id], |_| Ok(())).optional()?.is_some())
}

pub fn uuid_text(u: &uuid::Uuid) -> String {
    u.hyphenated().to_string()
}

pub fn parse_uuid(s: String) -> rusqlite::Result<uuid::Uuid> {
    uuid::Uuid::parse_str(&s).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))
}
