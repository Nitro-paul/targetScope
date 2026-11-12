//! `launcher:requests`: launcher_status, restart_program; event program_state_changed.
//! Duplicate protection with RedisIdempotencyStore.

use async_trait::async_trait;
use targetscope_common::bus::RequestHandler;
use targetscope_common::messages::launcher::LauncherRequest;
use targetscope_common::messages::ServiceError;
use uuid::Uuid;

pub struct LauncherRequestHandler {
    // TODO: channel to the main loop
}

#[async_trait]
impl RequestHandler<LauncherRequest> for LauncherRequestHandler {
    async fn handle(
        &self,
        request_id: Uuid,
        req: LauncherRequest,
    ) -> Result<serde_json::Value, ServiceError> {
        let _ = (request_id, req);
        todo!()
    }
}
