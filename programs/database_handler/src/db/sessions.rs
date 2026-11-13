// doc/database.md#sessions
// doc/database.md#records

use std::collections::HashMap;

use rusqlite::types::Value as SqlValue;
use rusqlite::{params, Connection, OptionalExtension, Row};
use targetscope_common::messages::database_handler::{SessionOrder, SessionOrderField};
use targetscope_common::model::{Direction, NewSession, PaperImage, Session, SessionFilter, SessionShot};
use uuid::Uuid;

use super::{parse_uuid, uuid_text};
use crate::error::{HandlerError, Result};
use crate::filter_sql::{direction, page, session_filter_to_sql, SESSION_FROM};

// doc/programs/database_handler.md#validation-rules
pub fn check_references(c: &Connection, s: &NewSession) -> Result<()> {
    if !super::exists(c, "devices", s.device_id)? {
        return Err(HandlerError::validation(format!("unknown device_id {}", s.device_id)));
    }
    if !super::exists(c, "target_types", s.target_type_id)? {
        return Err(HandlerError::validation(format!("unknown target_type_id {}", s.target_type_id)));
    }
    if let Some(id) = s.shooter_id {
        if !super::exists(c, "shooters", id)? {
            return Err(HandlerError::validation(format!("unknown shooter_id {id}")));
        }
    }
    Ok(())
}

// doc/programs/database_handler.md#database-and-queries
pub fn insert(c: &Connection, uuid: Uuid, saved_at: i64, s: &NewSession, images: &[(i64, String)]) -> Result<i64> {
    c.execute(
        "INSERT INTO sessions (uuid, saved_at, score, grouping_score, distance_m, target_type_id, shooter_id, device_id, \
         single_handed, supported, shot_count, duration_s, notes, paper_count) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            uuid_text(&uuid),
            saved_at,
            s.score,
            s.grouping_score,
            s.distance_m,
            s.target_type_id,
            s.shooter_id,
            s.device_id,
            s.single_handed,
            s.supported,
            s.shot_count,
            s.duration_s,
            s.notes,
            s.paper_count
        ],
    )?;
    let id = c.last_insert_rowid();
    let mut shot =
        c.prepare_cached("INSERT INTO shots (session_id, shot_no, paper_no, angle_turns, radius_mm) VALUES (?1, ?2, ?3, ?4, ?5)")?;
    for (i, s) in s.shots.iter().enumerate() {
        shot.execute(params![id, i as i64 + 1, s.paper_no, s.angle_turns, s.radius_mm])?;
    }
    let mut image = c.prepare_cached("INSERT INTO session_images (session_id, paper_no, image_path) VALUES (?1, ?2, ?3)")?;
    for (paper_no, path) in images {
        image.execute(params![id, paper_no, path])?;
    }
    Ok(id)
}

// doc/interfaces.md#create_session
pub fn create(c: &Connection, uuid: Uuid, saved_at: i64, s: &NewSession, images: &[(i64, String)]) -> Result<i64> {
    let id = insert(c, uuid, saved_at, s, images)?;
    c.execute("UPDATE devices SET shot_count = shot_count + ?1 WHERE id = ?2", params![s.shot_count, s.device_id])?;
    Ok(id)
}

pub fn exists(c: &Connection, uuid: Uuid) -> Result<Option<i64>> {
    Ok(c.query_row("SELECT id FROM sessions WHERE uuid = ?1", [uuid_text(&uuid)], |r| r.get(0)).optional()?)
}

pub fn uuid_of(c: &Connection, id: i64) -> Result<Option<Uuid>> {
    let text: Option<String> = c.query_row("SELECT uuid FROM sessions WHERE id = ?1", [id], |r| r.get(0)).optional()?;
    text.map(|t| parse_uuid(t).map_err(HandlerError::from)).transpose()
}

const COLUMNS: &str = "s.id, s.uuid, s.saved_at, s.score, s.grouping_score, s.distance_m, s.target_type_id, \
    s.shooter_id, s.device_id, s.single_handed, s.supported, s.shot_count, s.duration_s, s.notes, s.paper_count";

fn row(r: &Row) -> rusqlite::Result<Session> {
    Ok(Session {
        id: r.get(0)?,
        uuid: parse_uuid(r.get(1)?)?,
        saved_at: r.get(2)?,
        score: r.get(3)?,
        grouping_score: r.get(4)?,
        distance_m: r.get(5)?,
        target_type_id: r.get(6)?,
        shooter_id: r.get(7)?,
        device_id: r.get(8)?,
        single_handed: r.get(9)?,
        supported: r.get(10)?,
        shot_count: r.get(11)?,
        duration_s: r.get(12)?,
        notes: r.get(13)?,
        paper_count: r.get(14)?,
        images: vec![],
        shots: vec![],
    })
}

// doc/interfaces.md#read_sessions
pub fn find(
    c: &Connection,
    f: &SessionFilter,
    order: Option<SessionOrder>,
    limit: Option<i64>,
    offset: Option<i64>,
    include_shots: bool,
) -> Result<(Vec<Session>, i64)> {
    let sql = session_filter_to_sql(f);
    let total: i64 =
        c.query_row(&format!("SELECT COUNT(*) {SESSION_FROM}{}", sql.clause), rusqlite::params_from_iter(&sql.params), |r| r.get(0))?;
    let order = order.unwrap_or(SessionOrder { field: SessionOrderField::SavedAt, direction: Direction::Desc });
    let field = match order.field {
        SessionOrderField::SavedAt => "s.saved_at",
        SessionOrderField::Score => "s.score",
        SessionOrderField::ShotCount => "s.shot_count",
        SessionOrderField::Id => "s.id",
    };
    let dir = direction(order.direction);
    let mut clause = format!("{} ORDER BY {field} {dir}, s.id {dir}", sql.clause);
    let mut params: Vec<SqlValue> = sql.params;
    page(&mut clause, &mut params, limit, offset)?;
    let mut stmt = c.prepare(&format!("SELECT {COLUMNS} {SESSION_FROM}{clause}"))?;
    let mut sessions: Vec<Session> = stmt.query_map(rusqlite::params_from_iter(params), row)?.collect::<rusqlite::Result<_>>()?;
    if sessions.is_empty() {
        return Ok((sessions, total));
    }
    let ids: Vec<SqlValue> = sessions.iter().map(|s| SqlValue::Integer(s.id)).collect();
    let marks = vec!["?"; ids.len()].join(", ");
    let index: HashMap<i64, usize> = sessions.iter().enumerate().map(|(i, s)| (s.id, i)).collect();
    let mut img = c.prepare(&format!(
        "SELECT session_id, paper_no, image_path FROM session_images WHERE session_id IN ({marks}) ORDER BY session_id, paper_no"
    ))?;
    let mut rows = img.query(rusqlite::params_from_iter(&ids))?;
    while let Some(r) = rows.next()? {
        let i = index[&r.get::<_, i64>(0)?];
        sessions[i].images.push(PaperImage { paper_no: r.get(1)?, image_path: r.get(2)? });
    }
    if include_shots {
        let mut shots = c.prepare(&format!(
            "SELECT session_id, angle_turns, radius_mm, paper_no FROM shots WHERE session_id IN ({marks}) ORDER BY session_id, shot_no"
        ))?;
        let mut rows = shots.query(rusqlite::params_from_iter(&ids))?;
        while let Some(r) = rows.next()? {
            let i = index[&r.get::<_, i64>(0)?];
            sessions[i].shots.push(SessionShot { angle_turns: r.get(1)?, radius_mm: r.get(2)?, paper_no: r.get(3)? });
        }
    }
    Ok((sessions, total))
}

// doc/interfaces.md#delete_session
pub fn delete(c: &Connection, id: i64) -> Result<Vec<String>> {
    if !super::exists(c, "sessions", id)? {
        return Err(HandlerError::not_found(format!("session {id}")));
    }
    let mut stmt = c.prepare("SELECT image_path FROM session_images WHERE session_id = ?1")?;
    let paths: Vec<String> = stmt.query_map([id], |r| r.get(0))?.collect::<rusqlite::Result<_>>()?;
    c.execute("DELETE FROM sessions WHERE id = ?1", [id])?;
    Ok(paths)
}

// doc/programs/database_handler.md#startup
pub fn all_image_paths(c: &Connection) -> Result<Vec<String>> {
    let mut stmt =
        c.prepare("SELECT image_path FROM session_images UNION ALL SELECT image_path FROM devices WHERE image_path IS NOT NULL")?;
    let paths = stmt.query_map([], |r| r.get(0))?.collect::<rusqlite::Result<_>>()?;
    Ok(paths)
}
