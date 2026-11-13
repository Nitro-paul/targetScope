// doc/programs/importer.md#import-flow-import_file

use targetscope_common::export::ExportFile;
use targetscope_common::messages::ServiceError;

pub const SCHEMA: &str = include_str!("../../../schemas/export/v1.schema.json");
pub const MAX_JSON_BYTES: u64 = 100 * 1024 * 1024;

// doc/programs/importer.md#import-flow-import_file
pub fn parse_and_validate(bytes: &[u8]) -> Result<ExportFile, ServiceError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| ServiceError::validation(format!("not a JSON file: {e}")))?;
    if value.get("format").and_then(|v| v.as_str()) != Some(targetscope_common::export::FORMAT) {
        return Err(ServiceError::validation("format must be targetscope-export"));
    }
    if value.get("format_version").and_then(|v| v.as_u64()) != Some(targetscope_common::export::FORMAT_VERSION as u64) {
        return Err(ServiceError::validation("unsupported format_version"));
    }
    let schema: serde_json::Value = serde_json::from_str(SCHEMA).expect("export schema is JSON");
    let validator = jsonschema::validator_for(&schema).expect("export schema compiles");
    if let Err(e) = validator.validate(&value) {
        return Err(ServiceError::validation(format!("{} at {}", e, e.instance_path())));
    }
    serde_json::from_value(value).map_err(|e| ServiceError::validation(e.to_string()))
}
