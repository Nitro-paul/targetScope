// doc/interfaces.md#exporter

use serde::{Deserialize, Serialize};

use super::{request, Ping};
use crate::model::SessionFilter;

pub const PROGRAM: &str = "exporter";
pub const STREAM: &str = "exporter:requests";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ExporterRequest {
    Ping(Ping),
    ExportSessions(ExportSessions),
    ListExports(ListExports),
    DeleteExport(DeleteExport),
}

// doc/interfaces.md#export_sessions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Csv,
    Json,
    Png,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportSessions {
    #[serde(default)]
    pub filter: SessionFilter,
    pub format: ExportFormat,
    #[serde(default)]
    pub include_images: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportSessionsResponse {
    pub files: Vec<String>,
}
request!(ExportSessions, ExportSessionsResponse, "export_sessions", 300);

// doc/interfaces.md#list_exports
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ListExports {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportFile {
    pub name: String,
    pub size_bytes: i64,
    pub created_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListExportsResponse {
    pub files: Vec<ExportFile>,
}
request!(ListExports, ListExportsResponse, "list_exports", 5);

// doc/interfaces.md#delete_export
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteExport {
    pub name: String,
}
request!(DeleteExport, DeleteExport, "delete_export", 5);
