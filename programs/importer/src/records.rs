// doc/programs/importer.md#import-flow-import_file
// doc/programs/importer.md#conflicts

use std::collections::HashMap;
use std::path::Path;

use targetscope_common::bus::Bus;
use targetscope_common::export::ExportFile;
use targetscope_common::failpoints;
use targetscope_common::messages::database_handler::{
    ImportDevice, ImportResponse, ImportSession, ImportShooter, ImportTargetType, ReadTargetTypes,
};
use targetscope_common::messages::importer::{ImportCounts, ImportFileResponse};
use targetscope_common::messages::ServiceError;
use targetscope_common::model::{Device, Filter, ImportedSession, NewSession, Shooter, StagedPaperImage, TargetType};

use crate::archive::ImportArchive;

pub struct Importer<'a> {
    pub bus: &'a Bus,
    pub staging_dir: &'a Path,
    pub archive: Option<ImportArchive>,
    pub extracted: Vec<String>,
}

fn count(c: &mut ImportCounts, r: &ImportResponse) {
    if r.created {
        c.created += 1;
    } else {
        c.existing += 1;
    }
}

fn after_record() -> Result<(), ServiceError> {
    if failpoints::hit("importer.after_record") {
        return Err(ServiceError::new(targetscope_common::messages::ErrorCode::Timeout, "failpoint importer.after_record"));
    }
    Ok(())
}

impl Importer<'_> {
    fn extract(&mut self, entry: &str) -> Result<Option<String>, ServiceError> {
        let Some(zip) = self.archive.as_mut() else { return Ok(None) };
        if !zip.contains(entry) {
            return Ok(None);
        }
        let name = zip.extract_image(entry, self.staging_dir)?;
        self.extracted.push(name.clone());
        Ok(Some(name))
    }

    fn staged_exists(&self, name: &str) -> bool {
        self.staging_dir.join(name).exists()
    }

    // doc/programs/importer.md#import-flow-import_file
    pub async fn import_all(&mut self, file: &ExportFile, mut result: ImportFileResponse) -> Result<ImportFileResponse, ServiceError> {
        let mut shooters = HashMap::new();
        for s in &file.shooters {
            let r =
                self.bus.request(ImportShooter(Shooter { id: None, uuid: Some(s.uuid), name: s.name.clone(), retired: s.retired })).await?;
            count(&mut result.shooters, &r);
            shooters.insert(s.uuid, r.id);
            after_record()?;
        }
        let mut targets = HashMap::new();
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
            let r = self.bus.request(ImportTargetType(model)).await?;
            count(&mut result.target_types, &r);
            if r.created {
                self.warn_if_renamed(r.id, &t.name, &mut result.warnings).await;
            }
            targets.insert(t.uuid, r.id);
            after_record()?;
        }
        let mut devices = HashMap::new();
        for d in &file.devices {
            let staged = match &d.image_entry {
                Some(entry) => self.extract(entry)?,
                None => None,
            };
            let device = Device {
                id: None,
                uuid: Some(d.uuid),
                name: d.name.clone(),
                category: d.category.clone(),
                description: d.description.clone(),
                caliber: d.caliber.clone(),
                bullet_diameter_mm: d.bullet_diameter_mm,
                image_path: None,
                shot_count: d.shot_count,
                locked: false,
                retired: d.retired,
            };
            let r = self.bus.request(ImportDevice { device, staged_file: staged.clone() }).await?;
            count(&mut result.devices, &r);
            if staged.as_deref().is_some_and(|s| !self.staged_exists(s)) {
                result.images.restored += 1;
            }
            devices.insert(d.uuid, r.id);
            after_record()?;
        }
        for s in &file.sessions {
            let mut images = Vec::new();
            for img in &s.images {
                if let Some(staged_file) = self.extract(&img.image_entry)? {
                    images.push(StagedPaperImage { paper_no: img.paper_no, staged_file });
                }
            }
            let session = ImportedSession {
                uuid: s.uuid,
                saved_at: s.saved_at,
                session: NewSession {
                    score: s.score,
                    grouping_score: s.grouping_score,
                    distance_m: s.distance_m,
                    target_type_id: targets[&s.target_type_uuid],
                    shooter_id: s.shooter_uuid.map(|u| shooters[&u]),
                    device_id: devices[&s.device_uuid],
                    single_handed: s.single_handed,
                    supported: s.supported,
                    shot_count: s.shot_count,
                    duration_s: s.duration_s,
                    notes: s.notes.clone(),
                    paper_count: s.paper_count,
                    shots: s.shots.clone(),
                    images: images.clone(),
                },
            };
            let r = self.bus.request(ImportSession(session)).await?;
            count(&mut result.sessions, &r);
            result.images.restored += images.iter().filter(|i| !self.staged_exists(&i.staged_file)).count() as i64;
            after_record()?;
        }
        Ok(result)
    }

    // doc/programs/importer.md#conflicts
    async fn warn_if_renamed(&self, id: i64, name: &str, warnings: &mut Vec<String>) {
        if let Ok(r) = self.bus.request(ReadTargetTypes { filter: Filter::eq("id", id) }).await {
            if let Some(t) = r.target_types.first().filter(|t| t.name != name) {
                warnings.push(format!("target type {name} exists already and was imported as {}", t.name));
            }
        }
    }

    // doc/programs/importer.md#import-flow-import_file
    pub fn clean_up(&self) {
        for name in &self.extracted {
            let _ = std::fs::remove_file(self.staging_dir.join(name));
        }
    }
}
