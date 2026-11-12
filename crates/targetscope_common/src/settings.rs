// doc/shared_libraries.md#settings-cache

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use tokio::sync::watch;

use crate::bus::{Bus, BusError, ReadMode};
use crate::config::expand_home;
use crate::failpoints;
use crate::messages::database_handler::{DataChanged, DatabaseHandlerEvent, ReadSettings, Table, PROGRAM};
use crate::model::{Filter, Setting};

#[derive(Clone)]
pub struct SettingsCache {
    inner: Arc<Inner>,
}

struct Inner {
    settings: RwLock<HashMap<String, Setting>>,
    watchers: RwLock<Vec<(Vec<String>, watch::Sender<()>)>>,
}

impl SettingsCache {
    pub async fn load(bus: &Bus) -> Result<Self, BusError> {
        let after = bus.newest_event_id(PROGRAM).await?;
        if failpoints::hit("settings.after_note_id") {
            return Err(BusError::Decode("failpoint settings.after_note_id".into()));
        }
        let all = bus.request(ReadSettings { filter: Filter::default() }).await?.settings;
        let cache = SettingsCache {
            inner: Arc::new(Inner {
                settings: RwLock::new(all.into_iter().map(|s| (s.name.clone(), s)).collect()),
                watchers: RwLock::new(Vec::new()),
            }),
        };
        tokio::spawn(follow(bus.clone(), Arc::downgrade(&cache.inner), after));
        Ok(cache)
    }

    pub fn from_settings(settings: Vec<Setting>) -> Self {
        SettingsCache {
            inner: Arc::new(Inner {
                settings: RwLock::new(settings.into_iter().map(|s| (s.name.clone(), s)).collect()),
                watchers: RwLock::new(Vec::new()),
            }),
        }
    }

    pub fn get(&self, name: &str) -> Option<String> {
        let settings = self.inner.settings.read().unwrap();
        settings.get(name).filter(|s| s.active).map(|s| s.value.clone())
    }

    pub fn get_parsed<T: FromStr>(&self, name: &str) -> Option<T> {
        self.get(name)?.trim().parse().ok()
    }

    pub fn get_path(&self, name: &str) -> Option<PathBuf> {
        self.get(name).filter(|v| !v.is_empty()).map(|v| expand_home(&v))
    }

    pub fn setting(&self, name: &str) -> Option<Setting> {
        self.inner.settings.read().unwrap().get(name).cloned()
    }

    pub fn all(&self) -> Vec<Setting> {
        self.inner.settings.read().unwrap().values().cloned().collect()
    }

    pub fn watch(&self, names: &[&str]) -> watch::Receiver<()> {
        let (tx, rx) = watch::channel(());
        let names = names.iter().map(|n| n.to_string()).collect();
        self.inner.watchers.write().unwrap().push((names, tx));
        rx
    }
}

impl Inner {
    fn replace(&self, name: Option<&str>, fresh: Vec<Setting>) {
        let mut changed = Vec::new();
        {
            let mut settings = self.settings.write().unwrap();
            let mut seen: HashMap<String, Setting> = fresh.into_iter().map(|s| (s.name.clone(), s)).collect();
            let names: Vec<String> = match name {
                Some(n) => vec![n.to_string()],
                None => settings.keys().chain(seen.keys()).cloned().collect(),
            };
            for n in names {
                let new = seen.remove(&n);
                let old = settings.get(&n);
                let effective = |s: Option<&Setting>| s.filter(|s| s.active).map(|s| s.value.clone());
                if effective(old) != effective(new.as_ref()) && !changed.contains(&n) {
                    changed.push(n.clone());
                }
                match new {
                    Some(s) => settings.insert(n, s),
                    None => settings.remove(&n),
                };
            }
        }
        let watchers = self.watchers.read().unwrap();
        for (names, tx) in watchers.iter() {
            if names.iter().any(|n| changed.contains(n)) {
                tx.send_replace(());
            }
        }
    }
}

async fn follow(bus: Bus, inner: std::sync::Weak<Inner>, after: String) {
    let mut reader = bus.events::<DatabaseHandlerEvent>(PROGRAM, ReadMode::After(after));
    loop {
        let event = match reader.next().await {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "settings events failed");
                continue;
            }
        };
        let DatabaseHandlerEvent::DataChanged(DataChanged { table, key, .. }) = event.event;
        let name = match table {
            Table::Settings => key.as_str().map(str::to_string),
            Table::All => None,
            _ => continue,
        };
        let filter = match &name {
            Some(n) => Filter::eq("name", n.as_str()),
            None => Filter::default(),
        };
        let fresh = match bus.request(ReadSettings { filter }).await {
            Ok(r) => r.settings,
            Err(e) => {
                tracing::warn!(error = %e, "reading settings failed");
                continue;
            }
        };
        let Some(inner) = inner.upgrade() else { return };
        inner.replace(name.as_deref(), fresh);
    }
}
