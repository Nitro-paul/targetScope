//! statistics:requests: compute_statistics (doc/programs/statistics.md#calculation-flow-compute_statistics).

use async_trait::async_trait;
use targetscope_common::bus::RequestHandler;
use targetscope_common::messages::statistics::StatisticsRequest;
use targetscope_common::messages::ServiceError;
use uuid::Uuid;

pub struct StatisticsHandler {
    // TODO: Bus for read_sessions / read_devices
}

#[async_trait]
impl RequestHandler<StatisticsRequest> for StatisticsHandler {
    async fn handle(
        &self,
        request_id: Uuid,
        req: StatisticsRequest,
    ) -> Result<serde_json::Value, ServiceError> {
        let _ = (request_id, req);
        todo!()
    }
}
