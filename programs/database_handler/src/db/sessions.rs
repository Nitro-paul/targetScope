//! Session queries. Rows are never updated after insert.

use rusqlite::Connection;
use targetscope_common::model::{NewSession, Session, SessionFilter};
use uuid::Uuid;

use crate::error::HandlerError;

#[derive(Debug, Clone)]
pub struct SessionOrder {
    pub field: String,
    pub descending: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Page {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Inserts session (saved_at = now), shots and session_images; adds shot_count to the device.
pub fn create(
    c: &Connection,
    uuid: Uuid,
    s: &NewSession,
    images: &[(u32, String)],
) -> Result<i64, HandlerError> {
    let _ = (c, uuid, s, images);
    todo!()
}

/// Imported session with its uuid and saved_at; does not touch devices.shot_count. Returns (id, created).
pub fn import(
    c: &Connection,
    s: &Session,
    images: &[(u32, String)],
) -> Result<(i64, bool), HandlerError> {
    let _ = (c, s, images);
    todo!()
}

pub fn exists(c: &Connection, uuid: Uuid) -> Result<Option<i64>, HandlerError> {
    let _ = (c, uuid);
    todo!()
}

/// The page of sessions and the total number of matches.
pub fn find(
    c: &Connection,
    f: &SessionFilter,
    order: &SessionOrder,
    page: Page,
    include_shots: bool,
) -> Result<(Vec<Session>, u64), HandlerError> {
    let _ = (c, f, order, page, include_shots);
    todo!()
}

/// Returns the deleted session's image paths, so the files can be removed after commit.
pub fn delete(c: &Connection, id: i64) -> Result<Vec<String>, HandlerError> {
    let _ = (c, id);
    todo!()
}
