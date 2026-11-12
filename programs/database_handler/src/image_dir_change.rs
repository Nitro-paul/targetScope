// doc/programs/database_handler.md#image-folder-change

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use targetscope_common::failpoints;

use crate::error::Result;

pub const SUBFOLDERS: [&str; 2] = ["devices", "sessions"];

pub struct Copied {
    pub copied: Vec<String>,
    pub failed: Vec<String>,
}

pub fn list_images(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for sub in SUBFOLDERS {
        let Ok(entries) = fs::read_dir(dir.join(sub)) else { continue };
        for e in entries.flatten() {
            if e.file_type().is_ok_and(|t| t.is_file()) {
                files.push(format!("{sub}/{}", e.file_name().to_string_lossy()));
            }
        }
    }
    files.sort();
    files
}

fn copy_one(old: &Path, new: &Path, rel: &str) -> std::io::Result<()> {
    let target = new.join(rel);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(old.join(rel), target).map(|_| ())
}

// doc/programs/database_handler.md#image-folder-change
pub fn copy_all(old: &Path, new: &Path) -> Copied {
    let mut result = Copied { copied: Vec::new(), failed: Vec::new() };
    for sub in SUBFOLDERS {
        let _ = fs::create_dir_all(new.join(sub));
    }
    for rel in list_images(old) {
        failpoints::hit("db.image_dir_copy");
        match copy_one(old, new, &rel) {
            Ok(()) => result.copied.push(rel),
            Err(e) => {
                tracing::warn!(image_path = %rel, error = %e, "image could not be copied");
                result.failed.push(rel);
            }
        }
    }
    result
}

// doc/programs/database_handler.md#image-folder-change
pub fn sync(old: &Path, new: &Path, copied: &[String], failed: &[String]) -> Result<()> {
    let now: HashSet<String> = list_images(old).into_iter().collect();
    let before: HashSet<&str> = copied.iter().chain(failed).map(String::as_str).collect();
    for rel in now.iter().filter(|r| !before.contains(r.as_str())) {
        if let Err(e) = copy_one(old, new, rel) {
            tracing::warn!(image_path = %rel, error = %e, "image could not be copied");
        }
    }
    for rel in copied.iter().filter(|r| !now.contains(*r)) {
        let _ = fs::remove_file(new.join(rel));
    }
    Ok(())
}

// doc/programs/database_handler.md#image-folder-change
pub fn clean_up(old: &Path, new: &Path) -> (i64, Vec<String>) {
    let mut moved = 0;
    let mut failed = Vec::new();
    for rel in list_images(old) {
        if new.join(&rel).is_file() && fs::remove_file(old.join(&rel)).is_ok() {
            moved += 1;
        } else {
            failed.push(rel);
        }
    }
    (moved, failed)
}

// doc/programs/database_handler.md#validation-rules
pub fn same_device(a: &Path, b: &Path) -> Option<bool> {
    use std::os::unix::fs::MetadataExt;
    let dev = |p: &Path| p.ancestors().find_map(|a| fs::metadata(a).ok()).map(|m| m.dev());
    Some(dev(a)? == dev(b)?)
}

pub fn same_folder(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}
