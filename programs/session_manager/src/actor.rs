// doc/programs/session_manager.md#internal-design

use std::sync::Arc;
use std::time::Duration;

use redis::aio::ConnectionManager;
use serde_json::{json, Value};
use targetscope_common::bus::{Bus, BusError};
use targetscope_common::clock::{unix_ms, unix_s, Clock};
use targetscope_common::idempotency::RedisIdempotencyStore;
use targetscope_common::messages::analyzer::AnalyzeShots;
use targetscope_common::messages::database_handler::{
    ChangeAction, DataChanged, ReadDevices, ReadShooters, ReadTargetTypes, Table,
};
use targetscope_common::messages::image_processor::{
    CaptureTargetImage, DetectShotsInPhoto, ImageProcessorEvent, LiveCalibrate, LiveNewPaper, LiveSetCounting, LiveStart,
    LiveStop, LiveUncountHole, PaperReplaced, ShotDetected,
};
use targetscope_common::messages::session_manager::{
    SessionManagerEvent, SessionManagerRequest as R, SessionResponse, SessionSaveResponse, SessionSaved,
};
use targetscope_common::messages::ServiceError;
use targetscope_common::model::{
    AnalysisResult, CalibrationState, CurrentShot, Device, Filter, InputMode, SessionState, Shot, TargetType,
};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::analysis::AnalysisRunner;
use crate::persistence;
use crate::saving;
use crate::state::StoredSession;
use crate::transitions::{changes_state, is_allowed};

// doc/interfaces.md#session_set_notes
pub const MAX_NOTES: usize = 10_000;
// doc/programs/session_manager.md#persistence-and-recovery
pub const ANALYSIS_RETRY: Duration = Duration::from_secs(5);

pub enum Input {
    Request { id: Uuid, req: R, reply: oneshot::Sender<Result<Value, ServiceError>> },
    ShotEvent { id: Uuid, event: ImageProcessorEvent, ack: oneshot::Sender<()> },
    DataChanged { event: DataChanged },
    Analysis { shots_version: u64, result: Result<AnalysisResult, BusError> },
    RetryAnalysis,
}

pub struct SessionActor {
    pub bus: Bus,
    pub idem: RedisIdempotencyStore,
    pub conn: ConnectionManager,
    pub state: StoredSession,
    pub analysis: AnalysisRunner,
    pub tx: mpsc::Sender<Input>,
    pub clock: Arc<dyn Clock>,
}

pub struct Saved {
    pub session_id: i64,
    pub warnings: Vec<String>,
}

pub fn err<E: Into<ServiceError>>(e: E) -> ServiceError {
    e.into()
}

impl SessionActor {
    pub fn new(bus: Bus, tx: mpsc::Sender<Input>) -> Self {
        SessionActor {
            idem: RedisIdempotencyStore::new(&bus),
            conn: bus.connection(),
            clock: bus.clock(),
            state: StoredSession::idle(),
            analysis: AnalysisRunner::default(),
            bus,
            tx,
        }
    }

    pub fn now_ms(&self) -> i64 {
        unix_ms(self.clock.now())
    }

    pub async fn run(mut self, mut rx: mpsc::Receiver<Input>) {
        self.recover().await;
        while let Some(input) = rx.recv().await {
            match input {
                Input::Request { id, req, reply } => {
                    let _ = reply.send(self.on_request(id, req).await);
                }
                Input::ShotEvent { id, event, ack } => {
                    self.on_image_event(id, event).await;
                    let _ = ack.send(());
                }
                Input::DataChanged { event } => self.on_data_changed(event).await,
                Input::Analysis { shots_version, result } => self.on_analysis(shots_version, result).await,
                Input::RetryAnalysis => self.request_analysis(),
            }
        }
    }

    // doc/programs/session_manager.md#persistence-and-recovery
    async fn recover(&mut self) {
        let loaded = match persistence::load(&mut self.conn).await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(error = %e, "stored session could not be read");
                None
            }
        };
        let Some(mut s) = loaded.filter(|s| s.state() != SessionState::Idle) else { return };
        if s.state() != SessionState::Recovered {
            s.session.recovered_from = Some(s.state());
            if let Some(since) = s.running_since_ms.take() {
                s.session.duration_s += (s.last_change_ms - since).max(0) / 1000;
            }
            s.session.state = SessionState::Recovered;
        }
        if s.mode() == InputMode::Camera && s.live_active {
            if let Err(e) = self.bus.request(LiveStop {}).await {
                tracing::warn!(error = %e, "live_stop after restart failed");
            }
            s.live_active = false;
        }
        if let Err(e) = self.reread_records(&mut s).await {
            tracing::warn!(error = %e, "records could not be read again");
        }
        if !s.session.shots.is_empty() {
            s.shots_changed();
        }
        s.last_change_ms = self.now_ms();
        if let Err(e) = self.persist(s).await {
            tracing::error!(error = %e, "recovered session could not be stored");
        }
        self.publish_changed().await;
        self.request_analysis();
    }

    async fn on_request(&mut self, id: Uuid, req: R) -> Result<Value, ServiceError> {
        if changes_state(&req) {
            if let Some(stored) = self.idem.get(&id).await.map_err(err)? {
                return Ok(stored);
            }
        }
        if !is_allowed(self.state.state(), self.state.mode(), &req) {
            return Err(ServiceError::invalid_state(format!("not allowed in state {:?}", self.state.state())));
        }
        if !changes_state(&req) {
            return Ok(self.response(&self.state));
        }
        let mut next = self.state.clone();
        let before = next.shots_version;
        let saved = self.apply(&mut next, req).await?;
        next.last_change_ms = self.now_ms();
        let response = match &saved {
            Some(s) => serde_json::to_value(SessionSaveResponse {
                session: next.snapshot(next.last_change_ms),
                session_id: s.session_id,
                warnings: s.warnings.clone(),
            })
            .expect("serializable"),
            None => self.response(&next),
        };
        persistence::commit(&mut self.conn, &self.idem, &next, Some((id, &response))).await.map_err(err)?;
        self.state = next;
        if let Some(s) = &saved {
            self.publish(&SessionManagerEvent::SessionSaved(SessionSaved { session_id: s.session_id })).await;
        }
        self.publish_changed().await;
        if self.state.shots_version != before {
            self.request_analysis();
        }
        Ok(response)
    }

    fn response(&self, s: &StoredSession) -> Value {
        serde_json::to_value(SessionResponse { session: s.snapshot(self.now_ms()) }).expect("serializable")
    }

    // doc/programs/session_manager.md#saving
    pub async fn persist(&mut self, next: StoredSession) -> Result<(), BusError> {
        persistence::commit(&mut self.conn, &self.idem, &next, None).await?;
        self.state = next;
        Ok(())
    }

    async fn publish(&self, event: &SessionManagerEvent) {
        if let Err(e) = self.bus.publish(event).await {
            tracing::warn!(error = %e, "event not sent");
        }
    }

    // doc/interfaces.md#session_changed
    async fn publish_changed(&self) {
        let session = self.state.snapshot(self.now_ms());
        self.publish(&SessionManagerEvent::SessionChanged(SessionResponse { session })).await;
    }

    // doc/programs/session_manager.md#live-analysis
    fn request_analysis(&mut self) {
        if self.state.session.analysis_outdated && self.state.state() != SessionState::Idle {
            self.analysis.request(&self.state, &self.bus, &self.tx);
        }
    }

    async fn on_analysis(&mut self, version: u64, result: Result<AnalysisResult, BusError>) {
        let pending = self.analysis.finished();
        match result {
            Ok(r) if version == self.state.shots_version && self.state.state() != SessionState::Idle => {
                let mut next = self.state.clone();
                next.session.analysis = Some(r);
                next.session.analysis_outdated = false;
                match self.persist(next).await {
                    Ok(()) => self.publish_changed().await,
                    Err(e) => tracing::warn!(error = %e, "analysis result not stored"),
                }
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(error = %e, "analysis failed");
                if self.state.state() == SessionState::Recovered && !pending {
                    let (tx, clock) = (self.tx.clone(), self.clock.clone());
                    tokio::spawn(async move {
                        clock.sleep(ANALYSIS_RETRY).await;
                        let _ = tx.send(Input::RetryAnalysis).await;
                    });
                }
            }
        }
        if pending {
            self.request_analysis();
        }
    }

    // doc/programs/session_manager.md#transitions-and-actions
    async fn apply(&mut self, s: &mut StoredSession, req: R) -> Result<Option<Saved>, ServiceError> {
        let now_ms = self.now_ms();
        let camera = s.mode() == InputMode::Camera;
        match req {
            R::SessionSetup(r) => {
                *s = StoredSession::new(SessionState::Setup, r.input_mode);
            }
            R::SessionUpdateSetup(r) => {
                if r.distance_m.flatten().is_some_and(|d| d < 0) {
                    return Err(ServiceError::validation("distance_m >= 0"));
                }
                if let Some(Some(id)) = r.shooter_id {
                    match self.read_shooter(id).await? {
                        Some(sh) if !sh.retired => {}
                        Some(_) => return Err(ServiceError::validation(format!("shooter_id {id} is retired"))),
                        None => return Err(ServiceError::validation(format!("unknown shooter_id {id}"))),
                    }
                }
                let device = match r.device_id {
                    Some(Some(id)) => Some(Some(self.active_device(id).await?)),
                    Some(None) => Some(None),
                    None => None,
                };
                let target = match r.target_type_id {
                    Some(Some(id)) => Some(Some(self.active_target(id).await?)),
                    Some(None) => Some(None),
                    None => None,
                };
                let setup = &mut s.session.setup;
                if let Some(v) = r.shooter_id {
                    setup.shooter_id = v;
                }
                if let Some(v) = r.distance_m {
                    setup.distance_m = v;
                }
                if let Some(v) = r.single_handed {
                    setup.single_handed = v;
                }
                if let Some(v) = r.supported {
                    setup.supported = v;
                }
                let mut changed = false;
                if let Some(d) = device {
                    changed |= d.as_ref().map(|x| x.bullet_diameter_mm) != s.bullet_diameter_mm() || d.is_none() != s.device.is_none();
                    s.session.setup.device_id = d.as_ref().and_then(|x| x.id);
                    s.device = d;
                }
                if let Some(t) = target {
                    changed |= t != s.target_type;
                    s.session.setup.target_type_id = t.as_ref().and_then(|x| x.id);
                    s.target_type = t;
                }
                if changed {
                    if camera {
                        self.start_live(s).await?;
                    }
                    if !s.session.shots.is_empty() {
                        s.shots_changed();
                    }
                }
            }
            R::SessionCalibrate(r) => {
                if !s.live_active {
                    return Err(ServiceError::unavailable("live mode not started yet"));
                }
                s.session.calibration = Some(self.bus.request(LiveCalibrate { corners: r.corners }).await.map_err(err)?);
            }
            R::SessionStart(_) => {
                require_records(s)?;
                if camera {
                    require_calibrated(s)?;
                    self.bus.request(LiveSetCounting { counting: true }).await.map_err(err)?;
                }
                s.session.started_at.get_or_insert(now_ms / 1000);
                s.running_since_ms = Some(now_ms);
                s.session.state = SessionState::Running;
            }
            R::SessionUploadPhoto(r) => {
                let (target, bullet) = require_records(s)?;
                let resp = self
                    .bus
                    .request(DetectShotsInPhoto {
                        staged_file: r.staged_file,
                        target_type: target,
                        bullet_diameter_mm: bullet,
                        corners: r.corners,
                    })
                    .await
                    .map_err(err)?;
                let first = s.session.shots.is_empty() && s.session.paper_images.is_empty();
                let paper_no = if first { s.session.paper_no } else { s.session.paper_no + 1 };
                s.session.paper_no = paper_no;
                for d in resp.shots {
                    s.session.shots.push(CurrentShot {
                        angle_turns: d.angle_turns,
                        radius_mm: d.radius_mm,
                        origin: InputMode::Photo,
                        confidence: Some(d.confidence),
                        hole_id: None,
                        paper_no,
                    });
                }
                s.add_paper_image(paper_no, Some(resp.staged_file));
                s.save_warnings.extend(resp.warnings);
                s.shots_changed();
                s.session.state = SessionState::Review;
            }
            R::SessionPause(_) => {
                if camera {
                    self.bus.request(LiveSetCounting { counting: false }).await.map_err(err)?;
                }
                s.stop_clock(now_ms);
                s.session.state = SessionState::Paused;
            }
            R::SessionResume(_) => {
                if camera {
                    require_calibrated(s)?;
                    self.bus.request(LiveSetCounting { counting: true }).await.map_err(err)?;
                }
                s.running_since_ms = Some(now_ms);
                s.session.state = SessionState::Running;
            }
            R::SessionAddShot(r) => {
                let shot = valid_shot(r.angle_turns, r.radius_mm)?;
                let paper_no = r.paper_no.unwrap_or(s.session.paper_no);
                if !(1..=s.session.paper_no).contains(&paper_no) {
                    return Err(ServiceError::validation(format!("paper_no within 1..={}", s.session.paper_no)));
                }
                s.push_shot(CurrentShot {
                    angle_turns: shot.angle_turns,
                    radius_mm: shot.radius_mm,
                    origin: InputMode::Manual,
                    confidence: None,
                    hole_id: None,
                    paper_no,
                });
            }
            R::SessionUpdateShot(r) => {
                let shot = valid_shot(r.angle_turns, r.radius_mm)?;
                let i = valid_index(s, r.index)?;
                s.session.shots[i].angle_turns = shot.angle_turns;
                s.session.shots[i].radius_mm = shot.radius_mm;
                s.shots_changed();
            }
            R::SessionDeleteShot(r) => {
                let i = valid_index(s, r.index)?;
                let removed = s.session.shots.remove(i);
                s.shots_changed();
                let live = s.live_active && matches!(s.state(), SessionState::Running | SessionState::Paused);
                if let (Some(hole_id), true) = (removed.hole_id, live) {
                    if let Err(e) = self.bus.request(LiveUncountHole { hole_id }).await {
                        tracing::warn!(error = %e, hole_id, "live_uncount_hole failed");
                    }
                }
            }
            R::SessionNewPaper(_) => {
                if camera {
                    let from = s.last_live_paper.unwrap_or(1);
                    let resp = self.bus.request(LiveNewPaper { from_live_paper: from }).await.map_err(err)?;
                    if resp.changed {
                        let old = s.session.paper_no;
                        s.add_paper_image(old, resp.previous_paper_staged_file);
                        s.session.paper_no = old + 1;
                        s.last_live_paper = Some(resp.live_paper);
                    }
                } else {
                    s.session.paper_no += 1;
                }
            }
            R::SessionSetNotes(r) => {
                if r.notes.as_ref().is_some_and(|n| n.chars().count() > MAX_NOTES) {
                    return Err(ServiceError::validation("notes at most 10 000 characters"));
                }
                s.session.notes = r.notes;
            }
            R::SessionStop(_) => {
                s.stop_clock(now_ms);
                if camera && s.live_active {
                    match self.bus.request(CaptureTargetImage {}).await {
                        Ok(r) => s.add_paper_image(s.session.paper_no, Some(r.staged_file)),
                        Err(e) => s.save_warnings.push(format!("paper {} has no image: {e}", s.session.paper_no)),
                    }
                    if let Err(e) = self.bus.request(LiveStop {}).await {
                        tracing::warn!(error = %e, "live_stop failed");
                    }
                    s.live_active = false;
                }
                s.session.state = SessionState::Review;
            }
            R::SessionSave(_) => return saving::save(self, s).await.map(Some),
            R::SessionCancel(_) => {
                if camera && s.live_active {
                    if let Err(e) = self.bus.request(LiveStop {}).await {
                        tracing::warn!(error = %e, "live_stop failed");
                    }
                }
                *s = StoredSession::idle();
            }
            R::Ping(_) | R::SessionGet(_) => {}
        }
        Ok(None)
    }

    // doc/programs/session_manager.md#transitions-and-actions
    async fn start_live(&self, s: &mut StoredSession) -> Result<(), ServiceError> {
        let (Some(target_type), Some(bullet_diameter_mm)) = (s.target_type.clone(), s.bullet_diameter_mm()) else {
            return Ok(());
        };
        let status = self.bus.request(LiveStart { target_type, bullet_diameter_mm }).await.map_err(err)?;
        s.session.calibration = Some(status);
        s.live_active = true;
        s.live_paper_offset = s.session.paper_no - 1;
        s.last_live_paper = Some(1);
        for shot in s.session.shots.iter_mut().filter(|x| x.origin == InputMode::Camera) {
            shot.origin = InputMode::Manual;
            shot.hole_id = None;
        }
        Ok(())
    }

    pub async fn read_device(&self, id: i64) -> Result<Option<Device>, ServiceError> {
        Ok(self.bus.request(ReadDevices { filter: Filter::eq("id", id) }).await.map_err(err)?.devices.into_iter().next())
    }

    pub async fn read_target(&self, id: i64) -> Result<Option<TargetType>, ServiceError> {
        Ok(self
            .bus
            .request(ReadTargetTypes { filter: Filter::eq("id", id) })
            .await
            .map_err(err)?
            .target_types
            .into_iter()
            .next())
    }

    async fn read_shooter(&self, id: i64) -> Result<Option<targetscope_common::model::Shooter>, ServiceError> {
        Ok(self.bus.request(ReadShooters { filter: Filter::eq("id", id) }).await.map_err(err)?.shooters.into_iter().next())
    }

    async fn active_device(&self, id: i64) -> Result<Device, ServiceError> {
        match self.read_device(id).await? {
            Some(d) if !d.retired => Ok(d),
            Some(_) => Err(ServiceError::validation(format!("device_id {id} is retired"))),
            None => Err(ServiceError::validation(format!("unknown device_id {id}"))),
        }
    }

    async fn active_target(&self, id: i64) -> Result<TargetType, ServiceError> {
        match self.read_target(id).await? {
            Some(t) if !t.retired => Ok(t),
            Some(_) => Err(ServiceError::validation(format!("target_type_id {id} is retired"))),
            None => Err(ServiceError::validation(format!("unknown target_type_id {id}"))),
        }
    }

    // doc/programs/session_manager.md#data_changed-events
    pub async fn reread_records(&self, s: &mut StoredSession) -> Result<bool, ServiceError> {
        let mut changed = false;
        if let Some(id) = s.session.setup.device_id {
            let d = self.read_device(id).await?;
            changed |= d.as_ref().map(|x| x.bullet_diameter_mm) != s.bullet_diameter_mm();
            if d.is_none() {
                s.session.setup.device_id = None;
            }
            s.device = d;
        }
        if let Some(id) = s.session.setup.target_type_id {
            let t = self.read_target(id).await?;
            changed |= t != s.target_type;
            if t.is_none() {
                s.session.setup.target_type_id = None;
            }
            s.target_type = t;
        }
        Ok(changed)
    }

    // doc/programs/session_manager.md#data_changed-events
    async fn on_data_changed(&mut self, e: DataChanged) {
        let s = &self.state;
        if s.state() == SessionState::Idle {
            return;
        }
        let relevant = match e.table {
            Table::All => true,
            Table::Devices => s.session.setup.device_id.is_some_and(|id| e.key.as_i64() == Some(id)),
            Table::TargetTypes => s.session.setup.target_type_id.is_some_and(|id| e.key.as_i64() == Some(id)),
            _ => false,
        };
        if !relevant {
            return;
        }
        let mut next = self.state.clone();
        if e.action == ChangeAction::Deleted {
            match e.table {
                Table::Devices => {
                    next.session.setup.device_id = None;
                    next.device = None;
                }
                _ => {
                    next.session.setup.target_type_id = None;
                    next.target_type = None;
                }
            }
        } else {
            let before = (next.target_type.clone(), next.bullet_diameter_mm());
            if let Err(e) = self.reread_records(&mut next).await {
                return tracing::warn!(error = %e, "records could not be read again");
            }
            let geometry_changed = before != (next.target_type.clone(), next.bullet_diameter_mm());
            let live = next.live_active && matches!(next.state(), SessionState::Setup | SessionState::Running | SessionState::Paused);
            if live && geometry_changed {
                if let Err(e) = self.start_live(&mut next).await {
                    tracing::warn!(error = %e, "live_start failed");
                }
            }
            next.shots_changed();
        }
        next.last_change_ms = self.now_ms();
        match self.persist(next).await {
            Ok(()) => {
                self.publish_changed().await;
                self.request_analysis();
            }
            Err(e) => tracing::warn!(error = %e, "change not stored"),
        }
    }

    // doc/programs/session_manager.md#shot_detected-events
    async fn on_image_event(&mut self, event_id: Uuid, event: ImageProcessorEvent) {
        match self.idem.get(&event_id).await {
            Ok(Some(_)) => return,
            Ok(None) => {}
            Err(e) => tracing::warn!(error = %e, "event record could not be read"),
        }
        let mut next = self.state.clone();
        let changed = match event {
            ImageProcessorEvent::ShotDetected(e) => self.on_shot(&mut next, e).await,
            ImageProcessorEvent::PaperReplaced(e) => on_paper_replaced(&mut next, e),
            ImageProcessorEvent::CalibrationChanged(c) => {
                let applies = next.mode() == InputMode::Camera && next.state() != SessionState::Idle;
                if applies {
                    next.session.calibration = Some(c);
                }
                applies
            }
        };
        if !changed {
            return;
        }
        let before = self.state.shots_version;
        next.last_change_ms = self.now_ms();
        let record = json!({});
        if let Err(e) = persistence::commit(&mut self.conn, &self.idem, &next, Some((event_id, &record))).await {
            return tracing::warn!(error = %e, "event change not stored");
        }
        self.state = next;
        self.publish_changed().await;
        if self.state.shots_version != before {
            self.request_analysis();
        }
    }

    async fn on_shot(&self, s: &mut StoredSession, e: ShotDetected) -> bool {
        if s.state() == SessionState::Running && s.mode() == InputMode::Camera {
            s.push_shot(CurrentShot {
                angle_turns: e.angle_turns,
                radius_mm: e.radius_mm,
                origin: InputMode::Camera,
                confidence: Some(e.confidence),
                hole_id: Some(e.hole_id),
                paper_no: e.live_paper + s.live_paper_offset,
            });
            return true;
        }
        if s.live_active && matches!(s.state(), SessionState::Setup | SessionState::Running | SessionState::Paused) {
            if let Err(err) = self.bus.request(LiveUncountHole { hole_id: e.hole_id }).await {
                tracing::warn!(error = %err, "live_uncount_hole failed");
            }
        }
        false
    }

    // doc/programs/session_manager.md#saving
    pub async fn analyze_now(&self, s: &StoredSession) -> Result<AnalysisResult, ServiceError> {
        let (Some(target_type), Some(bullet_diameter_mm)) = (s.target_type.clone(), s.bullet_diameter_mm()) else {
            return Err(ServiceError::validation("device and target type are required"));
        };
        let shots = s.session.shots.iter().map(|x| x.shot()).collect();
        self.bus
            .request(AnalyzeShots { shots, target_type, bullet_diameter_mm })
            .await
            .map_err(|e| ServiceError::unavailable(format!("analyzer: {e}")))
    }

    pub fn now_s(&self) -> i64 {
        unix_s(self.clock.now())
    }
}

// doc/programs/session_manager.md#shot_detected-events
fn on_paper_replaced(s: &mut StoredSession, e: PaperReplaced) -> bool {
    let newer = s.last_live_paper.is_none_or(|last| e.live_paper > last);
    if !matches!(s.state(), SessionState::Running | SessionState::Paused) || !newer {
        return false;
    }
    let old = s.session.paper_no;
    s.add_paper_image(old, e.previous_paper_staged_file);
    s.session.paper_no = e.live_paper + s.live_paper_offset;
    s.last_live_paper = Some(e.live_paper);
    true
}

// doc/programs/session_manager.md#transitions-and-actions
pub fn require_records(s: &StoredSession) -> Result<(TargetType, f64), ServiceError> {
    let bullet = s.bullet_diameter_mm().filter(|_| s.session.setup.device_id.is_some());
    let Some(bullet) = bullet else {
        return Err(ServiceError::validation("device_id is required"));
    };
    match (&s.target_type, s.session.setup.target_type_id) {
        (Some(t), Some(_)) => Ok((t.clone(), bullet)),
        _ => Err(ServiceError::validation("target_type_id is required")),
    }
}

fn require_calibrated(s: &StoredSession) -> Result<(), ServiceError> {
    match &s.session.calibration {
        Some(c) if c.state == CalibrationState::Calibrated => Ok(()),
        _ => Err(ServiceError::unavailable("not calibrated")),
    }
}

fn valid_shot(angle_turns: f64, radius_mm: f64) -> Result<Shot, ServiceError> {
    let shot = Shot { angle_turns, radius_mm };
    if shot.is_valid() {
        Ok(shot)
    } else {
        Err(ServiceError::validation("0 <= angle_turns < 1 and radius_mm >= 0"))
    }
}

fn valid_index(s: &StoredSession, index: i64) -> Result<usize, ServiceError> {
    if index < 0 || index as usize >= s.session.shots.len() {
        return Err(ServiceError::validation(format!("index within 0..{}", s.session.shots.len())));
    }
    Ok(index as usize)
}
