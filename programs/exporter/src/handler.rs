//! exporter:requests: export_sessions, list_exports, delete_export.
//! setting_not_set (details.setting = "export_dir") if the export folder isn't set.

use async_trait::async_trait;
use targetscope_common::bus::RequestHandler;
use targetscope_common::messages::exporter::ExporterRequest;
use targetscope_common::messages::ServiceError;
use uuid::Uuid;

pub struct ExporterHandler {
    // TODO: Bus, SettingsCache (export_dir, image_dir)
}

#[async_trait]
impl RequestHandler<ExporterRequest> for ExporterHandler {
    async fn handle(
        &self,
        request_id: Uuid,
        req: ExporterRequest,
    ) -> Result<serde_json::Value, ServiceError> {
        let _ = (request_id, req);
        todo!()
    }
}
