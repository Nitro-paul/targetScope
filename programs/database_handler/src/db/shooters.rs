//! Shooter queries.

use rusqlite::Connection;
use targetscope_common::model::{Filter, Shooter};

use crate::error::HandlerError;

/// Insert (new uuid) or update (name, retired); name trimmed, unique ignoring case.
pub fn save(c: &Connection, s: &Shooter) -> Result<i64, HandlerError> {
    let _ = (c, s);
    todo!()
}

/// Existing shooter with the same uuid or, failing that, the same name; returns (id, created).
pub fn import(c: &Connection, s: &Shooter) -> Result<(i64, bool), HandlerError> {
    let _ = (c, s);
    todo!()
}

pub fn find(c: &Connection, f: &Filter) -> Result<Vec<Shooter>, HandlerError> {
    let _ = (c, f);
    todo!()
}

/// InUse if a session references the shooter.
pub fn delete(c: &Connection, id: i64) -> Result<(), HandlerError> {
    let _ = (c, id);
    todo!()
}
