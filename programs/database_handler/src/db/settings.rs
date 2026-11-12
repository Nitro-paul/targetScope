//! Setting queries.

use rusqlite::Connection;
use targetscope_common::model::{Filter, Setting};

use crate::error::HandlerError;

/// INSERT ... ON CONFLICT(name) DO UPDATE
pub fn save(c: &Connection, s: &Setting) -> Result<(), HandlerError> {
    let _ = (c, s);
    todo!()
}

/// Sets value and active of the given known settings back to their defaults.
pub fn reset(c: &Connection, names: &[&str]) -> Result<(), HandlerError> {
    let _ = (c, names);
    todo!()
}

pub fn find(c: &Connection, f: &Filter) -> Result<Vec<Setting>, HandlerError> {
    let _ = (c, f);
    todo!()
}

pub fn get(c: &Connection, name: &str) -> Result<Option<String>, HandlerError> {
    let _ = (c, name);
    todo!()
}

pub fn delete(c: &Connection, name: &str) -> Result<(), HandlerError> {
    let _ = (c, name);
    todo!()
}
