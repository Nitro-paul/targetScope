//! The export file format (schemas/export/v1.schema.json), as read by the importer.

use serde::Deserialize;

pub const FORMAT: &str = "targetscope-export";
pub const SUPPORTED_VERSIONS: &[u32] = &[1];
/// Embedded schema, validated before anything is imported.
// TODO: pub const SCHEMA: &str = include_str!("../../../schemas/export/v1.schema.json");

#[derive(Debug, Clone, Deserialize)]
pub struct ExportFile {
    pub format: String,
    pub format_version: u32,
    pub exported_at: String,
    // TODO: shooters, target_types, devices, sessions (doc/programs/exporter.md#json)
}

/// Step 2: validate against the export schema; Err names the first problem.
pub fn parse_and_validate(json: &[u8]) -> Result<ExportFile, String> {
    let _ = json;
    todo!()
}
