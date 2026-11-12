//! SessionActor (doc/programs/session_manager.md#internal-design): all requests and events
//! are handled by one task, one at a time, so the state never races.

use serde_json::Value;
use targetscope_common::bus::{Bus, BusError};
use targetscope_common::idempotency::RedisIdempotencyStore;
use targetscope_common::messages::database_handler::DataChanged;
use targetscope_common::messages::image_processor::ImageProcessorEvent;
use targetscope_common::messages::session_manager::SessionManagerRequest;
use targetscope_common::messages::ServiceError;
use targetscope_common::model::AnalysisResult;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::analysis::AnalysisRunner;
use crate::state::StoredSession;

pub enum Input {
    Request {
        id: Uuid,
        req: SessionManagerRequest,
        reply: oneshot::Sender<Result<Value, ServiceError>>,
    },
    ShotEvent {
        id: Uuid,
        event: ImageProcessorEvent,
        ack: oneshot::Sender<()>,
    },
    DataChanged {
        event: DataChanged,
    },
    Analysis {
        shots_version: u64,
        result: Result<AnalysisResult, BusError>,
    },
}

pub struct SessionActor {
    pub bus: Bus,
    pub idem: RedisIdempotencyStore,
    /// CurrentSession + internal fields
    pub state: StoredSession,
    /// At most one running analysis + one pending
    pub analysis: AnalysisRunner,
}

impl SessionActor {
    pub async fn run(mut self, mut rx: mpsc::Receiver<Input>) {
        while let Some(input) = rx.recv().await {
            let _ = (&mut self, input);
            todo!()
        }
    }

    /// Checks the state table, runs the actions, returns the new state.
    async fn apply(&mut self, req: SessionManagerRequest) -> Result<StoredSession, ServiceError> {
        let _ = req;
        todo!()
    }

    /// MULTI: SET session_manager:current, idempotency record; EXEC; then publish session_changed.
    async fn commit(
        &mut self,
        new: StoredSession,
        id: Uuid,
        response: &Value,
    ) -> Result<(), BusError> {
        let _ = (new, id, response);
        todo!()
    }
}
