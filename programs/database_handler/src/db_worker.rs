//! DbWorker (doc/programs/database_handler.md#dbworker): a dedicated thread that owns the only
//! rusqlite::Connection and processes commands one at a time.

use targetscope_common::messages::database_handler::{DataChanged, DatabaseHandlerRequest};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::db::Database;
use crate::error::HandlerError;
use crate::images::ImageStore;

pub struct DbCommand {
    pub request_id: Uuid,
    pub request: DatabaseHandlerRequest,
    pub reply: oneshot::Sender<Result<DbOutcome, HandlerError>>,
}

pub struct DbOutcome {
    pub response: serde_json::Value,
    /// None for reads and duplicates
    pub event: Option<DataChanged>,
}

/// Starts the thread; it owns `db` and `images` and runs until the sender is dropped.
pub fn spawn_db_worker(db: Database, images: ImageStore) -> mpsc::Sender<DbCommand> {
    let _ = (db, images);
    todo!()
}
