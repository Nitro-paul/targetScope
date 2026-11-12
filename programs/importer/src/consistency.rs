//! Step 3: references exist in the file, uuids unique, target types and sessions valid,
//! ZIP image entries exist and are JPEG/PNG (missing image = warning only).

use crate::export_file::ExportFile;

pub struct Checked {
    pub warnings: Vec<String>,
}

pub fn check(file: &ExportFile, image_entries: Option<&[String]>) -> Result<Checked, String> {
    let _ = (file, image_entries);
    todo!()
}
