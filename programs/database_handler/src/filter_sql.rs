//! Filter and SessionFilter -> SQL (doc/programs/database_handler.md#filter-to-sql).
//! Values are always bound as parameters, never formatted into SQL.

use targetscope_common::model::{Filter, SessionFilter};

use crate::error::HandlerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    Integer,
    Real,
    Text,
    Bool,
}

/// Builds "WHERE ... ORDER BY ... LIMIT ... OFFSET ..." and its parameters.
/// `columns` is the column whitelist of the queried table; `like` only on text columns.
pub fn filter_to_sql(
    filter: &Filter,
    columns: &[(&str, ColumnType)],
) -> Result<(String, Vec<rusqlite::types::Value>), HandlerError> {
    let _ = (filter, columns);
    todo!()
}

/// WHERE clause for a SessionFilter, with joins for device fields and include_retired.
/// An empty list becomes a condition that matches nothing.
pub fn session_filter_to_sql(filter: &SessionFilter) -> (String, Vec<rusqlite::types::Value>) {
    let _ = filter;
    todo!()
}
