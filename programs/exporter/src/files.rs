//! File names, .part files and duplicate protection.
//! Names start with `<yyyy-mm-dd_HHMMSS>_<id8>_…` (id8 = first 8 characters of request_id);
//! a repeated request returns the existing files with the same id8.

use std::path::{Path, PathBuf};

use uuid::Uuid;

pub fn prefix(now: time::OffsetDateTime, request_id: &Uuid) -> String {
    let _ = (now, request_id);
    todo!()
}

/// Files of an earlier export with the same request_id.
pub fn existing_files(export_dir: &Path, request_id: &Uuid) -> std::io::Result<Vec<String>> {
    let _ = (export_dir, request_id);
    todo!()
}

/// Writes `.<name>.part`, then renames (failpoint exporter.before_rename between the two).
pub fn write_atomic(
    export_dir: &Path,
    name: &str,
    write: impl FnOnce(&Path) -> std::io::Result<()>,
) -> std::io::Result<PathBuf> {
    let _ = (export_dir, name, write);
    todo!()
}

/// Deletes leftover .part files at startup.
pub fn remove_part_files(export_dir: &Path) -> std::io::Result<()> {
    let _ = export_dir;
    todo!()
}

pub struct ExportFile {
    pub name: String,
    pub size_bytes: u64,
    pub created_ms: i64,
}

/// list_exports: files without .part, newest first.
pub fn list(export_dir: &Path) -> std::io::Result<Vec<ExportFile>> {
    let _ = export_dir;
    todo!()
}

/// delete_export: plain file name only (no separators, no "..").
pub fn delete(export_dir: &Path, name: &str) -> std::io::Result<()> {
    let _ = (export_dir, name);
    todo!()
}
