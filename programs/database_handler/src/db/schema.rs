// doc/programs/database_handler.md#schema-migrations

use std::path::Path;

use rusqlite::{params, Connection};
use targetscope_common::failpoints;

use crate::error::Result;
use crate::known_settings::KNOWN_SETTINGS;

pub const MIGRATIONS: &[&str] = &[include_str!("../../migrations/0001_initial.sql")];

pub fn migrate(c: &mut Connection, migrations: &[&str]) -> Result<()> {
    let version: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    for (i, sql) in migrations.iter().enumerate().skip(version as usize) {
        // doc/testing/overview.md#failpoints
        failpoints::hit("db.migrate");
        let tx = c.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", (i + 1) as i64)?;
        tx.commit()?;
        tracing::info!(version = i + 1, "migration applied");
    }
    Ok(())
}

// doc/database.md#known-settings
pub fn insert_default_settings(c: &Connection, data_dir: &Path) -> Result<()> {
    let mut stmt =
        c.prepare("INSERT INTO settings (name, display_name, active, value) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(name) DO NOTHING")?;
    for k in KNOWN_SETTINGS {
        stmt.execute(params![k.name, k.display_name, k.default_active, k.default_value(data_dir)])?;
    }
    Ok(())
}
