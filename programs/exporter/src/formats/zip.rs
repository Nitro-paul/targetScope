//! ZIP with images `<prefix>_json.zip` / `<prefix>_csv.zip`: data files plus
//! images/sessions/<session uuid>_<paper_no>.<ext> and images/devices/<device uuid>.<ext>.
//! Data files deflated; JPEG/PNG stored without compression. Missing images are skipped and logged.

use std::io::{Seek, Write};
use std::path::Path;

use crate::collect::ExportData;
use crate::formats::Format;

pub fn write(
    out: impl Write + Seek,
    data: &ExportData,
    format: Format,
    image_dir: &Path,
) -> std::io::Result<()> {
    let _ = (out, data, format, image_dir);
    todo!()
}
