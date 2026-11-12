// doc/programs/exporter.md#formats

pub mod csv;
pub mod json;
pub mod png;
pub mod zip;

use std::collections::HashMap;
use std::path::Path;

use targetscope_common::export::{device_image_entry, session_image_entry};

use crate::collect::ExportData;

// doc/programs/exporter.md#zip-with-images
pub struct ImageEntries {
    pub sessions: HashMap<i64, Vec<(i64, String, String)>>,
    pub devices: HashMap<i64, (String, String)>,
}

fn ext(image_path: &str) -> &str {
    Path::new(image_path).extension().and_then(|e| e.to_str()).unwrap_or("jpg")
}

// doc/programs/exporter.md#zip-with-images
pub fn image_entries(data: &ExportData, image_dir: &Path) -> ImageEntries {
    let exists = |p: &str| {
        let ok = image_dir.join(p).is_file();
        if !ok {
            tracing::warn!(image_path = p, "image file missing, left out of the export");
        }
        ok
    };
    let mut sessions = HashMap::new();
    for (s, _) in &data.sessions {
        let list: Vec<(i64, String, String)> = s
            .images
            .iter()
            .filter(|i| exists(&i.image_path))
            .map(|i| (i.paper_no, session_image_entry(&s.uuid, i.paper_no, ext(&i.image_path)), i.image_path.clone()))
            .collect();
        sessions.insert(s.id, list);
    }
    let mut devices = HashMap::new();
    for (id, d) in &data.devices {
        if let (Some(path), Some(uuid)) = (&d.image_path, d.uuid) {
            if exists(path) {
                devices.insert(*id, (device_image_entry(&uuid, ext(path)), path.clone()));
            }
        }
    }
    ImageEntries { sessions, devices }
}
