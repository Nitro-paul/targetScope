// doc/database.md#settings

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};
use targetscope_common::model::{Filter, Setting};

use super::shooters::insert_order;
use crate::error::{HandlerError, Result};
use crate::filter_sql::{filter_to_sql, SETTING_COLUMNS};
use crate::known_settings::{self, complete};

fn row(r: &Row) -> rusqlite::Result<Setting> {
    Ok(Setting {
        name: r.get(0)?,
        display_name: r.get(1)?,
        active: r.get(2)?,
        value: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
        optional: false,
        default_value: String::new(),
        default_active: true,
        value_rule: String::new(),
    })
}

// doc/interfaces.md#save_setting
pub fn save(c: &Connection, s: &Setting) -> Result<bool> {
    let created = get_setting(c, &s.name)?.is_none();
    c.execute(
        "INSERT INTO settings (name, display_name, active, value) VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(name) DO UPDATE SET display_name = excluded.display_name, active = excluded.active, value = excluded.value",
        params![s.name, s.display_name, s.active, s.value],
    )?;
    Ok(created)
}

// doc/interfaces.md#reset_settings
pub fn reset(c: &Connection, names: &[&str], data_dir: &Path) -> Result<()> {
    for name in names {
        let k = known_settings::find(name).ok_or_else(|| HandlerError::validation(format!("unknown setting {name}")))?;
        c.execute(
            "UPDATE settings SET active = ?1, value = ?2 WHERE name = ?3",
            params![k.default_active, k.default_value(data_dir), name],
        )?;
    }
    Ok(())
}

pub fn find(c: &Connection, f: &Filter, data_dir: &Path) -> Result<Vec<Setting>> {
    let sql = filter_to_sql(f, SETTING_COLUMNS)?;
    let order = if f.order_by.is_none() { " ORDER BY name" } else { "" };
    let mut stmt = c.prepare(&format!("SELECT name, display_name, active, value FROM settings{}", insert_order(&sql.clause, order)))?;
    let rows = stmt.query_map(rusqlite::params_from_iter(sql.params), row)?;
    Ok(rows.map(|r| r.map(|s| complete(s, data_dir))).collect::<rusqlite::Result<_>>()?)
}

pub fn get_setting(c: &Connection, name: &str) -> Result<Option<Setting>> {
    Ok(c.query_row("SELECT name, display_name, active, value FROM settings WHERE name = ?1", [name], row).optional()?)
}

pub fn get(c: &Connection, name: &str) -> Result<Option<String>> {
    Ok(get_setting(c, name)?.filter(|s| s.active).map(|s| s.value))
}

// doc/interfaces.md#delete_setting
pub fn delete(c: &Connection, name: &str) -> Result<()> {
    if known_settings::find(name).is_some() {
        return Err(HandlerError::validation(format!("{name} is a known setting and can't be deleted")));
    }
    if c.execute("DELETE FROM settings WHERE name = ?1", [name])? == 0 {
        return Err(HandlerError::not_found(format!("setting {name}")));
    }
    Ok(())
}
