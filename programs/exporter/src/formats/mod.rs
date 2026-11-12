//! Export formats (doc/programs/exporter.md#formats).

pub mod csv;
pub mod json;
pub mod png;
pub mod zip;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Csv,
    Json,
    Png,
}
