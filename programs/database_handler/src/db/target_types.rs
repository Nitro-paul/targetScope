// doc/database.md#target_types

use rusqlite::{params, Connection, OptionalExtension, Row};
use targetscope_common::model::{Filter, TargetType};
use targetscope_common::targets::generate_rings;
use uuid::Uuid;

use super::shooters::insert_order;
use super::{parse_uuid, used_by_session, uuid_text};
use crate::error::{HandlerError, Result};
use crate::filter_sql::{filter_to_sql, TARGET_TYPE_COLUMNS};

const SELECT: &str = "SELECT id, uuid, name, paper_width_mm, paper_height_mm, center_x_mm, center_y_mm, \
    inner_white_radius_mm, inner_white_score, black_radius_mm, black_score, outer_radius_mm, outer_score, \
    EXISTS (SELECT 1 FROM sessions WHERE target_type_id = target_types.id), retired FROM target_types";

fn row(r: &Row) -> rusqlite::Result<TargetType> {
    let mut t = TargetType {
        id: Some(r.get(0)?),
        uuid: Some(parse_uuid(r.get(1)?)?),
        name: r.get(2)?,
        paper_width_mm: r.get(3)?,
        paper_height_mm: r.get(4)?,
        center_x_mm: r.get(5)?,
        center_y_mm: r.get(6)?,
        inner_white_radius_mm: r.get(7)?,
        inner_white_score: r.get(8)?,
        black_radius_mm: r.get(9)?,
        black_score: r.get(10)?,
        outer_radius_mm: r.get(11)?,
        outer_score: r.get(12)?,
        rings: vec![],
        locked: r.get(13)?,
        retired: r.get(14)?,
    };
    t.rings = generate_rings(&t);
    Ok(t)
}

pub fn get(c: &Connection, id: i64) -> Result<Option<TargetType>> {
    Ok(c.query_row(&format!("{SELECT} WHERE id = ?1"), [id], row).optional()?)
}

fn name_taken(c: &Connection, name: &str, except: Option<i64>) -> Result<bool> {
    Ok(c
        .query_row("SELECT 1 FROM target_types WHERE name = ?1 COLLATE NOCASE AND id IS NOT ?2", params![name, except], |_| Ok(()))
        .optional()?
        .is_some())
}

fn same_geometry(a: &TargetType, b: &TargetType) -> bool {
    a.paper_width_mm == b.paper_width_mm
        && a.paper_height_mm == b.paper_height_mm
        && a.center_x_mm == b.center_x_mm
        && a.center_y_mm == b.center_y_mm
        && a.inner_white_radius_mm == b.inner_white_radius_mm
        && a.inner_white_score == b.inner_white_score
        && a.black_radius_mm == b.black_radius_mm
        && a.black_score == b.black_score
        && a.outer_radius_mm == b.outer_radius_mm
        && a.outer_score == b.outer_score
}

fn insert(c: &Connection, uuid: &Uuid, name: &str, t: &TargetType) -> Result<i64> {
    c.execute(
        "INSERT INTO target_types (uuid, name, paper_width_mm, paper_height_mm, center_x_mm, center_y_mm, \
         inner_white_radius_mm, inner_white_score, black_radius_mm, black_score, outer_radius_mm, outer_score, retired) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            uuid_text(uuid),
            name,
            t.paper_width_mm,
            t.paper_height_mm,
            t.center_x_mm,
            t.center_y_mm,
            t.inner_white_radius_mm,
            t.inner_white_score,
            t.black_radius_mm,
            t.black_score,
            t.outer_radius_mm,
            t.outer_score,
            t.retired
        ],
    )?;
    Ok(c.last_insert_rowid())
}

// doc/interfaces.md#save_target_type
pub fn save(c: &Connection, t: &TargetType) -> Result<(i64, bool)> {
    let name = t.name.trim();
    if name_taken(c, name, t.id)? {
        return Err(HandlerError::validation(format!("a target type named {name} exists already")));
    }
    let Some(id) = t.id else {
        return Ok((insert(c, &Uuid::new_v4(), name, t)?, true));
    };
    let current = get(c, id)?.ok_or_else(|| HandlerError::not_found(format!("target type {id}")))?;
    if current.locked {
        if !same_geometry(&current, t) {
            return Err(HandlerError::InUse("target type is used by a session".into()));
        }
        c.execute("UPDATE target_types SET name = ?1, retired = ?2 WHERE id = ?3", params![name, t.retired, id])?;
    } else {
        c.execute(
            "UPDATE target_types SET name = ?1, paper_width_mm = ?2, paper_height_mm = ?3, center_x_mm = ?4, \
             center_y_mm = ?5, inner_white_radius_mm = ?6, inner_white_score = ?7, black_radius_mm = ?8, \
             black_score = ?9, outer_radius_mm = ?10, outer_score = ?11, retired = ?12 WHERE id = ?13",
            params![
                name,
                t.paper_width_mm,
                t.paper_height_mm,
                t.center_x_mm,
                t.center_y_mm,
                t.inner_white_radius_mm,
                t.inner_white_score,
                t.black_radius_mm,
                t.black_score,
                t.outer_radius_mm,
                t.outer_score,
                t.retired,
                id
            ],
        )?;
    }
    Ok((id, false))
}

// doc/interfaces.md#import_target_type
pub fn import(c: &Connection, t: &TargetType) -> Result<(i64, bool)> {
    let uuid = t.uuid.ok_or_else(|| HandlerError::validation("uuid is required"))?;
    if let Some(id) = c.query_row("SELECT id FROM target_types WHERE uuid = ?1", [uuid_text(&uuid)], |r| r.get(0)).optional()? {
        return Ok((id, false));
    }
    let base = t.name.trim().to_string();
    let mut name = base.clone();
    let mut n = 1;
    while name_taken(c, &name, None)? {
        name = if n == 1 { format!("{base} (imported)") } else { format!("{base} (imported {n})") };
        n += 1;
    }
    Ok((insert(c, &uuid, &name, t)?, true))
}

pub fn find(c: &Connection, f: &Filter) -> Result<Vec<TargetType>> {
    let sql = filter_to_sql(f, TARGET_TYPE_COLUMNS)?;
    let order = if f.order_by.is_none() { " ORDER BY name COLLATE NOCASE" } else { "" };
    let mut stmt = c.prepare(&format!("{SELECT}{}", insert_order(&sql.clause, order)))?;
    let rows = stmt.query_map(rusqlite::params_from_iter(sql.params), row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

// doc/interfaces.md#delete_target_type
pub fn delete(c: &Connection, id: i64) -> Result<()> {
    if !super::exists(c, "target_types", id)? {
        return Err(HandlerError::not_found(format!("target type {id}")));
    }
    if used_by_session(c, "target_type_id", id)? {
        return Err(HandlerError::InUse(format!("target type {id} is used by a session")));
    }
    c.execute("DELETE FROM target_types WHERE id = ?1", [id])?;
    Ok(())
}
