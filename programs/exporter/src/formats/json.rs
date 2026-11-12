//! JSON export `<prefix>.json` (schemas/export/v1.schema.json): format "targetscope-export",
//! format_version 1; records reference each other by uuid; only used shooters, types, devices.

use std::io::Write;

use crate::collect::ExportData;

pub const FORMAT: &str = "targetscope-export";
pub const FORMAT_VERSION: u32 = 1;

/// with_image_entries: add images[].image_entry on sessions and image_entry on devices (ZIP only).
pub fn write(
    out: impl Write,
    data: &ExportData,
    exported_at: time::OffsetDateTime,
    with_image_entries: bool,
) -> std::io::Result<()> {
    let _ = (out, data, exported_at, with_image_entries);
    todo!()
}
