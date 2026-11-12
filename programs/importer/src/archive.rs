//! ZIP files: only `export.json`, `images/sessions/<uuid>_<paper_no>.<jpg|png>` and
//! `images/devices/<uuid>.<jpg|png>`; no absolute paths, no "..", no links (ZIP slip);
//! size limits against ZIP bombs. Images are extracted to the staging folder as
//! `import_<uuid>.<ext>` right before the record that needs them.

use std::path::Path;

pub const MAX_JSON_BYTES: u64 = 100 * 1024 * 1024;
pub const MAX_ZIP_BYTES: u64 = 2 * 1024 * 1024 * 1024;

pub struct ImportArchive {
    // TODO: zip::ZipArchive<File>
}

impl ImportArchive {
    /// Checks every entry name before anything is read.
    pub fn open(path: &Path) -> Result<Self, String> {
        let _ = path;
        todo!()
    }

    pub fn export_json(&mut self) -> Result<Vec<u8>, String> {
        todo!()
    }

    /// Extracts one image into the staging folder; returns its staged file name.
    pub fn extract_image(&mut self, entry: &str, staging_dir: &Path) -> Result<String, String> {
        let _ = (entry, staging_dir);
        todo!()
    }
}
