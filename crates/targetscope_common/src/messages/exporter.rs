//! Messages of `exporter` (doc/interfaces.md#exporter).
//! Source of truth: `schemas/exporter/`.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::Request;

pub const STREAM: &str = "exporter:requests";

/// Requests understood by `exporter`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ExporterRequest {
    Ping {},
    ExportSessions(ExportSessions),
    ListExports(ListExports),
    DeleteExport(DeleteExport),
}

/// `export_sessions` request (doc/interfaces.md#export_sessions, schemas/exporter/export_sessions.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportSessions {
    // TODO: fields
}

/// `export_sessions_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportSessionsResponse {
    // TODO: fields
}

impl Request for ExportSessions {
    type Response = ExportSessionsResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(300);
}

/// `list_exports` request (doc/interfaces.md#list_exports, schemas/exporter/list_exports.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListExports {}

/// `list_exports_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListExportsResponse {
    // TODO: fields
}

impl Request for ListExports {
    type Response = ListExportsResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `delete_export` request (doc/interfaces.md#delete_export, schemas/exporter/delete_export.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteExport {
    pub name: String,
}

/// `delete_export_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteExportResponse {
    pub name: String,
}

impl Request for DeleteExport {
    type Response = DeleteExportResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}
