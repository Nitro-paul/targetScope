//! analyzer:requests: analyze_shots. Validation: target type as in doc/interfaces.md#targettype,
//! bullet_diameter_mm > 0, every Shot within its ranges; otherwise validation_failed.

use async_trait::async_trait;
use targetscope_common::bus::RequestHandler;
use targetscope_common::messages::analyzer::AnalyzerRequest;
use targetscope_common::messages::ServiceError;
use uuid::Uuid;

pub struct AnalyzerHandler;

#[async_trait]
impl RequestHandler<AnalyzerRequest> for AnalyzerHandler {
    async fn handle(
        &self,
        request_id: Uuid,
        req: AnalyzerRequest,
    ) -> Result<serde_json::Value, ServiceError> {
        let _ = (request_id, req);
        todo!()
    }
}
