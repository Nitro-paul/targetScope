// doc/programs/session_manager.md#shot_detected-events
// doc/programs/session_manager.md#data_changed-events

use targetscope_common::bus::{Bus, ReadMode};
use targetscope_common::messages::database_handler::DatabaseHandlerEvent;
use targetscope_common::messages::image_processor::ImageProcessorEvent;
use tokio::sync::{mpsc, oneshot};

use crate::actor::Input;

pub async fn forward_image_processor_events(bus: Bus, tx: mpsc::Sender<Input>) {
    let mut reader = bus.events::<ImageProcessorEvent>("image_processor", ReadMode::ConsumerGroup);
    loop {
        let event = match reader.next().await {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "image_processor events failed");
                continue;
            }
        };
        let (ack, done) = oneshot::channel();
        if tx.send(Input::ShotEvent { id: event.envelope.event_id, event: event.event, ack }).await.is_err() {
            return;
        }
        if done.await.is_ok() {
            if let Err(e) = reader.ack(&event.entry_id).await {
                tracing::warn!(error = %e, "event not acknowledged");
            }
        }
    }
}

pub async fn forward_database_events(bus: Bus, tx: mpsc::Sender<Input>, after: String) {
    let mut reader = bus.events::<DatabaseHandlerEvent>("database_handler", ReadMode::After(after));
    loop {
        match reader.next().await {
            Ok(e) => {
                let DatabaseHandlerEvent::DataChanged(event) = e.event;
                if tx.send(Input::DataChanged { event }).await.is_err() {
                    return;
                }
            }
            Err(e) => tracing::warn!(error = %e, "database_handler events failed"),
        }
    }
}
