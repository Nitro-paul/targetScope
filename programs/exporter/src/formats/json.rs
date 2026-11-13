// doc/programs/exporter.md#json

use std::collections::BTreeSet;
use std::io::Write;

use targetscope_common::clock::rfc3339_local;
use targetscope_common::export::{
    ExportDevice, ExportFile, ExportImage, ExportSession, ExportShooter, ExportTargetType, FORMAT, FORMAT_VERSION,
};

use crate::collect::ExportData;
use crate::formats::ImageEntries;

pub fn build(data: &ExportData, exported_at_s: i64, entries: Option<&ImageEntries>) -> ExportFile {
    let used = |f: &dyn Fn(&targetscope_common::model::Session) -> Option<i64>| {
        data.sessions.iter().filter_map(|(s, _)| f(s)).collect::<BTreeSet<i64>>()
    };
    let shooters = used(&|s| s.shooter_id)
        .into_iter()
        .filter_map(|id| data.shooters.get(&id))
        .filter_map(|s| Some(ExportShooter { uuid: s.uuid?, name: s.name.clone(), retired: s.retired }))
        .collect();
    let target_types = used(&|s| Some(s.target_type_id))
        .into_iter()
        .filter_map(|id| data.target_types.get(&id))
        .filter_map(|t| {
            Some(ExportTargetType {
                uuid: t.uuid?,
                name: t.name.clone(),
                paper_width_mm: t.paper_width_mm,
                paper_height_mm: t.paper_height_mm,
                center_x_mm: t.center_x_mm,
                center_y_mm: t.center_y_mm,
                inner_white_radius_mm: t.inner_white_radius_mm,
                inner_white_score: t.inner_white_score,
                black_radius_mm: t.black_radius_mm,
                black_score: t.black_score,
                outer_radius_mm: t.outer_radius_mm,
                outer_score: t.outer_score,
                retired: t.retired,
            })
        })
        .collect();
    let devices = used(&|s| Some(s.device_id))
        .into_iter()
        .filter_map(|id| data.devices.get(&id).map(|d| (id, d)))
        .filter_map(|(id, d)| {
            Some(ExportDevice {
                uuid: d.uuid?,
                name: d.name.clone(),
                category: d.category.clone(),
                description: d.description.clone(),
                caliber: d.caliber.clone(),
                bullet_diameter_mm: d.bullet_diameter_mm,
                shot_count: d.shot_count,
                retired: d.retired,
                image_entry: entries.and_then(|e| e.devices.get(&id)).map(|(entry, _)| entry.clone()),
            })
        })
        .collect();
    let sessions = data
        .sessions
        .iter()
        .map(|(s, a)| ExportSession {
            uuid: s.uuid,
            saved_at: s.saved_at,
            score: s.score,
            grouping_score: s.grouping_score,
            distance_m: s.distance_m,
            shooter_uuid: s.shooter_id.and_then(|id| data.shooters.get(&id)).and_then(|x| x.uuid),
            device_uuid: data.devices.get(&s.device_id).and_then(|d| d.uuid).unwrap_or_default(),
            target_type_uuid: data.target_types.get(&s.target_type_id).and_then(|t| t.uuid).unwrap_or_default(),
            single_handed: s.single_handed,
            supported: s.supported,
            shot_count: s.shot_count,
            duration_s: s.duration_s,
            notes: s.notes.clone(),
            paper_count: s.paper_count,
            shots: s.shots.clone(),
            shot_scores: a.shot_scores.clone(),
            images: entries
                .and_then(|e| e.sessions.get(&s.id))
                .map(|l| l.iter().map(|(paper_no, entry, _)| ExportImage { paper_no: *paper_no, image_entry: entry.clone() }).collect())
                .unwrap_or_default(),
        })
        .collect();
    ExportFile {
        format: FORMAT.into(),
        format_version: FORMAT_VERSION,
        exported_at: rfc3339_local(exported_at_s),
        shooters,
        target_types,
        devices,
        sessions,
    }
}

pub fn write<W: Write>(out: W, file: &ExportFile) -> serde_json::Result<()> {
    serde_json::to_writer_pretty(out, file)
}
