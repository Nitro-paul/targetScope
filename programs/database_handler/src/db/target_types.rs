//! Target type queries.

use rusqlite::Connection;
use targetscope_common::model::{Filter, TargetType};

use crate::error::HandlerError;

/// Insert (new uuid) or update; InUse if anything but name and retired changes on a locked type.
pub fn save(c: &Connection, t: &TargetType) -> Result<i64, HandlerError> {
    let _ = (c, t);
    todo!()
}

/// Creates the type with its uuid unless the uuid exists; renames on name conflict
/// (" (imported)", " (imported 2)", …); returns (id, created).
pub fn import(c: &Connection, t: &TargetType) -> Result<(i64, bool), HandlerError> {
    let _ = (c, t);
    todo!()
}

/// Types with generated rings (targetscope_common::targets::generate_rings) and `locked`.
pub fn find(c: &Connection, f: &Filter) -> Result<Vec<TargetType>, HandlerError> {
    let _ = (c, f);
    todo!()
}

/// InUse if a session references the type.
pub fn delete(c: &Connection, id: i64) -> Result<(), HandlerError> {
    let _ = (c, id);
    todo!()
}
