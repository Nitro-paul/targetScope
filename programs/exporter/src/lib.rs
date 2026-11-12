//! exporter (doc/programs/exporter.md): export_sessions, list_exports, delete_export.
//! Read-only access to the image folder for ZIP exports.

pub mod collect;
pub mod files;
pub mod formats;
pub mod handler;

use std::process::ExitCode;

/// Startup: logging, bus, settings cache, delete leftover .part files, serve requests.
pub async fn run() -> ExitCode {
    todo!()
}
