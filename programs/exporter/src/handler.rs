// doc/programs/exporter.md
// doc/interfaces.md#exporter

use std::path::PathBuf;

use async_trait::async_trait;
use serde_json::Value;
use targetscope_common::bus::{Bus, RequestHandler};
use targetscope_common::clock::unix_s;
use targetscope_common::messages::exporter::{
    DeleteExport, ExportFormat, ExportSessions, ExportSessionsResponse, ExporterRequest, ListExportsResponse,
};
use targetscope_common::messages::ServiceError;
use targetscope_common::settings::SettingsCache;
use uuid::Uuid;

use crate::collect::{collect, ExportData};
use crate::files;
use crate::formats::{csv, image_entries, json, png, zip};

pub struct ExporterHandler {
    pub bus: Bus,
    pub settings: SettingsCache,
}

#[async_trait]
impl RequestHandler<ExporterRequest> for ExporterHandler {
    async fn handle(&self, id: Uuid, req: ExporterRequest) -> Result<Value, ServiceError> {
        let v = match req {
            ExporterRequest::ExportSessions(r) => serde_json::to_value(self.export(id, r).await?),
            ExporterRequest::ListExports(_) => serde_json::to_value(ListExportsResponse { files: files::list(&self.export_dir()?).map_err(io)? }),
            ExporterRequest::DeleteExport(r) => serde_json::to_value(self.delete(r)?),
            ExporterRequest::Ping(_) => Ok(Value::Null),
        };
        Ok(v.expect("serializable"))
    }
}

fn io(e: impl std::fmt::Display) -> ServiceError {
    ServiceError::io(e.to_string())
}

impl ExporterHandler {
    // doc/programs/exporter.md#error-handling
    pub fn export_dir(&self) -> Result<PathBuf, ServiceError> {
        let dir = self.settings.get_path("export_dir").ok_or_else(|| ServiceError::setting_not_set("export_dir"))?;
        std::fs::create_dir_all(&dir).map_err(io)?;
        Ok(dir)
    }

    fn image_dir(&self) -> Result<PathBuf, ServiceError> {
        self.settings.get_path("image_dir").ok_or_else(|| ServiceError::setting_not_set("image_dir"))
    }

    // doc/programs/exporter.md#export-flow-export_sessions
    async fn export(&self, request_id: Uuid, r: ExportSessions) -> Result<ExportSessionsResponse, ServiceError> {
        if r.include_images && r.format == ExportFormat::Png {
            return Err(ServiceError::validation("include_images is only possible for csv and json"));
        }
        let dir = self.export_dir()?;
        let existing = files::existing_files(&dir, &request_id).map_err(io)?;
        if !existing.is_empty() {
            return Ok(ExportSessionsResponse { files: existing });
        }
        let data = collect(&self.bus, &r.filter).await?;
        let now = unix_s(self.bus.clock().now());
        let prefix = files::prefix(now, &request_id);
        let image_dir = if r.include_images { Some(self.image_dir()?) } else { None };
        tokio::task::spawn_blocking(move || write_files(&dir, &prefix, &data, r.format, image_dir.as_deref(), now))
            .await
            .map_err(io)?
            .map(|files| ExportSessionsResponse { files })
    }

    // doc/programs/exporter.md#list_exports-and-delete_export
    fn delete(&self, r: DeleteExport) -> Result<DeleteExport, ServiceError> {
        if !files::is_plain_name(&r.name) || r.name.starts_with('.') {
            return Err(ServiceError::validation("name must be a plain file name"));
        }
        let path = self.export_dir()?.join(&r.name);
        if !path.is_file() {
            return Err(ServiceError::not_found(format!("export {}", r.name)));
        }
        std::fs::remove_file(path).map_err(io)?;
        Ok(r)
    }
}

// doc/programs/exporter.md#formats
pub fn write_files(
    dir: &std::path::Path,
    prefix: &str,
    data: &ExportData,
    format: ExportFormat,
    image_dir: Option<&std::path::Path>,
    now: i64,
) -> Result<Vec<String>, ServiceError> {
    let mut written = Vec::new();
    let mut write = |name: String, f: &mut dyn FnMut(&mut std::fs::File) -> std::io::Result<()>| {
        let result = files::write_atomic(dir, &name, |file| f(file));
        match result {
            Ok(()) => {
                written.push(name);
                Ok(())
            }
            Err(e) => Err(io(e)),
        }
    };
    let result = match (format, image_dir) {
        (ExportFormat::Csv, None) => write(format!("{prefix}_sessions.csv"), &mut |f| csv::write_sessions(f, data, None).map_err(std::io::Error::other))
            .and_then(|_| write(format!("{prefix}_shots.csv"), &mut |f| csv::write_shots(f, data).map_err(std::io::Error::other))),
        (ExportFormat::Json, None) => {
            let file = json::build(data, now, None);
            write(format!("{prefix}.json"), &mut |f| json::write(f, &file).map_err(std::io::Error::other))
        }
        (ExportFormat::Png, _) => {
            let mut result = Ok(());
            for (s, a) in &data.sessions {
                let (Some(t), Some(d)) = (data.target_types.get(&s.target_type_id), data.devices.get(&s.device_id)) else { continue };
                let shooter = s.shooter_id.and_then(|id| data.shooters.get(&id));
                let bytes = png::render(s, a, t, d, shooter).map_err(ServiceError::io)?;
                result = write(format!("{prefix}_session_{}.png", s.id), &mut |f| std::io::Write::write_all(f, &bytes));
                if result.is_err() {
                    break;
                }
            }
            result
        }
        (format, Some(image_dir)) => {
            let entries = image_entries(data, image_dir);
            let (name, file) = match format {
                ExportFormat::Json => (format!("{prefix}_json.zip"), Some(json::build(data, now, Some(&entries)))),
                _ => (format!("{prefix}_csv.zip"), None),
            };
            write(name, &mut |f| {
                let content = match &file {
                    Some(file) => zip::Data::Json(file),
                    None => zip::Data::Csv,
                };
                zip::write(f, data, content, &entries, image_dir)
            })
        }
    };
    if let Err(e) = result {
        for name in &written {
            let _ = std::fs::remove_file(dir.join(name));
        }
        return Err(e);
    }
    Ok(written)
}
