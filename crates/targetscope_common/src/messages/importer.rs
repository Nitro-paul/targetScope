//! Messages of `importer` (doc/interfaces.md#importer).
//! Source of truth: `schemas/importer/`.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::Request;

pub const STREAM: &str = "importer:requests";

/// Requests understood by `importer`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ImporterRequest {
    Ping {},
    ImportFile(ImportFile),
}

/// `import_file` request (doc/interfaces.md#import_file, schemas/importer/import_file.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportFile {
    pub staged_file: String,
}

/// `import_file_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportFileResponse {
    // TODO: fields
}

impl Request for ImportFile {
    type Response = ImportFileResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(300);
}
