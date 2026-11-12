// doc/programs/web_server.md

use std::path::PathBuf;
use std::sync::Arc;

use targetscope_common::bus::Bus;
use targetscope_common::settings::SettingsCache;

use crate::admin::AdminSessions;
use crate::mjpeg::Feeds;

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<Inner>,
}

pub struct Inner {
    pub bus: Bus,
    pub settings: SettingsCache,
    pub admin: AdminSessions,
    pub events: tokio::sync::broadcast::Sender<Arc<str>>,
    pub feeds: Feeds,
    pub static_dir: PathBuf,
}

impl std::ops::Deref for AppState {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.inner
    }
}

impl AppState {
    pub fn folder(&self, name: &str) -> Option<PathBuf> {
        self.settings.get_path(name)
    }
}
