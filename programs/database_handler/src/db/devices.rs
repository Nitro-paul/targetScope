// doc/database.md#devices

use rusqlite::{params, Connection, OptionalExtension, Row};
use targetscope_common::model::{Device, Filter};
use uuid::Uuid;

use super::shooters::insert_order;
use super::{parse_uuid, used_by_session, uuid_text};
use crate::error::{HandlerError, Result};
use crate::filter_sql::{filter_to_sql, DEVICE_COLUMNS};

const SELECT: &str = "SELECT id, uuid, name, category, description, caliber, bullet_diameter_mm, image_path, shot_count, \
    EXISTS (SELECT 1 FROM sessions WHERE device_id = devices.id), retired FROM devices";

fn row(r: &Row) -> rusqlite::Result<Device> {
    Ok(Device {
        id: Some(r.get(0)?),
        uuid: Some(parse_uuid(r.get(1)?)?),
        name: r.get(2)?,
        category: r.get(3)?,
        description: r.get(4)?,
        caliber: r.get(5)?,
        bullet_diameter_mm: r.get(6)?,
        image_path: r.get(7)?,
        shot_count: r.get(8)?,
        locked: r.get(9)?,
        retired: r.get(10)?,
    })
}

pub fn get(c: &Connection, id: i64) -> Result<Option<Device>> {
    Ok(c.query_row(&format!("{SELECT} WHERE id = ?1"), [id], row).optional()?)
}

// doc/interfaces.md#save_device
pub fn save(c: &Connection, d: &Device) -> Result<(i64, bool)> {
    let name = d.name.trim();
    match d.id {
        None => {
            c.execute(
                "INSERT INTO devices (uuid, name, category, description, caliber, bullet_diameter_mm, retired) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![uuid_text(&Uuid::new_v4()), name, d.category, d.description, d.caliber, d.bullet_diameter_mm, d.retired],
            )?;
            Ok((c.last_insert_rowid(), true))
        }
        Some(id) => {
            let current = get(c, id)?.ok_or_else(|| HandlerError::not_found(format!("device {id}")))?;
            c.execute(
                "UPDATE devices SET name = ?1, category = ?2, description = ?3, caliber = ?4, retired = ?5 WHERE id = ?6",
                params![name, d.category, d.description, d.caliber, d.retired, id],
            )?;
            if current.bullet_diameter_mm != d.bullet_diameter_mm {
                if current.locked {
                    return Err(HandlerError::InUse("bullet diameter is used by a session".into()));
                }
                c.execute("UPDATE devices SET bullet_diameter_mm = ?1 WHERE id = ?2", params![d.bullet_diameter_mm, id])?;
            }
            Ok((id, false))
        }
    }
}

// doc/interfaces.md#import_device
pub fn import(c: &Connection, d: &Device) -> Result<(i64, bool)> {
    let uuid = d.uuid.ok_or_else(|| HandlerError::validation("uuid is required"))?;
    if let Some(id) = c.query_row("SELECT id FROM devices WHERE uuid = ?1", [uuid_text(&uuid)], |r| r.get(0)).optional()? {
        return Ok((id, false));
    }
    c.execute(
        "INSERT INTO devices (uuid, name, category, description, caliber, bullet_diameter_mm, shot_count, retired) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![uuid_text(&uuid), d.name.trim(), d.category, d.description, d.caliber, d.bullet_diameter_mm, d.shot_count, d.retired],
    )?;
    Ok((c.last_insert_rowid(), true))
}

pub fn set_image(c: &Connection, id: i64, image_path: Option<&str>) -> Result<Option<String>> {
    let old: Option<String> = c
        .query_row("SELECT image_path FROM devices WHERE id = ?1", [id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| HandlerError::not_found(format!("device {id}")))?;
    c.execute("UPDATE devices SET image_path = ?1 WHERE id = ?2", params![image_path, id])?;
    Ok(old)
}

pub fn find(c: &Connection, f: &Filter) -> Result<Vec<Device>> {
    let sql = filter_to_sql(f, DEVICE_COLUMNS)?;
    let order = if f.order_by.is_none() { " ORDER BY name COLLATE NOCASE, id" } else { "" };
    let mut stmt = c.prepare(&format!("{SELECT}{}", insert_order(&sql.clause, order)))?;
    let rows = stmt.query_map(rusqlite::params_from_iter(sql.params), row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

// doc/interfaces.md#delete_device
pub fn delete(c: &Connection, id: i64) -> Result<Option<String>> {
    let device = get(c, id)?.ok_or_else(|| HandlerError::not_found(format!("device {id}")))?;
    if used_by_session(c, "device_id", id)? {
        return Err(HandlerError::InUse(format!("device {id} is used by a session")));
    }
    c.execute("DELETE FROM devices WHERE id = ?1", [id])?;
    Ok(device.image_path)
}
