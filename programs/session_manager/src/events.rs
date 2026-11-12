//! Events read by session_manager:
//! - image_processor:events (consumer group, deduplicated by event_id): shot_detected,
//!   paper_replaced, calibration_changed (doc/programs/session_manager.md#shot_detected-events)
//! - database_handler:events (plain read): data_changed for devices and target types

use tokio::sync::mpsc;

use crate::actor::Input;

pub async fn forward_image_processor_events(tx: mpsc::Sender<Input>) {
    let _ = tx;
    todo!()
}

pub async fn forward_database_events(tx: mpsc::Sender<Input>) {
    let _ = tx;
    todo!()
}
