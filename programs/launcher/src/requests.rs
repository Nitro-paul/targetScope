// doc/interfaces.md#launcher

use async_trait::async_trait;
use serde_json::Value;
use targetscope_common::bus::RequestHandler;
use targetscope_common::messages::launcher::LauncherRequest;
use targetscope_common::messages::ServiceError;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::launcher::Event;

pub struct LauncherRequestHandler {
    pub tx: mpsc::Sender<Event>,
}

#[async_trait]
impl RequestHandler<LauncherRequest> for LauncherRequestHandler {
    async fn handle(&self, id: Uuid, req: LauncherRequest) -> Result<Value, ServiceError> {
        let (reply, rx) = oneshot::channel();
        self.tx.send(Event::Request { id, req, reply }).await.map_err(|_| ServiceError::invalid_state("stopping"))?;
        rx.await.map_err(|_| ServiceError::invalid_state("stopping"))?
    }
}
