// doc/programs/database_handler.md#architecture
// doc/programs/database_handler.md#request-processing

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use targetscope_common::bus::{Bus, RequestHandler};
use targetscope_common::messages::database_handler::{
    ChangeImageDirResponse, DataChanged, DatabaseHandlerRequest as R, MovedImages, ResetSettingsResponse,
};
use targetscope_common::messages::ServiceError;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::db_worker::{ensure_writable_dir, DbCommand, DbOutcome, Switch};
use crate::error::{HandlerError, Result};
use crate::image_dir_change::{clean_up, copy_all, same_device, same_folder};
use crate::known_settings::{self, NOT_RESET_BY_DEFAULT};
use crate::validation::validate;

pub struct StreamWorker {
    pub bus: Bus,
    pub db: mpsc::Sender<DbCommand>,
    pub data_dir: PathBuf,
    pub changing_image_dir: AtomicBool,
}

#[async_trait]
impl RequestHandler<R> for StreamWorker {
    async fn handle(&self, request_id: Uuid, req: R) -> std::result::Result<Value, ServiceError> {
        let outcome = self.process(request_id, req).await.map_err(ServiceError::from)?;
        self.publish(&outcome.events).await;
        Ok(outcome.response)
    }
}

impl StreamWorker {
    async fn process(&self, request_id: Uuid, req: R) -> Result<DbOutcome> {
        validate(&req)?;
        match req {
            R::ChangeImageDir(r) => {
                let path = targetscope_common::config::expand_home(&r.path);
                self.change_image_dir(request_id, "change_image_dir", &path, r.move_images).await.map(|(o, report)| {
                    let response = ChangeImageDirResponse { path: path.display().to_string(), images: report };
                    DbOutcome { response: serde_json::to_value(response).unwrap(), events: o.events }
                })
            }
            R::ResetSettings(mut r) => {
                let names = match r.names.take() {
                    Some(names) => {
                        if let Some(bad) = names.iter().find(|n| known_settings::find(n).is_none()) {
                            return Err(HandlerError::validation(format!("unknown setting {bad}")));
                        }
                        names
                    }
                    None => known_settings::KNOWN_SETTINGS
                        .iter()
                        .map(|k| k.name.to_string())
                        .filter(|n| !NOT_RESET_BY_DEFAULT.contains(&n.as_str()))
                        .collect(),
                };
                self.reset_settings(request_id, names, r.move_images).await
            }
            other => self.command(request_id, other).await,
        }
    }

    async fn command(&self, request_id: Uuid, request: R) -> Result<DbOutcome> {
        let (reply, rx) = oneshot::channel();
        self.send(DbCommand::Request { request_id, request, reply }).await?;
        rx.await.map_err(|_| worker_gone())?
    }

    async fn send(&self, cmd: DbCommand) -> Result<()> {
        self.db.send(cmd).await.map_err(|_| worker_gone())
    }

    async fn processed(&self, request_id: Uuid) -> Result<Option<Value>> {
        let (reply, rx) = oneshot::channel();
        self.send(DbCommand::Processed { request_id, reply }).await?;
        rx.await.map_err(|_| worker_gone())?
    }

    // doc/interfaces.md#reset_settings
    async fn reset_settings(&self, request_id: Uuid, names: Vec<String>, move_images: bool) -> Result<DbOutcome> {
        if let Some(stored) = self.processed(request_id).await? {
            return Ok(DbOutcome { response: stored, events: vec![] });
        }
        let mut events = Vec::new();
        let mut images = None;
        if names.iter().any(|n| n == "image_dir") {
            let default = self.data_dir.join("images");
            let (o, report) = self.change_image_dir_unrecorded(&default, move_images).await?;
            events.extend(o.events);
            images = report;
        }
        let request = R::ResetSettings(targetscope_common::messages::database_handler::ResetSettings {
            names: Some(names.clone()),
            move_images,
        });
        let mut o = self.command(request_id, request).await?;
        let mut response: ResetSettingsResponse = serde_json::from_value(o.response).map_err(|e| HandlerError::InvalidMessage(e.to_string()))?;
        response.names = names;
        response.images = images;
        events.append(&mut o.events);
        Ok(DbOutcome { response: serde_json::to_value(response).unwrap(), events })
    }

    async fn change_image_dir_unrecorded(&self, path: &Path, move_images: bool) -> Result<(DbOutcome, Option<MovedImages>)> {
        self.run_image_dir_change(None, path, move_images).await
    }

    // doc/programs/database_handler.md#image-folder-change
    async fn change_image_dir(
        &self,
        request_id: Uuid,
        type_name: &'static str,
        path: &Path,
        move_images: bool,
    ) -> Result<(DbOutcome, Option<MovedImages>)> {
        if let Some(stored) = self.processed(request_id).await? {
            let images = serde_json::from_value::<ChangeImageDirResponse>(stored.clone()).ok().and_then(|r| r.images);
            return Ok((DbOutcome { response: stored, events: vec![] }, images));
        }
        self.run_image_dir_change(Some((request_id, type_name)), path, move_images).await
    }

    async fn run_image_dir_change(
        &self,
        record: Option<(Uuid, &'static str)>,
        path: &Path,
        move_images: bool,
    ) -> Result<(DbOutcome, Option<MovedImages>)> {
        ensure_writable_dir(path)?;
        let (old, staging) = self.folders().await?;
        if same_device(path, &staging) == Some(false) {
            return Err(HandlerError::validation("the image folder must be on the same file system as staging_dir"));
        }
        if self.changing_image_dir.swap(true, Ordering::SeqCst) {
            return Err(HandlerError::InvalidState("an image folder change is already running".into()));
        }
        let result = self.switch(record, &old, path, move_images).await;
        self.changing_image_dir.store(false, Ordering::SeqCst);
        result
    }

    async fn switch(
        &self,
        record: Option<(Uuid, &'static str)>,
        old: &Path,
        new: &Path,
        move_images: bool,
    ) -> Result<(DbOutcome, Option<MovedImages>)> {
        let path = new.display().to_string();
        if !(move_images && !same_folder(old, new)) {
            let images = move_images.then(MovedImages::default);
            let response = json!({ "path": path, "images": images });
            let switch = Switch { request_id: record, new_dir: new.to_path_buf(), copied: vec![], failed: vec![], response };
            return Ok((self.switch_command(switch).await?, images));
        }
        let (o, n) = (old.to_path_buf(), new.to_path_buf());
        let copied = tokio::task::spawn_blocking(move || copy_all(&o, &n)).await.map_err(|e| HandlerError::InvalidState(e.to_string()))?;
        let switch = Switch { request_id: None, new_dir: new.to_path_buf(), copied: copied.copied, failed: copied.failed, response: Value::Null };
        let mut outcome = self.switch_command(switch).await?;
        let (o, n) = (old.to_path_buf(), new.to_path_buf());
        let (moved, failed) =
            tokio::task::spawn_blocking(move || clean_up(&o, &n)).await.map_err(|e| HandlerError::InvalidState(e.to_string()))?;
        let images = MovedImages { moved, failed };
        outcome.response = json!({ "path": path, "images": images });
        if let Some((request_id, type_name)) = record {
            let (reply, rx) = oneshot::channel();
            self.send(DbCommand::Record { request_id, type_name, response: outcome.response.clone(), reply }).await?;
            rx.await.map_err(|_| worker_gone())??;
        }
        Ok((outcome, Some(images)))
    }

    async fn switch_command(&self, switch: Switch) -> Result<DbOutcome> {
        let (reply, rx) = oneshot::channel();
        self.send(DbCommand::SwitchImageDir { switch, reply }).await?;
        rx.await.map_err(|_| worker_gone())?
    }

    async fn folders(&self) -> Result<(PathBuf, PathBuf)> {
        let (reply, rx) = oneshot::channel();
        self.send(DbCommand::Folders { reply }).await?;
        rx.await.map_err(|_| worker_gone())
    }

    // doc/interfaces.md#data_changed
    pub async fn publish(&self, events: &[DataChanged]) {
        for e in events {
            let event = targetscope_common::messages::database_handler::DatabaseHandlerEvent::DataChanged(e.clone());
            if let Err(err) = self.bus.publish(&event).await {
                tracing::warn!(error = %err, "data_changed not sent");
            }
        }
    }
}

fn worker_gone() -> HandlerError {
    HandlerError::InvalidState("database worker stopped".into())
}

pub fn shared(worker: StreamWorker) -> Arc<StreamWorker> {
    Arc::new(worker)
}
