// doc/programs/session_manager.md

pub mod actor;
pub mod analysis;
pub mod events;
pub mod persistence;
pub mod saving;
pub mod state;
pub mod transitions;

use std::process::ExitCode;

use async_trait::async_trait;
use serde_json::Value;
use targetscope_common::bus::RequestHandler;
use targetscope_common::messages::session_manager::{SessionManagerRequest, PROGRAM};
use targetscope_common::messages::ServiceError;
use targetscope_common::program;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::actor::{Input, SessionActor};

pub struct Forward(mpsc::Sender<Input>);

#[async_trait]
impl RequestHandler<SessionManagerRequest> for Forward {
    async fn handle(&self, id: Uuid, req: SessionManagerRequest) -> Result<Value, ServiceError> {
        let (reply, rx) = oneshot::channel();
        self.0.send(Input::Request { id, req, reply }).await.map_err(|_| ServiceError::unavailable("stopping"))?;
        rx.await.map_err(|_| ServiceError::unavailable("stopping"))?
    }
}

// doc/programs/session_manager.md#internal-design
pub async fn run() -> ExitCode {
    let p = program::start(PROGRAM).await;
    let after = p.bus.newest_event_id("database_handler").await.unwrap_or_else(|_| "0-0".into());
    let (tx, rx) = mpsc::channel(256);
    tokio::spawn(SessionActor::new(p.bus.clone(), tx.clone()).run(rx));
    tokio::spawn(events::forward_image_processor_events(p.bus.clone(), tx.clone()));
    tokio::spawn(events::forward_database_events(p.bus.clone(), tx.clone(), after));
    program::stop_on_signal(&p.bus);
    match p.bus.serve(Forward(tx)).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "serve loop ended");
            ExitCode::FAILURE
        }
    }
}
