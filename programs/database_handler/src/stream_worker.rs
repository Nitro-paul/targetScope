//! StreamWorker (async): reads requests through targetscope_common::bus, parses and validates,
//! sends a DbCommand to the DbWorker, awaits the result, writes response and event, acknowledges.

use async_trait::async_trait;
use targetscope_common::bus::RequestHandler;
use targetscope_common::messages::database_handler::DatabaseHandlerRequest;
use targetscope_common::messages::ServiceError;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::db_worker::DbCommand;

pub struct StreamWorker {
    pub db: mpsc::Sender<DbCommand>,
}

#[async_trait]
impl RequestHandler<DatabaseHandlerRequest> for StreamWorker {
    async fn handle(
        &self,
        request_id: Uuid,
        req: DatabaseHandlerRequest,
    ) -> Result<serde_json::Value, ServiceError> {
        let _ = (request_id, req);
        todo!()
    }
}
