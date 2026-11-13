// doc/programs/exporter.md#export-flow-export_sessions
// doc/programs/exporter.md#list_exports-and-delete_export

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use targetscope_common::clock::local;
use targetscope_common::failpoints;
use targetscope_common::messages::exporter::ExportFile;
use uuid::Uuid;

pub fn id8(request_id: &Uuid) -> String {
    request_id.simple().to_string()[..8].to_string()
}

// doc/programs/exporter.md#export-flow-export_sessions
pub fn prefix(now_s: i64, request_id: &Uuid) -> String {
    let l = local(now_s);
    format!("{:04}-{:02}-{:02}_{:02}{:02}{:02}_{}", l.date.year, l.date.month, l.date.day, l.hour, l.minute, l.second, id8(request_id))
}

fn is_part(name: &str) -> bool {
    name.starts_with('.') && name.ends_with(".part")
}

// doc/programs/exporter.md#export-flow-export_sessions
pub fn existing_files(export_dir: &Path, request_id: &Uuid) -> io::Result<Vec<String>> {
    let tag = format!("_{}", id8(request_id));
    let mut files: Vec<String> = list_names(export_dir)?.into_iter().filter(|n| n.len() > 17 && n[17..].starts_with(&tag)).collect();
    files.sort();
    Ok(files)
}

fn list_names(dir: &Path) -> io::Result<Vec<String>> {
    let mut names = Vec::new();
    for e in fs::read_dir(dir)? {
        let e = e?;
        if e.file_type()?.is_file() {
            let name = e.file_name().to_string_lossy().into_owned();
            if !is_part(&name) {
                names.push(name);
            }
        }
    }
    Ok(names)
}

pub fn part_path(export_dir: &Path, name: &str) -> PathBuf {
    export_dir.join(format!(".{name}.part"))
}

// doc/programs/exporter.md#export-flow-export_sessions
pub fn write_atomic(export_dir: &Path, name: &str, write: impl FnOnce(&mut fs::File) -> io::Result<()>) -> io::Result<()> {
    let part = part_path(export_dir, name);
    let result = fs::File::create(&part).and_then(|mut f| {
        write(&mut f)?;
        f.flush()?;
        f.sync_all()
    });
    if let Err(e) = result {
        let _ = fs::remove_file(&part);
        return Err(e);
    }
    if failpoints::hit("exporter.before_rename") {
        let _ = fs::remove_file(&part);
        return Err(io::Error::other("failpoint exporter.before_rename"));
    }
    fs::rename(&part, export_dir.join(name)).inspect_err(|_| {
        let _ = fs::remove_file(&part);
    })
}

// doc/programs/exporter.md#export-flow-export_sessions
pub fn remove_part_files(export_dir: &Path) {
    let Ok(entries) = fs::read_dir(export_dir) else { return };
    for e in entries.flatten() {
        if is_part(&e.file_name().to_string_lossy()) {
            let _ = fs::remove_file(e.path());
        }
    }
}

// doc/interfaces.md#list_exports
pub fn list(export_dir: &Path) -> io::Result<Vec<ExportFile>> {
    let mut files = Vec::new();
    for name in list_names(export_dir)? {
        let meta = fs::metadata(export_dir.join(&name))?;
        let created_ms = meta.modified()?.duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0);
        files.push(ExportFile { name, size_bytes: meta.len() as i64, created_ms });
    }
    files.sort_by(|a, b| b.created_ms.cmp(&a.created_ms).then(b.name.cmp(&a.name)));
    Ok(files)
}

// doc/interfaces.md#staging-folder
pub fn is_plain_name(name: &str) -> bool {
    !name.is_empty() && !name.contains('/') && !name.contains('\\') && name != "." && name != ".." && !name.contains('\0')
}
