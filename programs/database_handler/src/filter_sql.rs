// doc/programs/database_handler.md#filter-to-sql

use rusqlite::types::Value as SqlValue;
use serde_json::Value;
use targetscope_common::model::{Direction, Filter, FilterOp, SessionFilter};

use crate::error::{HandlerError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    Integer,
    Real,
    Text,
    Bool,
}

pub const SHOOTER_COLUMNS: &[(&str, ColumnType)] =
    &[("id", ColumnType::Integer), ("uuid", ColumnType::Text), ("name", ColumnType::Text), ("retired", ColumnType::Bool)];

pub const DEVICE_COLUMNS: &[(&str, ColumnType)] = &[
    ("id", ColumnType::Integer),
    ("uuid", ColumnType::Text),
    ("name", ColumnType::Text),
    ("category", ColumnType::Text),
    ("description", ColumnType::Text),
    ("caliber", ColumnType::Text),
    ("image_path", ColumnType::Text),
    ("bullet_diameter_mm", ColumnType::Real),
    ("shot_count", ColumnType::Integer),
    ("retired", ColumnType::Bool),
];

pub const TARGET_TYPE_COLUMNS: &[(&str, ColumnType)] = &[
    ("id", ColumnType::Integer),
    ("uuid", ColumnType::Text),
    ("name", ColumnType::Text),
    ("paper_width_mm", ColumnType::Real),
    ("paper_height_mm", ColumnType::Real),
    ("center_x_mm", ColumnType::Real),
    ("center_y_mm", ColumnType::Real),
    ("inner_white_radius_mm", ColumnType::Real),
    ("inner_white_score", ColumnType::Integer),
    ("black_radius_mm", ColumnType::Real),
    ("black_score", ColumnType::Integer),
    ("outer_radius_mm", ColumnType::Real),
    ("outer_score", ColumnType::Integer),
    ("retired", ColumnType::Bool),
];

pub const SETTING_COLUMNS: &[(&str, ColumnType)] =
    &[("name", ColumnType::Text), ("display_name", ColumnType::Text), ("active", ColumnType::Bool), ("value", ColumnType::Text)];

pub struct Sql {
    pub clause: String,
    pub params: Vec<SqlValue>,
}

// doc/interfaces.md#filter
pub fn filter_to_sql(filter: &Filter, columns: &[(&str, ColumnType)]) -> Result<Sql> {
    let mut parts = Vec::new();
    let mut params = Vec::new();
    for c in &filter.conditions {
        let (_, kind) = columns
            .iter()
            .find(|(name, _)| *name == c.field)
            .ok_or_else(|| HandlerError::validation(format!("unknown filter field {}", c.field)))?;
        let col = c.field.as_str();
        match c.op {
            FilterOp::In => {
                let list = c.value.as_array().ok_or_else(|| HandlerError::validation("in needs a list"))?;
                if list.is_empty() {
                    parts.push("0".to_string());
                    continue;
                }
                for v in list {
                    params.push(to_sql(v, *kind)?);
                }
                parts.push(format!("{col} IN ({})", vec!["?"; list.len()].join(", ")));
            }
            FilterOp::Like => {
                if *kind != ColumnType::Text {
                    return Err(HandlerError::validation(format!("like is only allowed on text fields, not {col}")));
                }
                params.push(to_sql(&c.value, *kind)?);
                parts.push(format!("{col} LIKE ?"));
            }
            op if c.value.is_null() => match op {
                FilterOp::Eq => parts.push(format!("{col} IS NULL")),
                FilterOp::Ne => parts.push(format!("{col} IS NOT NULL")),
                _ => return Err(HandlerError::validation(format!("null can only be compared with eq or ne ({col})"))),
            },
            op => {
                params.push(to_sql(&c.value, *kind)?);
                let sym = match op {
                    FilterOp::Eq => "=",
                    FilterOp::Ne => "<>",
                    FilterOp::Lt => "<",
                    FilterOp::Le => "<=",
                    FilterOp::Gt => ">",
                    FilterOp::Ge => ">=",
                    FilterOp::Like | FilterOp::In => unreachable!(),
                };
                parts.push(format!("{col} {sym} ?"));
            }
        }
    }
    let mut clause = if parts.is_empty() { String::new() } else { format!(" WHERE {}", parts.join(" AND ")) };
    if let Some(o) = &filter.order_by {
        if !columns.iter().any(|(name, _)| *name == o.field) {
            return Err(HandlerError::validation(format!("unknown order field {}", o.field)));
        }
        clause.push_str(&format!(" ORDER BY {} {}", o.field, direction(o.direction)));
    }
    page(&mut clause, &mut params, filter.limit, filter.offset)?;
    Ok(Sql { clause, params })
}

pub fn direction(d: Direction) -> &'static str {
    match d {
        Direction::Asc => "ASC",
        Direction::Desc => "DESC",
    }
}

pub fn page(clause: &mut String, params: &mut Vec<SqlValue>, limit: Option<i64>, offset: Option<i64>) -> Result<()> {
    if limit.is_some_and(|l| l < 0) || offset.is_some_and(|o| o < 0) {
        return Err(HandlerError::validation("limit and offset are >= 0"));
    }
    if limit.is_some() || offset.is_some() {
        clause.push_str(" LIMIT ? OFFSET ?");
        params.push(SqlValue::Integer(limit.unwrap_or(-1)));
        params.push(SqlValue::Integer(offset.unwrap_or(0)));
    }
    Ok(())
}

fn to_sql(v: &Value, kind: ColumnType) -> Result<SqlValue> {
    let bad = || HandlerError::validation(format!("value {v} doesn't fit the field"));
    Ok(match (kind, v) {
        (ColumnType::Bool, Value::Bool(b)) => SqlValue::Integer(*b as i64),
        (ColumnType::Integer | ColumnType::Bool, Value::Number(n)) => SqlValue::Integer(n.as_i64().ok_or_else(bad)?),
        (ColumnType::Real, Value::Number(n)) => SqlValue::Real(n.as_f64().ok_or_else(bad)?),
        (ColumnType::Text, Value::String(s)) => SqlValue::Text(s.clone()),
        _ => return Err(bad()),
    })
}

// doc/interfaces.md#sessionfilter
pub const SESSION_FROM: &str = "FROM sessions s \
    JOIN devices d ON d.id = s.device_id \
    JOIN target_types t ON t.id = s.target_type_id \
    LEFT JOIN shooters sh ON sh.id = s.shooter_id";

// doc/programs/database_handler.md#filter-to-sql
pub fn session_filter_to_sql(f: &SessionFilter) -> Sql {
    let mut parts: Vec<String> = Vec::new();
    let mut params: Vec<SqlValue> = Vec::new();
    let mut list_i = |parts: &mut Vec<String>, col: &str, list: &Option<Vec<i64>>| {
        if let Some(list) = list {
            push_list(parts, &mut params, col, list.iter().map(|v| SqlValue::Integer(*v)).collect());
        }
    };
    list_i(&mut parts, "s.id", &f.session_ids);
    list_i(&mut parts, "s.shooter_id", &f.shooter_ids);
    list_i(&mut parts, "s.device_id", &f.device_ids);
    list_i(&mut parts, "s.target_type_id", &f.target_type_ids);
    if let Some(list) = &f.device_categories {
        push_list(&mut parts, &mut params, "d.category", list.iter().map(|v| SqlValue::Text(v.clone())).collect());
    }
    if let Some(list) = &f.calibers {
        push_list(&mut parts, &mut params, "d.caliber", list.iter().map(|v| SqlValue::Text(v.clone())).collect());
    }
    let mut range = |parts: &mut Vec<String>, col: &str, min: Option<i64>, max: Option<i64>| {
        if let Some(v) = min {
            parts.push(format!("{col} >= ?"));
            params.push(SqlValue::Integer(v));
        }
        if let Some(v) = max {
            parts.push(format!("{col} <= ?"));
            params.push(SqlValue::Integer(v));
        }
    };
    if let Some(r) = f.distance_m {
        range(&mut parts, "s.distance_m", r.min, r.max);
    }
    if let Some(r) = f.saved_at {
        range(&mut parts, "s.saved_at", r.from, r.to);
    }
    if let Some(r) = f.shot_count {
        range(&mut parts, "s.shot_count", r.min, r.max);
    }
    if let Some(b) = f.single_handed {
        parts.push(format!("s.single_handed = {}", b as i64));
    }
    if let Some(b) = f.supported {
        parts.push(format!("s.supported = {}", b as i64));
    }
    if f.include_retired == Some(false) {
        parts.push("d.retired = 0 AND t.retired = 0 AND (sh.id IS NULL OR sh.retired = 0)".into());
    }
    let clause = if parts.is_empty() { String::new() } else { format!(" WHERE {}", parts.join(" AND ")) };
    Sql { clause, params }
}

fn push_list(parts: &mut Vec<String>, params: &mut Vec<SqlValue>, col: &str, values: Vec<SqlValue>) {
    if values.is_empty() {
        parts.push("0".into());
        return;
    }
    parts.push(format!("{col} IN ({})", vec!["?"; values.len()].join(", ")));
    params.extend(values);
}
