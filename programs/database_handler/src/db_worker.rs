// doc/programs/database_handler.md#dbworker
// doc/programs/database_handler.md#request-processing

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rusqlite::{Connection, TransactionBehavior};
use serde_json::{json, Value};
use targetscope_common::clock::{unix_s, Clock};
use targetscope_common::failpoints;
use targetscope_common::messages::database_handler::{
    ChangeAction, CreateSessionResponse, DataChanged, DatabaseHandlerRequest as R, IdResponse, ImportResponse,
    MovedImages, NameResponse, ReadDevicesResponse, ReadSessionsResponse, ReadSettingsResponse,
    ReadShootersResponse, ReadTargetTypesResponse, ResetSettingsResponse, SetDeviceImageResponse, Table,
};
use targetscope_common::model::{NewSession, PaperImage, Setting};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::db::{self, Database};
use crate::error::{HandlerError, Result};
use crate::images::{undo, ImageStore, Moved};
use crate::known_settings::{self, check_value, Rule};

pub struct DbOutcome {
    pub response: Value,
    pub events: Vec<DataChanged>,
}

// doc/programs/database_handler.md#dbworker
pub enum DbCommand {
    Request { request_id: Uuid, request: R, reply: oneshot::Sender<Result<DbOutcome>> },
    Processed { request_id: Uuid, reply: oneshot::Sender<Result<Option<Value>>> },
    Folders { reply: oneshot::Sender<(PathBuf, PathBuf)> },
    SwitchImageDir { switch: Switch, reply: oneshot::Sender<Result<DbOutcome>> },
    Maintenance { reply: oneshot::Sender<Result<usize>> },
    Record { request_id: Uuid, type_name: &'static str, response: Value, reply: oneshot::Sender<Result<()>> },
}

// doc/programs/database_handler.md#image-folder-change
pub struct Switch {
    pub request_id: Option<(Uuid, &'static str)>,
    pub new_dir: PathBuf,
    pub copied: Vec<String>,
    pub failed: Vec<String>,
    pub response: Value,
}

pub struct DbWorker {
    db: Database,
    images: ImageStore,
    data_dir: PathBuf,
    clock: Arc<dyn Clock>,
}

#[derive(Default)]
struct Effects {
    moved: Vec<Moved>,
    delete_after: Vec<String>,
    events: Vec<DataChanged>,
    staging_dir: Option<PathBuf>,
}

fn event(table: Table, key: Value, action: ChangeAction) -> DataChanged {
    DataChanged { table, key, action }
}

fn failpoint_error(name: &str) -> HandlerError {
    HandlerError::Database(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
        Some(format!("failpoint {name}")),
    ))
}

pub fn spawn_db_worker(db: Database, images: ImageStore, data_dir: PathBuf, clock: Arc<dyn Clock>) -> mpsc::Sender<DbCommand> {
    let (tx, mut rx) = mpsc::channel::<DbCommand>(64);
    std::thread::Builder::new()
        .name("db_worker".into())
        .spawn(move || {
            let mut worker = DbWorker { db, images, data_dir, clock };
            worker.log_orphans();
            while let Some(cmd) = rx.blocking_recv() {
                worker.handle(cmd);
            }
        })
        .expect("db worker thread");
    tx
}

impl DbWorker {
    pub fn new(db: Database, images: ImageStore, data_dir: PathBuf, clock: Arc<dyn Clock>) -> Self {
        DbWorker { db, images, data_dir, clock }
    }

    pub fn connection(&self) -> &Connection {
        &self.db.conn
    }

    pub fn images(&self) -> &ImageStore {
        &self.images
    }

    pub fn handle(&mut self, cmd: DbCommand) {
        match cmd {
            DbCommand::Request { request_id, request, reply } => {
                let _ = reply.send(self.execute(request_id, request));
            }
            DbCommand::Processed { request_id, reply } => {
                let _ = reply.send(db::processed_requests::find(&self.db.conn, &request_id));
            }
            DbCommand::Folders { reply } => {
                let _ = reply.send((self.images.image_dir.clone(), self.images.staging_dir.clone()));
            }
            DbCommand::SwitchImageDir { switch, reply } => {
                let _ = reply.send(self.switch_image_dir(switch));
            }
            DbCommand::Maintenance { reply } => {
                let _ = reply.send(self.maintenance());
            }
            DbCommand::Record { request_id, type_name, response, reply } => {
                let now = self.now();
                let _ = reply.send(db::processed_requests::record(&self.db.conn, &request_id, type_name, &response, now));
            }
        }
    }

    fn now(&self) -> i64 {
        unix_s(self.clock.now())
    }

    // doc/programs/database_handler.md#request-processing
    pub fn execute(&mut self, request_id: Uuid, request: R) -> Result<DbOutcome> {
        let read = |v: Value| Ok(DbOutcome { response: v, events: vec![] });
        let c = &self.db.conn;
        match request {
            R::ReadSessions(r) => {
                let (sessions, total) = db::sessions::find(c, &r.filter, r.order_by, r.limit, r.offset, r.include_shots)?;
                read(to_value(ReadSessionsResponse { sessions, total }))
            }
            R::ReadShooters(r) => read(to_value(ReadShootersResponse { shooters: db::shooters::find(c, &r.filter)? })),
            R::ReadDevices(r) => read(to_value(ReadDevicesResponse { devices: db::devices::find(c, &r.filter)? })),
            R::ReadTargetTypes(r) => {
                read(to_value(ReadTargetTypesResponse { target_types: db::target_types::find(c, &r.filter)? }))
            }
            R::ReadSettings(r) => {
                read(to_value(ReadSettingsResponse { settings: db::settings::find(c, &r.filter, &self.data_dir)? }))
            }
            R::Ping(_) => read(json!({})),
            R::ChangeImageDir(_) => Err(HandlerError::InvalidMessage("change_image_dir is handled by the stream worker".into())),
            write => {
                let type_name = request_type(&write);
                self.write(request_id, type_name, |w, tx, fx| w.apply(tx, fx, write))
            }
        }
    }

    // doc/programs/database_handler.md#request-processing
    fn write<F>(&mut self, request_id: Uuid, type_name: &str, f: F) -> Result<DbOutcome>
    where
        F: FnOnce(&WriteCtx, &Connection, &mut Effects) -> Result<Value>,
    {
        let now = self.now();
        let ctx = WriteCtx { images: &self.images, data_dir: &self.data_dir, now };
        let tx = self.db.conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(stored) = db::processed_requests::find(&tx, &request_id)? {
            return Ok(DbOutcome { response: stored, events: vec![] });
        }
        let mut fx = Effects::default();
        let result = f(&ctx, &tx, &mut fx).and_then(|response| {
            db::processed_requests::record(&tx, &request_id, type_name, &response, now)?;
            if failpoints::hit("db.before_commit") {
                return Err(failpoint_error("db.before_commit"));
            }
            tx.commit()?;
            Ok(response)
        });
        match result {
            Ok(response) => {
                failpoints::hit("db.after_commit");
                for path in &fx.delete_after {
                    self.images.delete(path);
                }
                if let Some(dir) = fx.staging_dir.take() {
                    self.images.staging_dir = dir;
                }
                Ok(DbOutcome { response, events: fx.events })
            }
            Err(e) => {
                undo(&self.images, &fx.moved);
                Err(e)
            }
        }
    }

    // doc/programs/database_handler.md#image-folder-change
    fn switch_image_dir(&mut self, s: Switch) -> Result<DbOutcome> {
        let old = self.images.image_dir.clone();
        let now = self.now();
        let tx = self.db.conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some((id, _)) = &s.request_id {
            if let Some(stored) = db::processed_requests::find(&tx, id)? {
                return Ok(DbOutcome { response: stored, events: vec![] });
            }
        }
        if !s.copied.is_empty() || !s.failed.is_empty() {
            crate::image_dir_change::sync(&old, &s.new_dir, &s.copied, &s.failed)?;
        }
        tx.execute(
            "UPDATE settings SET value = ?1, active = 1 WHERE name = 'image_dir'",
            [s.new_dir.display().to_string()],
        )?;
        if let Some((id, type_name)) = &s.request_id {
            db::processed_requests::record(&tx, id, type_name, &s.response, now)?;
        }
        tx.commit()?;
        self.images.set_image_dir(s.new_dir);
        Ok(DbOutcome {
            response: s.response,
            events: vec![event(Table::Settings, json!("image_dir"), ChangeAction::Updated)],
        })
    }

    // doc/programs/database_handler.md#startup
    fn maintenance(&mut self) -> Result<usize> {
        let c = &self.db.conn;
        let days = db::settings::get(c, "processed_requests_retention_days")?.and_then(|v| v.parse().ok()).unwrap_or(30);
        let hours: u64 = db::settings::get(c, "staging_max_age_hours")?.and_then(|v| v.parse().ok()).unwrap_or(24);
        let purged = db::processed_requests::purge(c, days, self.now())?;
        crate::maintenance::clean_staging(&self.images.staging_dir, hours, self.clock.now());
        Ok(purged)
    }

    // doc/programs/database_handler.md#error-handling
    fn log_orphans(&self) {
        let known: std::collections::HashSet<String> = match db::sessions::all_image_paths(&self.db.conn) {
            Ok(p) => p.into_iter().collect(),
            Err(e) => return tracing::warn!(error = %e, "orphan check failed"),
        };
        for sub in ["devices", "sessions"] {
            let Ok(entries) = std::fs::read_dir(self.images.image_dir.join(sub)) else { continue };
            for entry in entries.flatten() {
                let rel = format!("{sub}/{}", entry.file_name().to_string_lossy());
                if !known.contains(&rel) {
                    tracing::warn!(path = %entry.path().display(), "image file without record");
                }
            }
        }
    }
}

struct WriteCtx<'a> {
    images: &'a ImageStore,
    data_dir: &'a Path,
    now: i64,
}

impl WriteCtx<'_> {
    // doc/programs/database_handler.md#image-handling-per-request
    fn move_session_images(&self, uuid: Uuid, s: &NewSession, fx: &mut Effects) -> Result<Vec<(i64, String)>> {
        let mut kinds = Vec::new();
        for img in &s.images {
            kinds.push(self.images.check_staged(&img.staged_file)?);
        }
        let mut paths = Vec::new();
        for (img, kind) in s.images.iter().zip(kinds) {
            let name = format!("{uuid}_{}.{}", img.paper_no, kind.ext());
            let path = self.images.move_in(&img.staged_file, "sessions", &name)?;
            fx.moved.push(Moved { staged: img.staged_file.clone(), image_path: path.clone() });
            paths.push((img.paper_no, path));
        }
        Ok(paths)
    }

    // doc/programs/database_handler.md#image-handling-per-request
    fn move_device_image(&self, device_id: i64, staged: &str, fx: &mut Effects) -> Result<String> {
        let kind = self.images.check_staged(staged)?;
        let name = format!("{device_id}_{}.{}", &Uuid::new_v4().simple().to_string()[..8], kind.ext());
        let path = self.images.move_in(staged, "devices", &name)?;
        fx.moved.push(Moved { staged: staged.to_string(), image_path: path.clone() });
        Ok(path)
    }

    fn apply(&self, c: &Connection, fx: &mut Effects, req: R) -> Result<Value> {
        let ev = |fx: &mut Effects, t: Table, key: Value, a: ChangeAction| fx.events.push(event(t, key, a));
        let created = |b: bool| if b { ChangeAction::Created } else { ChangeAction::Updated };
        Ok(match req {
            // doc/interfaces.md#create_session
            R::CreateSession(r) => {
                let s = r.0;
                db::sessions::check_references(c, &s)?;
                let uuid = Uuid::new_v4();
                let images = self.move_session_images(uuid, &s, fx)?;
                let id = db::sessions::create(c, uuid, self.now, &s, &images)?;
                ev(fx, Table::Sessions, json!(id), ChangeAction::Created);
                ev(fx, Table::Devices, json!(s.device_id), ChangeAction::Updated);
                let images = images.into_iter().map(|(paper_no, image_path)| PaperImage { paper_no, image_path }).collect();
                to_value(CreateSessionResponse { id, uuid, images })
            }
            // doc/interfaces.md#delete_session
            R::DeleteSession(r) => {
                fx.delete_after = db::sessions::delete(c, r.id)?;
                ev(fx, Table::Sessions, json!(r.id), ChangeAction::Deleted);
                to_value(IdResponse { id: r.id })
            }
            R::SaveShooter(r) => {
                let (id, new) = db::shooters::save(c, &r.0)?;
                ev(fx, Table::Shooters, json!(id), created(new));
                to_value(IdResponse { id })
            }
            R::DeleteShooter(r) => {
                db::shooters::delete(c, r.id)?;
                ev(fx, Table::Shooters, json!(r.id), ChangeAction::Deleted);
                to_value(IdResponse { id: r.id })
            }
            R::SaveDevice(r) => {
                let (id, new) = db::devices::save(c, &r.0)?;
                ev(fx, Table::Devices, json!(id), created(new));
                to_value(IdResponse { id })
            }
            R::DeleteDevice(r) => {
                fx.delete_after.extend(db::devices::delete(c, r.id)?);
                ev(fx, Table::Devices, json!(r.id), ChangeAction::Deleted);
                to_value(IdResponse { id: r.id })
            }
            // doc/interfaces.md#set_device_image
            R::SetDeviceImage(r) => {
                if db::devices::get(c, r.device_id)?.is_none() {
                    return Err(HandlerError::not_found(format!("device {}", r.device_id)));
                }
                let path = self.move_device_image(r.device_id, &r.staged_file, fx)?;
                fx.delete_after.extend(db::devices::set_image(c, r.device_id, Some(&path))?);
                ev(fx, Table::Devices, json!(r.device_id), ChangeAction::Updated);
                to_value(SetDeviceImageResponse { image_path: path })
            }
            R::DeleteDeviceImage(r) => {
                fx.delete_after.extend(db::devices::set_image(c, r.device_id, None)?);
                ev(fx, Table::Devices, json!(r.device_id), ChangeAction::Updated);
                to_value(r)
            }
            R::SaveTargetType(r) => {
                let (id, new) = db::target_types::save(c, &r.0)?;
                ev(fx, Table::TargetTypes, json!(id), created(new));
                to_value(IdResponse { id })
            }
            R::DeleteTargetType(r) => {
                db::target_types::delete(c, r.id)?;
                ev(fx, Table::TargetTypes, json!(r.id), ChangeAction::Deleted);
                to_value(IdResponse { id: r.id })
            }
            // doc/interfaces.md#import
            R::ImportShooter(r) => {
                let (id, new) = db::shooters::import(c, &r.0)?;
                if new {
                    ev(fx, Table::Shooters, json!(id), ChangeAction::Created);
                }
                to_value(ImportResponse { id, created: new })
            }
            R::ImportTargetType(r) => {
                let (id, new) = db::target_types::import(c, &r.0)?;
                if new {
                    ev(fx, Table::TargetTypes, json!(id), ChangeAction::Created);
                }
                to_value(ImportResponse { id, created: new })
            }
            R::ImportDevice(r) => {
                let (id, new) = db::devices::import(c, &r.device)?;
                if new {
                    if let Some(staged) = &r.staged_file {
                        let path = self.move_device_image(id, staged, fx)?;
                        db::devices::set_image(c, id, Some(&path))?;
                    }
                    ev(fx, Table::Devices, json!(id), ChangeAction::Created);
                }
                to_value(ImportResponse { id, created: new })
            }
            R::ImportSession(r) => {
                let s = r.0;
                if let Some(id) = db::sessions::exists(c, s.uuid)? {
                    return Ok(to_value(ImportResponse { id, created: false }));
                }
                db::sessions::check_references(c, &s.session)?;
                let images = self.move_session_images(s.uuid, &s.session, fx)?;
                let id = db::sessions::insert(c, s.uuid, s.saved_at, &s.session, &images)?;
                ev(fx, Table::Sessions, json!(id), ChangeAction::Created);
                to_value(ImportResponse { id, created: true })
            }
            // doc/interfaces.md#save_setting
            R::SaveSetting(r) => {
                let s = self.check_setting(c, r.0)?;
                let new = db::settings::save(c, &s)?;
                if s.name == "staging_dir" {
                    fx.staging_dir = Some(targetscope_common::config::expand_home(&s.value));
                }
                ev(fx, Table::Settings, json!(s.name), created(new));
                to_value(NameResponse { name: s.name })
            }
            R::DeleteSetting(r) => {
                db::settings::delete(c, &r.name)?;
                ev(fx, Table::Settings, json!(r.name), ChangeAction::Deleted);
                to_value(NameResponse { name: r.name })
            }
            // doc/interfaces.md#reset_settings
            R::ResetSettings(r) => {
                let names: Vec<String> = r.names.unwrap_or_default().into_iter().filter(|n| n != "image_dir").collect();
                let refs: Vec<&str> = names.iter().map(String::as_str).collect();
                db::settings::reset(c, &refs, self.data_dir)?;
                if refs.contains(&"staging_dir") {
                    if let Some(v) = db::settings::get(c, "staging_dir")? {
                        fx.staging_dir = Some(targetscope_common::config::expand_home(&v));
                    }
                }
                for n in &names {
                    ev(fx, Table::Settings, json!(n), ChangeAction::Updated);
                }
                to_value(ResetSettingsResponse { names, images: None::<MovedImages> })
            }
            other => return Err(HandlerError::InvalidMessage(format!("{} is not a write", request_type(&other)))),
        })
    }

    // doc/programs/database_handler.md#validation-rules
    fn check_setting(&self, c: &Connection, mut s: Setting) -> Result<Setting> {
        s.name = s.name.trim().to_string();
        let Some(k) = known_settings::find(&s.name) else {
            if !s.active {
                return Err(HandlerError::validation("only optional settings can be inactive"));
            }
            return Ok(s);
        };
        if !s.active && !k.optional {
            return Err(HandlerError::validation(format!("{} can't be unset", s.name)));
        }
        if s.active || !s.value.is_empty() {
            check_value(k.rule, &s.value).map_err(HandlerError::ValidationFailed)?;
        }
        if s.active && matches!(k.rule, Rule::Folder | Rule::StagingFolder) {
            let dir = targetscope_common::config::expand_home(&s.value);
            ensure_writable_dir(&dir)?;
            if k.rule == Rule::StagingFolder {
                let image_dir = db::settings::get(c, "image_dir")?.map(PathBuf::from).unwrap_or_else(|| self.images.image_dir.clone());
                if crate::image_dir_change::same_device(&dir, &image_dir) == Some(false) {
                    return Err(HandlerError::validation("staging_dir must be on the same file system as image_dir"));
                }
            }
        }
        Ok(s)
    }
}

// doc/programs/database_handler.md#validation-rules
pub fn ensure_writable_dir(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir).map_err(|e| HandlerError::validation(format!("{} can't be created: {e}", dir.display())))?;
    let probe = dir.join(format!(".targetscope_write_test_{}", Uuid::new_v4().simple()));
    std::fs::write(&probe, b"")
        .and_then(|_| std::fs::remove_file(&probe))
        .map_err(|e| HandlerError::validation(format!("{} is not writable: {e}", dir.display())))
}

fn to_value<T: serde::Serialize>(v: T) -> Value {
    serde_json::to_value(v).expect("serializable response")
}

pub fn request_type(r: &R) -> &'static str {
    match r {
        R::Ping(_) => "ping",
        R::CreateSession(_) => "create_session",
        R::ReadSessions(_) => "read_sessions",
        R::DeleteSession(_) => "delete_session",
        R::SaveShooter(_) => "save_shooter",
        R::ReadShooters(_) => "read_shooters",
        R::DeleteShooter(_) => "delete_shooter",
        R::SaveDevice(_) => "save_device",
        R::ReadDevices(_) => "read_devices",
        R::DeleteDevice(_) => "delete_device",
        R::SetDeviceImage(_) => "set_device_image",
        R::DeleteDeviceImage(_) => "delete_device_image",
        R::SaveTargetType(_) => "save_target_type",
        R::ReadTargetTypes(_) => "read_target_types",
        R::DeleteTargetType(_) => "delete_target_type",
        R::ImportShooter(_) => "import_shooter",
        R::ImportTargetType(_) => "import_target_type",
        R::ImportDevice(_) => "import_device",
        R::ImportSession(_) => "import_session",
        R::SaveSetting(_) => "save_setting",
        R::ReadSettings(_) => "read_settings",
        R::DeleteSetting(_) => "delete_setting",
        R::ResetSettings(_) => "reset_settings",
        R::ChangeImageDir(_) => "change_image_dir",
    }
}
