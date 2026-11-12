// doc/interfaces.md#importer

use serde::{Deserialize, Serialize};

use super::{request, Ping};

pub const PROGRAM: &str = "importer";
pub const STREAM: &str = "importer:requests";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ImporterRequest {
    Ping(Ping),
    ImportFile(ImportFile),
}

// doc/interfaces.md#import_file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportFile {
    pub staged_file: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportCounts {
    pub created: i64,
    pub existing: i64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoredImages {
    pub restored: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportFileResponse {
    pub shooters: ImportCounts,
    pub target_types: ImportCounts,
    pub devices: ImportCounts,
    pub sessions: ImportCounts,
    pub images: RestoredImages,
    pub warnings: Vec<String>,
}
request!(ImportFile, ImportFileResponse, "import_file", 300);
