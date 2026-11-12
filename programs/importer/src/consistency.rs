// doc/programs/importer.md#import-flow-import_file

use std::collections::HashSet;

use targetscope_common::export::ExportFile;
use targetscope_common::messages::ServiceError;
use targetscope_common::model::{Shot, TargetType};
use targetscope_common::targets::validate_target_type;

use crate::archive::ImportArchive;

pub fn check(file: &ExportFile, mut archive: Option<&mut ImportArchive>) -> Result<Vec<String>, ServiceError> {
    let fail = |m: String| Err(ServiceError::validation(m));
    let mut seen = HashSet::new();
    let all = file.shooters.iter().map(|s| s.uuid).chain(file.target_types.iter().map(|t| t.uuid)).chain(file.devices.iter().map(|d| d.uuid)).chain(file.sessions.iter().map(|s| s.uuid));
    for u in all {
        if !seen.insert(u) {
            return fail(format!("uuid {u} appears twice"));
        }
    }
    let shooters: HashSet<_> = file.shooters.iter().map(|s| s.uuid).collect();
    let devices: HashSet<_> = file.devices.iter().map(|d| d.uuid).collect();
    let targets: HashSet<_> = file.target_types.iter().map(|t| t.uuid).collect();
    for t in &file.target_types {
        let model = TargetType {
            id: None,
            uuid: Some(t.uuid),
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
            rings: vec![],
            locked: false,
            retired: t.retired,
        };
        if let Err(rule) = validate_target_type(&model) {
            return fail(format!("target type {}: {rule}", t.uuid));
        }
    }
    for s in &file.sessions {
        if s.shooter_uuid.is_some_and(|u| !shooters.contains(&u)) {
            return fail(format!("session {}: shooter_uuid is not in the file", s.uuid));
        }
        if !devices.contains(&s.device_uuid) {
            return fail(format!("session {}: device_uuid is not in the file", s.uuid));
        }
        if !targets.contains(&s.target_type_uuid) {
            return fail(format!("session {}: target_type_uuid is not in the file", s.uuid));
        }
        if s.shots.is_empty() || s.shot_count != s.shots.len() as i64 {
            return fail(format!("session {}: shot_count must equal the number of shots (at least 1)", s.uuid));
        }
        for shot in &s.shots {
            if !(Shot { angle_turns: shot.angle_turns, radius_mm: shot.radius_mm }).is_valid() {
                return fail(format!("session {}: 0 <= angle_turns < 1 and radius_mm >= 0", s.uuid));
            }
            if !(1..=s.paper_count).contains(&shot.paper_no) {
                return fail(format!("session {}: paper_no within 1..={}", s.uuid, s.paper_count));
            }
        }
    }
    let mut warnings = Vec::new();
    if let Some(zip) = archive.as_deref_mut() {
        let entries = file
            .sessions
            .iter()
            .flat_map(|s| s.images.iter().map(|i| i.image_entry.clone()))
            .chain(file.devices.iter().filter_map(|d| d.image_entry.clone()));
        for entry in entries {
            if !zip.contains(&entry) {
                warnings.push(format!("image {entry} is missing in the ZIP"));
            } else if !zip.is_image(&entry)? {
                return fail(format!("{entry} is not a JPEG or PNG image"));
            }
        }
    }
    Ok(warnings)
}
