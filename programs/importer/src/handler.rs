// doc/programs/importer.md
// doc/interfaces.md#import_file

use async_trait::async_trait;
use serde_json::Value;
use targetscope_common::bus::{Bus, RequestHandler};
use targetscope_common::messages::importer::{ImportFile, ImportFileResponse, ImporterRequest};
use targetscope_common::messages::ServiceError;
use targetscope_common::settings::SettingsCache;
use uuid::Uuid;

use crate::archive::{ImportArchive, MAX_ZIP_BYTES};
use crate::consistency::check;
use crate::export_file::{parse_and_validate, MAX_JSON_BYTES};
use crate::records::Importer;

pub struct ImporterHandler {
    pub bus: Bus,
    pub settings: SettingsCache,
}

#[async_trait]
impl RequestHandler<ImporterRequest> for ImporterHandler {
    async fn handle(&self, _id: Uuid, req: ImporterRequest) -> Result<Value, ServiceError> {
        match req {
            ImporterRequest::ImportFile(r) => Ok(serde_json::to_value(self.import(r).await?).expect("serializable")),
            ImporterRequest::Ping(_) => Ok(Value::Null),
        }
    }
}

fn plain(name: &str) -> bool {
    !name.is_empty() && !name.contains('/') && !name.contains('\\') && name != "." && name != ".."
}

impl ImporterHandler {
    // doc/programs/importer.md#import-flow-import_file
    async fn import(&self, r: ImportFile) -> Result<ImportFileResponse, ServiceError> {
        if !plain(&r.staged_file) {
            return Err(ServiceError::validation("staged_file must be a plain file name"));
        }
        let staging = self.settings.get_path("staging_dir").ok_or_else(|| ServiceError::setting_not_set("staging_dir"))?;
        let path = staging.join(&r.staged_file);
        let meta = std::fs::metadata(&path).map_err(|_| ServiceError::not_found(format!("staged file {}", r.staged_file)))?;
        let zip = r.staged_file.to_lowercase().ends_with(".zip");
        let prepared = (|| {
            if zip {
                if meta.len() > MAX_ZIP_BYTES {
                    return Err(ServiceError::validation("the ZIP is larger than 2 GB"));
                }
                let mut archive = ImportArchive::open(&path)?;
                let file = parse_and_validate(&archive.export_json()?)?;
                let warnings = check(&file, Some(&mut archive))?;
                Ok((file, warnings, Some(archive)))
            } else {
                if meta.len() > MAX_JSON_BYTES {
                    return Err(ServiceError::validation("the JSON file is larger than 100 MB"));
                }
                let bytes = std::fs::read(&path).map_err(|e| ServiceError::io(e.to_string()))?;
                let file = parse_and_validate(&bytes)?;
                let warnings = check(&file, None)?;
                Ok((file, warnings, None))
            }
        })();
        let (file, warnings, archive) = match prepared {
            Ok(p) => p,
            Err(e) => {
                let _ = std::fs::remove_file(&path);
                return Err(e);
            }
        };
        let mut importer = Importer { bus: &self.bus, staging_dir: &staging, archive, extracted: Vec::new() };
        let result = importer.import_all(&file, ImportFileResponse { warnings, ..Default::default() }).await;
        importer.clean_up();
        if result.is_ok() {
            let _ = std::fs::remove_file(&path);
        }
        result
    }
}
