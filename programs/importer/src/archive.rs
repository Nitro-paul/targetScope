// doc/programs/importer.md#import-flow-import_file

use std::fs::File;
use std::io::Read;
use std::path::Path;

use targetscope_common::export::JSON_ENTRY;
use targetscope_common::messages::ServiceError;
use uuid::Uuid;
use zip::ZipArchive;

pub const MAX_ZIP_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const MAX_UNPACKED_BYTES: u64 = 4 * 1024 * 1024 * 1024;

pub struct ImportArchive {
    zip: ZipArchive<File>,
}

fn invalid(msg: impl Into<String>) -> ServiceError {
    ServiceError::validation(msg)
}

// doc/programs/importer.md#import-flow-import_file
pub fn allowed_entry(name: &str) -> bool {
    if name == JSON_ENTRY {
        return true;
    }
    let image = |file: &str, with_paper: bool| -> bool {
        let Some((stem, ext)) = file.rsplit_once('.') else { return false };
        if ext != "jpg" && ext != "png" {
            return false;
        }
        let (uuid, rest) = if with_paper {
            match stem.rsplit_once('_') {
                Some((u, p)) => (u, Some(p)),
                None => return false,
            }
        } else {
            (stem, None)
        };
        Uuid::parse_str(uuid).is_ok_and(|u| u.hyphenated().to_string() == uuid.to_lowercase())
            && rest.is_none_or(|p| p.parse::<u32>().is_ok_and(|n| n >= 1) && !p.starts_with('0'))
    };
    match name.split_once('/').and_then(|(a, b)| b.split_once('/').map(|(c, d)| (a, c, d))) {
        Some(("images", "sessions", file)) => !file.contains('/') && image(file, true),
        Some(("images", "devices", file)) => !file.contains('/') && image(file, false),
        _ => false,
    }
}

impl ImportArchive {
    // doc/programs/importer.md#import-flow-import_file
    pub fn open(path: &Path) -> Result<Self, ServiceError> {
        let file = File::open(path).map_err(|e| ServiceError::io(e.to_string()))?;
        let mut zip = ZipArchive::new(file).map_err(|e| invalid(format!("not a ZIP file: {e}")))?;
        let mut total: u64 = 0;
        for i in 0..zip.len() {
            let entry = zip.by_index_raw(i).map_err(|e| invalid(e.to_string()))?;
            let name = entry.name().map_err(|e| invalid(e.to_string()))?.to_string();
            if name.starts_with('/') || name.split('/').any(|p| p == ".." || p.is_empty()) {
                return Err(invalid(format!("entry {name} leaves the folder")));
            }
            if entry.is_symlink() || entry.is_dir() {
                return Err(invalid(format!("entry {name} is not a file")));
            }
            if !allowed_entry(&name) {
                return Err(invalid(format!("unexpected entry {name}")));
            }
            total = total.saturating_add(entry.size());
            if total > MAX_UNPACKED_BYTES {
                return Err(invalid("the ZIP unpacks to more than 4 GB"));
            }
        }
        if zip.index_for_name(JSON_ENTRY).is_none() {
            return Err(invalid("the ZIP has no export.json"));
        }
        Ok(ImportArchive { zip })
    }

    pub fn export_json(&mut self) -> Result<Vec<u8>, ServiceError> {
        let entry = self.zip.by_name(JSON_ENTRY).map_err(|e| invalid(e.to_string()))?;
        if entry.size() > crate::export_file::MAX_JSON_BYTES {
            return Err(invalid("export.json is larger than 100 MB"));
        }
        let mut bytes = Vec::new();
        entry.take(crate::export_file::MAX_JSON_BYTES + 1).read_to_end(&mut bytes).map_err(|e| invalid(e.to_string()))?;
        Ok(bytes)
    }

    pub fn contains(&self, entry: &str) -> bool {
        self.zip.index_for_name(entry).is_some()
    }

    // doc/interfaces.md#database_handler
    pub fn is_image(&mut self, entry: &str) -> Result<bool, ServiceError> {
        let mut file = self.zip.by_name(entry).map_err(|e| invalid(e.to_string()))?;
        let mut head = [0u8; 8];
        let n = file.read(&mut head).map_err(|e| invalid(e.to_string()))?;
        Ok(matches!(&head[..n], [0xFF, 0xD8, 0xFF, ..] | [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]))
    }

    // doc/interfaces.md#staging-folder
    pub fn extract_image(&mut self, entry: &str, staging_dir: &Path) -> Result<String, ServiceError> {
        let ext = entry.rsplit('.').next().unwrap_or("jpg");
        let name = format!("import_{}.{ext}", Uuid::new_v4());
        let part = staging_dir.join(format!(".{name}.part"));
        let mut src = self.zip.by_name(entry).map_err(|e| invalid(e.to_string()))?;
        let result = File::create(&part).and_then(|mut f| std::io::copy(&mut src, &mut f)).and_then(|_| std::fs::rename(&part, staging_dir.join(&name)));
        if let Err(e) = result {
            let _ = std::fs::remove_file(&part);
            return Err(ServiceError::io(e.to_string()));
        }
        Ok(name)
    }
}
