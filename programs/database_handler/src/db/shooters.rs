// doc/database.md#shooters

use rusqlite::{params, Connection, OptionalExtension, Row};
use targetscope_common::model::{Filter, Shooter};
use uuid::Uuid;

use super::{parse_uuid, used_by_session, uuid_text};
use crate::error::{HandlerError, Result};
use crate::filter_sql::{filter_to_sql, SHOOTER_COLUMNS};

fn row(r: &Row) -> rusqlite::Result<Shooter> {
    Ok(Shooter { id: Some(r.get(0)?), uuid: Some(parse_uuid(r.get(1)?)?), name: r.get(2)?, retired: r.get(3)? })
}

fn name_taken(c: &Connection, name: &str, except: Option<i64>) -> Result<Option<i64>> {
    Ok(c.query_row("SELECT id FROM shooters WHERE name = ?1 COLLATE NOCASE AND id IS NOT ?2", params![name, except], |r| r.get(0))
        .optional()?)
}

// doc/interfaces.md#save_shooter
pub fn save(c: &Connection, s: &Shooter) -> Result<(i64, bool)> {
    let name = s.name.trim();
    if name_taken(c, name, s.id)?.is_some() {
        return Err(HandlerError::validation(format!("a shooter named {name} exists already")));
    }
    match s.id {
        None => {
            c.execute(
                "INSERT INTO shooters (uuid, name, retired) VALUES (?1, ?2, ?3)",
                params![uuid_text(&Uuid::new_v4()), name, s.retired],
            )?;
            Ok((c.last_insert_rowid(), true))
        }
        Some(id) => {
            let n = c.execute("UPDATE shooters SET name = ?1, retired = ?2 WHERE id = ?3", params![name, s.retired, id])?;
            if n == 0 {
                return Err(HandlerError::not_found(format!("shooter {id}")));
            }
            Ok((id, false))
        }
    }
}

// doc/interfaces.md#import_shooter
pub fn import(c: &Connection, s: &Shooter) -> Result<(i64, bool)> {
    let uuid = s.uuid.ok_or_else(|| HandlerError::validation("uuid is required"))?;
    if let Some(id) = c.query_row("SELECT id FROM shooters WHERE uuid = ?1", [uuid_text(&uuid)], |r| r.get(0)).optional()? {
        return Ok((id, false));
    }
    let name = s.name.trim();
    if let Some(id) = name_taken(c, name, None)? {
        return Ok((id, false));
    }
    c.execute("INSERT INTO shooters (uuid, name, retired) VALUES (?1, ?2, ?3)", params![uuid_text(&uuid), name, s.retired])?;
    Ok((c.last_insert_rowid(), true))
}

pub fn find(c: &Connection, f: &Filter) -> Result<Vec<Shooter>> {
    let sql = filter_to_sql(f, SHOOTER_COLUMNS)?;
    let order = if f.order_by.is_none() { " ORDER BY name COLLATE NOCASE" } else { "" };
    let clause = insert_order(&sql.clause, order);
    let mut stmt = c.prepare(&format!("SELECT id, uuid, name, retired FROM shooters{clause}"))?;
    let rows = stmt.query_map(rusqlite::params_from_iter(sql.params), row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

// doc/interfaces.md#delete_shooter
pub fn delete(c: &Connection, id: i64) -> Result<()> {
    if !super::exists(c, "shooters", id)? {
        return Err(HandlerError::not_found(format!("shooter {id}")));
    }
    if used_by_session(c, "shooter_id", id)? {
        return Err(HandlerError::InUse(format!("shooter {id} is used by a session")));
    }
    c.execute("DELETE FROM shooters WHERE id = ?1", [id])?;
    Ok(())
}

pub fn insert_order(clause: &str, order: &str) -> String {
    if order.is_empty() {
        return clause.to_string();
    }
    match clause.find(" LIMIT ") {
        Some(i) => format!("{}{}{}", &clause[..i], order, &clause[i..]),
        None => format!("{clause}{order}"),
    }
}
