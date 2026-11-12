//! Device queries.

use rusqlite::Connection;
use targetscope_common::model::{Device, Filter};

use crate::error::HandlerError;

/// Insert (new uuid) if id is None, otherwise update; never overwrites uuid, shot_count or image
/// fields; InUse if bullet_diameter_mm changes on a locked device.
pub fn save(c: &Connection, d: &Device) -> Result<i64, HandlerError> {
    let _ = (c, d);
    todo!()
}

/// Creates the device with its uuid and shot_count unless the uuid exists; returns (id, created).
pub fn import(c: &Connection, d: &Device) -> Result<(i64, bool), HandlerError> {
    let _ = (c, d);
    todo!()
}

pub fn is_locked(c: &Connection, id: i64) -> Result<bool, HandlerError> {
    let _ = (c, id);
    todo!()
}

/// Returns the previous image_path.
pub fn set_image(
    c: &Connection,
    id: i64,
    image_path: Option<&str>,
) -> Result<Option<String>, HandlerError> {
    let _ = (c, id, image_path);
    todo!()
}

pub fn find(c: &Connection, f: &Filter) -> Result<Vec<Device>, HandlerError> {
    let _ = (c, f);
    todo!()
}

/// InUse if a session references the device; returns image_path.
pub fn delete(c: &Connection, id: i64) -> Result<Option<String>, HandlerError> {
    let _ = (c, id);
    todo!()
}
