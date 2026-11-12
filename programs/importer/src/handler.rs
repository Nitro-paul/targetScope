//! importer:requests: import_file (doc/interfaces.md#import_file). Afterwards the uploaded file
//! and all extracted images left in the staging folder are deleted.

use async_trait::async_trait;
use targetscope_common::bus::RequestHandler;
use targetscope_common::messages::importer::ImporterRequest;
use targetscope_common::messages::ServiceError;
use uuid::Uuid;

pub struct ImporterHandler {
    // TODO: Bus, SettingsCache (staging_dir)
}

#[async_trait]
impl RequestHandler<ImporterRequest> for ImporterHandler {
    async fn handle(
        &self,
        request_id: Uuid,
        req: ImporterRequest,
    ) -> Result<serde_json::Value, ServiceError> {
        let _ = (request_id, req);
        todo!()
    }
}
