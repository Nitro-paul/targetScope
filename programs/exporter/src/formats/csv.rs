// doc/programs/exporter.md#csv

use std::io::Write;

use targetscope_common::clock::rfc3339_local;
use targetscope_common::geometry::Point;

use crate::collect::ExportData;
use crate::formats::ImageEntries;

const SESSION_COLUMNS: [&str; 19] = [
    "session_id", "session_uuid", "saved_at", "shooter", "device", "device_category", "caliber", "target_type", "distance_m",
    "single_handed", "supported", "shot_count", "paper_count", "duration_s", "score", "grouping_score",
    "grouping_shift_x_mm", "grouping_shift_y_mm", "notes",
];
const SHOT_COLUMNS: [&str; 8] = ["session_id", "shot_no", "paper_no", "angle_turns", "radius_mm", "x_mm", "y_mm", "score"];

fn writer<W: Write>(out: W) -> csv::Writer<W> {
    csv::WriterBuilder::new().delimiter(b';').from_writer(out)
}

fn opt<T: ToString>(v: &Option<T>) -> String {
    v.as_ref().map(|x| x.to_string()).unwrap_or_default()
}

pub fn write_sessions<W: Write>(out: W, data: &ExportData, entries: Option<&ImageEntries>) -> csv::Result<()> {
    let mut w = writer(out);
    let mut header: Vec<&str> = SESSION_COLUMNS.to_vec();
    if entries.is_some() {
        header.extend(["image_entries", "device_image_entry"]);
    }
    w.write_record(&header)?;
    for (s, a) in &data.sessions {
        let device = data.devices.get(&s.device_id);
        let mut row = vec![
            s.id.to_string(),
            s.uuid.to_string(),
            rfc3339_local(s.saved_at),
            s.shooter_id.and_then(|id| data.shooters.get(&id)).map(|sh| sh.name.clone()).unwrap_or_default(),
            device.map(|d| d.name.clone()).unwrap_or_default(),
            device.and_then(|d| d.category.clone()).unwrap_or_default(),
            device.and_then(|d| d.caliber.clone()).unwrap_or_default(),
            data.target_types.get(&s.target_type_id).map(|t| t.name.clone()).unwrap_or_default(),
            opt(&s.distance_m),
            s.single_handed.to_string(),
            s.supported.to_string(),
            s.shot_count.to_string(),
            s.paper_count.to_string(),
            opt(&s.duration_s),
            s.score.to_string(),
            s.grouping_score.to_string(),
            a.grouping_shift.x_mm.to_string(),
            a.grouping_shift.y_mm.to_string(),
            s.notes.clone().unwrap_or_default(),
        ];
        if let Some(e) = entries {
            let list = e.sessions.get(&s.id).map(|l| l.iter().map(|(_, entry, _)| entry.as_str()).collect::<Vec<_>>().join("|"));
            row.push(list.unwrap_or_default());
            row.push(e.devices.get(&s.device_id).map(|(entry, _)| entry.clone()).unwrap_or_default());
        }
        w.write_record(&row)?;
    }
    w.flush()?;
    Ok(())
}

pub fn write_shots<W: Write>(out: W, data: &ExportData) -> csv::Result<()> {
    let mut w = writer(out);
    w.write_record(SHOT_COLUMNS)?;
    for (s, a) in &data.sessions {
        for (i, shot) in s.shots.iter().enumerate() {
            let p = Point::from(shot.shot());
            w.write_record([
                s.id.to_string(),
                (i + 1).to_string(),
                shot.paper_no.to_string(),
                shot.angle_turns.to_string(),
                shot.radius_mm.to_string(),
                p.x.to_string(),
                p.y.to_string(),
                a.shot_scores.get(i).copied().unwrap_or(0).to_string(),
            ])?;
        }
    }
    w.flush()?;
    Ok(())
}
