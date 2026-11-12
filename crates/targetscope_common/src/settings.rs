//! Settings cache (doc/shared_libraries.md#settings-cache).

use std::path::PathBuf;
use std::str::FromStr;

use crate::bus::{Bus, BusError};

/// Notes the newest id of `database_handler:events`, reads all settings through database_handler,
/// then keeps them up to date by reading `data_changed` events for the settings table after that id.
/// A `data_changed` with table "all" reloads every setting.
pub struct SettingsCache {
    // TODO: map name -> Setting, watchers
}

impl SettingsCache {
    pub async fn load(bus: &Bus) -> Result<Self, BusError> {
        let _ = bus;
        todo!()
    }

    /// None if the setting is unset (active = false) or unknown.
    pub fn get(&self, name: &str) -> Option<String> {
        let _ = name;
        todo!()
    }

    pub fn get_parsed<T: FromStr>(&self, name: &str) -> Option<T> {
        self.get(name)?.parse().ok()
    }

    /// Expands "~".
    pub fn get_path(&self, name: &str) -> Option<PathBuf> {
        let _ = name;
        todo!()
    }

    /// Notifies the program when one of the given settings changes.
    pub fn watch(&self, names: &[&str]) -> tokio::sync::watch::Receiver<()> {
        let _ = names;
        todo!()
    }
}
