// doc/programs/exporter.md#zip-with-images

use std::io::{Seek, Write};
use std::path::Path;

use targetscope_common::export::{ExportFile, JSON_ENTRY};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::collect::ExportData;
use crate::formats::{csv, ImageEntries};

pub enum Data<'a> {
    Csv,
    Json(&'a ExportFile),
}

fn io(e: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::other(e.to_string())
}

pub fn write<W: Write + Seek>(out: W, data: &ExportData, content: Data, entries: &ImageEntries, image_dir: &Path) -> std::io::Result<()> {
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let mut zip = ZipWriter::new(out);
    match content {
        Data::Json(file) => {
            zip.start_file(JSON_ENTRY, deflated).map_err(io)?;
            serde_json::to_writer_pretty(&mut zip, file).map_err(io)?;
        }
        Data::Csv => {
            zip.start_file("sessions.csv", deflated).map_err(io)?;
            csv::write_sessions(&mut zip, data, Some(entries)).map_err(io)?;
            zip.start_file("shots.csv", deflated).map_err(io)?;
            csv::write_shots(&mut zip, data).map_err(io)?;
        }
    }
    let images = entries.sessions.values().flatten().map(|(_, e, p)| (e, p)).chain(entries.devices.values().map(|(e, p)| (e, p)));
    for (entry, path) in images {
        match std::fs::read(image_dir.join(path)) {
            Ok(bytes) => {
                zip.start_file(entry.as_str(), stored).map_err(io)?;
                zip.write_all(&bytes)?;
            }
            Err(e) => tracing::warn!(image_path = %path, error = %e, "image could not be read, left out"),
        }
    }
    zip.finish().map_err(io)?;
    Ok(())
}
