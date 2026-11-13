// doc/programs/database_handler.md#validation-rules

use std::collections::HashSet;

use targetscope_common::messages::database_handler::DatabaseHandlerRequest as R;
use targetscope_common::model::{Device, NewSession, Shooter, TargetType};
use targetscope_common::targets::validate_target_type;

use crate::error::{HandlerError, Result};

pub const MAX_NOTES: usize = 10_000;

pub fn validate(req: &R) -> Result<()> {
    match req {
        R::CreateSession(r) => validate_new_session(&r.0),
        R::ImportSession(r) => validate_new_session(&r.0.session),
        R::SaveShooter(r) => validate_shooter(&r.0),
        R::ImportShooter(r) => validate_shooter(&r.0),
        R::SaveDevice(r) => validate_device(&r.0),
        R::ImportDevice(r) => {
            validate_device(&r.device)?;
            if r.device.shot_count < 0 {
                return Err(HandlerError::validation("shot_count >= 0"));
            }
            Ok(())
        }
        R::SaveTargetType(r) => validate_target(&r.0),
        R::ImportTargetType(r) => validate_target(&r.0),
        R::SaveSetting(r) => {
            if r.0.name.trim().is_empty() || r.0.display_name.trim().is_empty() {
                return Err(HandlerError::validation("name and display_name are required"));
            }
            if r.0.name == "image_dir" {
                return Err(HandlerError::validation("image_dir is changed with change_image_dir"));
            }
            Ok(())
        }
        R::ChangeImageDir(r) if r.path.trim().is_empty() => Err(HandlerError::validation("path is required")),
        _ => Ok(()),
    }
}

// doc/programs/database_handler.md#validation-rules
pub fn validate_new_session(s: &NewSession) -> Result<()> {
    let fail = |m: &str| Err(HandlerError::validation(m));
    if s.shots.is_empty() {
        return fail("at least one shot");
    }
    if s.shot_count != s.shots.len() as i64 {
        return fail("shot_count must equal the number of shots");
    }
    if s.paper_count < 1 {
        return fail("paper_count >= 1");
    }
    let papers = 1..=s.paper_count;
    for (i, shot) in s.shots.iter().enumerate() {
        if !papers.contains(&shot.paper_no) {
            return Err(HandlerError::validation(format!("shot {i}: paper_no within 1..={}", s.paper_count)));
        }
        if !shot.shot().is_valid() {
            return Err(HandlerError::validation(format!("shot {i}: 0 <= angle_turns < 1 and radius_mm >= 0")));
        }
    }
    let mut seen = HashSet::new();
    for img in &s.images {
        if !papers.contains(&img.paper_no) {
            return fail("image paper_no within 1..=paper_count");
        }
        if !seen.insert(img.paper_no) {
            return fail("at most one image per paper");
        }
    }
    if s.notes.as_ref().is_some_and(|n| n.chars().count() > MAX_NOTES) {
        return fail("notes at most 10 000 characters");
    }
    if s.distance_m.is_some_and(|d| d < 0) || s.duration_s.is_some_and(|d| d < 0) {
        return fail("distance_m and duration_s >= 0");
    }
    Ok(())
}

pub fn validate_shooter(s: &Shooter) -> Result<()> {
    if s.name.trim().is_empty() {
        return Err(HandlerError::validation("name is required"));
    }
    Ok(())
}

pub fn validate_device(d: &Device) -> Result<()> {
    if d.name.trim().is_empty() {
        return Err(HandlerError::validation("name is required"));
    }
    if !(d.bullet_diameter_mm.is_finite() && d.bullet_diameter_mm > 0.0) {
        return Err(HandlerError::validation("bullet_diameter_mm > 0"));
    }
    Ok(())
}

pub fn validate_target(t: &TargetType) -> Result<()> {
    if t.name.trim().is_empty() {
        return Err(HandlerError::validation("name is required"));
    }
    validate_target_type(t).map_err(HandlerError::ValidationFailed)
}
