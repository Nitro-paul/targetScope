//! Shared state of all handlers.

use std::sync::Arc;

use targetscope_common::bus::Bus;
use targetscope_common::settings::SettingsCache;

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<Inner>,
}

pub struct Inner {
    pub bus: Bus,
    pub settings: SettingsCache,
    pub admin: crate::admin::AdminSessions,
    pub events: tokio::sync::broadcast::Sender<String>,
    // TODO: newest JPEG per MJPEG feed
}
