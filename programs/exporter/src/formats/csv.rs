//! CSV: `<prefix>_sessions.csv` and `<prefix>_shots.csv`, semicolon separated, UTF-8 with
//! header row, decimal point; saved_at as ISO 8601 in local time.
//! In a ZIP: extra columns image_entries (separated by "|") and device_image_entry.

use std::io::Write;

use crate::collect::ExportData;

pub fn write_sessions(
    out: impl Write,
    data: &ExportData,
    with_image_entries: bool,
) -> std::io::Result<()> {
    let _ = (out, data, with_image_entries);
    todo!()
}

/// session_id;shot_no;paper_no;angle_turns;radius_mm;x_mm;y_mm;score
pub fn write_shots(out: impl Write, data: &ExportData) -> std::io::Result<()> {
    let _ = (out, data);
    todo!()
}
