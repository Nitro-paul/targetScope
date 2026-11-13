// doc/programs/launcher.md#startup-sequence
// doc/programs/launcher.md#supervision
// doc/programs/launcher.md#shutdown

use std::collections::HashMap;
use std::process::Stdio;
use std::time::{Duration, Instant, SystemTime};

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use serde_json::Value;
use targetscope_common::bus::Bus;
use targetscope_common::idempotency::RedisIdempotencyStore;
use targetscope_common::messages::launcher::{LauncherEvent, LauncherRequest, LauncherStatusResponse, ProgramState, RestartProgram};
use targetscope_common::messages::ServiceError;
use tokio::process::Command;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::config::LauncherConfig;
use crate::program::ProgramHandle;
use crate::supervisor::{initial_backoff, next_backoff, too_many_restarts};
use crate::{env, logs, order, redis_server};

// doc/programs/launcher.md#startup-sequence
pub const READY_CHECK: Duration = Duration::from_millis(250);
pub const READY_TIMEOUT: Duration = Duration::from_secs(30);
// doc/programs/launcher.md#shutdown
pub const STOP_TIMEOUT: Duration = Duration::from_secs(10);

pub enum Event {
    Exited { name: String, generation: u64 },
    Ready { name: String, generation: u64 },
    NotReady { name: String, generation: u64 },
    RestartDue { name: String, generation: u64 },
    RedisExited,
    RedisRestartDue,
    Request { id: Uuid, req: LauncherRequest, reply: oneshot::Sender<Result<Value, ServiceError>> },
    Shutdown,
}

pub struct Launcher {
    pub config: LauncherConfig,
    pub programs: Vec<ProgramHandle>,
    index: HashMap<String, usize>,
    env: Vec<(String, String)>,
    bus: Bus,
    idem: RedisIdempotencyStore,
    tx: mpsc::Sender<Event>,
    stopping: bool,
    redis_pid: Option<u32>,
    redis_started: Instant,
    redis_backoff: Duration,
}

fn signal(pid: u32, sig: Signal) {
    let _ = kill(Pid::from_raw(pid as i32), sig);
}

// doc/programs/launcher.md#redis
pub fn watch_redis(mut child: tokio::process::Child, tx: mpsc::Sender<Event>) -> Option<u32> {
    let pid = child.id();
    tokio::spawn(async move {
        let _ = child.wait().await;
        let _ = tx.send(Event::RedisExited).await;
    });
    pid
}

impl Launcher {
    pub fn new(config: LauncherConfig, bus: Bus, tx: mpsc::Sender<Event>, redis_pid: Option<u32>) -> Result<Self, String> {
        let order = order::start_order(&config.programs)?;
        let backoff = initial_backoff(&config.restart);
        let programs: Vec<ProgramHandle> = order
            .iter()
            .map(|n| ProgramHandle::new(config.programs.iter().find(|p| &p.name == n).expect("ordered").clone(), backoff))
            .collect();
        let index = programs.iter().enumerate().map(|(i, p)| (p.spec.name.clone(), i)).collect();
        Ok(Launcher {
            env: env::program_env(&config),
            idem: RedisIdempotencyStore::new(&bus),
            config,
            programs,
            index,
            bus,
            tx,
            stopping: false,
            redis_pid,
            redis_started: Instant::now(),
            redis_backoff: backoff,
        })
    }

    pub async fn run(mut self, mut rx: mpsc::Receiver<Event>) {
        self.start_ready_programs().await;
        while let Some(event) = rx.recv().await {
            if let Event::Shutdown = event {
                self.shutdown(&mut rx).await;
                return;
            }
            self.handle(event).await;
        }
    }

    async fn handle(&mut self, event: Event) {
        match event {
            Event::Ready { name, generation } => {
                if let Some(i) = self.current(&name, generation) {
                    if self.programs[i].state == ProgramState::Starting {
                        self.set_state(i, ProgramState::Running).await;
                        self.start_ready_programs().await;
                    }
                }
            }
            Event::NotReady { name, generation } => {
                if let Some(i) = self.current(&name, generation) {
                    tracing::warn!(program = %name, "no ping answer within 30 s; stopping it");
                    self.terminate(i);
                }
            }
            Event::Exited { name, generation } => {
                if let Some(i) = self.current(&name, generation) {
                    self.on_exit(i).await;
                }
            }
            Event::RestartDue { name, generation } => {
                if let Some(i) = self.current(&name, generation) {
                    if self.programs[i].state == ProgramState::Backoff && !self.stopping {
                        self.programs[i].restart_count += 1;
                        self.start(i).await;
                    }
                }
            }
            Event::RedisExited => {
                self.redis_pid = None;
                if self.redis_started.elapsed() > Duration::from_secs(self.config.restart.stable_after_s) {
                    self.redis_backoff = initial_backoff(&self.config.restart);
                }
                tracing::warn!(backoff_s = self.redis_backoff.as_secs(), "Redis exited; restarting");
                let (tx, delay) = (self.tx.clone(), self.redis_backoff);
                self.redis_backoff = next_backoff(self.redis_backoff, &self.config.restart);
                tokio::spawn(async move {
                    tokio::time::sleep(delay).await;
                    let _ = tx.send(Event::RedisRestartDue).await;
                });
            }
            Event::RedisRestartDue => match redis_server::spawn(&self.config) {
                Ok(child) => {
                    self.redis_started = Instant::now();
                    self.redis_pid = watch_redis(child, self.tx.clone());
                }
                Err(e) => {
                    tracing::error!(error = %e, "Redis could not be restarted");
                    let _ = self.tx.send(Event::RedisExited).await;
                }
            },
            Event::Request { id, req, reply } => {
                let _ = reply.send(self.request(id, req).await);
            }
            Event::Shutdown => {}
        }
    }

    fn current(&self, name: &str, generation: u64) -> Option<usize> {
        self.index.get(name).copied().filter(|i| self.programs[*i].generation == generation)
    }

    // doc/programs/launcher.md#startup-sequence
    async fn start_ready_programs(&mut self) {
        if self.stopping {
            return;
        }
        for i in 0..self.programs.len() {
            let p = &self.programs[i];
            if !p.spec.enabled || p.ever_started {
                continue;
            }
            let deps = order::enabled_dependencies(&p.spec, &self.config.programs);
            let ready = deps.iter().all(|d| self.index.get(*d).is_some_and(|j| self.programs[*j].state == ProgramState::Running));
            if ready {
                self.start(i).await;
            }
        }
    }

    async fn start(&mut self, i: usize) {
        let binary = self.config.binary(&self.programs[i].spec);
        let p = &mut self.programs[i];
        p.generation += 1;
        p.ever_started = true;
        p.started = Some(Instant::now());
        let (name, generation) = (p.spec.name.clone(), p.generation);
        let spawned = Command::new(&binary)
            .envs(self.env.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(false)
            .spawn();
        match spawned {
            Ok(mut child) => {
                logs::forward_output(&name, &mut child);
                self.programs[i].pid = child.id();
                let tx = self.tx.clone();
                let n = name.clone();
                tokio::spawn(async move {
                    let _ = child.wait().await;
                    let _ = tx.send(Event::Exited { name: n, generation }).await;
                });
                let (bus, tx, n) = (self.bus.clone(), self.tx.clone(), name.clone());
                tokio::spawn(async move {
                    let deadline = Instant::now() + READY_TIMEOUT;
                    loop {
                        if bus.ping(&n, READY_CHECK).await.is_ok() {
                            let _ = tx.send(Event::Ready { name: n, generation }).await;
                            return;
                        }
                        if Instant::now() > deadline {
                            let _ = tx.send(Event::NotReady { name: n, generation }).await;
                            return;
                        }
                        tokio::time::sleep(READY_CHECK).await;
                    }
                });
                self.set_state(i, ProgramState::Starting).await;
            }
            Err(e) => {
                tracing::error!(program = %name, binary = %binary.display(), error = %e, "program could not be started");
                self.set_state(i, ProgramState::Starting).await;
                let _ = self.tx.send(Event::Exited { name, generation }).await;
            }
        }
    }

    // doc/programs/launcher.md#supervision
    async fn on_exit(&mut self, i: usize) {
        let policy = self.config.restart.clone();
        let p = &mut self.programs[i];
        p.pid = None;
        if self.stopping {
            return self.set_state(i, ProgramState::Stopped).await;
        }
        if std::mem::take(&mut p.restart_requested) {
            p.backoff = initial_backoff(&policy);
            p.restarts.clear();
            return self.start(i).await;
        }
        if p.started.is_some_and(|s| s.elapsed() > Duration::from_secs(policy.stable_after_s)) {
            p.backoff = initial_backoff(&policy);
        }
        tracing::warn!(program = %p.spec.name, "program exited");
        if too_many_restarts(&mut p.restarts, Instant::now(), &policy) {
            tracing::error!(program = %p.spec.name, "too many restarts; giving up");
            return self.set_state(i, ProgramState::Failed).await;
        }
        let (delay, name, generation) = (p.backoff, p.spec.name.clone(), p.generation);
        p.backoff = next_backoff(p.backoff, &policy);
        let tx = self.tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let _ = tx.send(Event::RestartDue { name, generation }).await;
        });
        self.set_state(i, ProgramState::Backoff).await;
    }

    fn terminate(&self, i: usize) {
        let Some(pid) = self.programs[i].pid else { return };
        signal(pid, Signal::SIGTERM);
        tokio::spawn(async move {
            tokio::time::sleep(STOP_TIMEOUT).await;
            if kill(Pid::from_raw(pid as i32), None).is_ok() {
                signal(pid, Signal::SIGKILL);
            }
        });
    }

    async fn set_state(&mut self, i: usize, state: ProgramState) {
        let p = &mut self.programs[i];
        p.state = state;
        p.since = SystemTime::now();
        let status = p.status();
        tracing::info!(program = %status.name, state = ?status.state, pid = ?status.pid, "program state changed");
        if let Err(e) = self.bus.publish(&LauncherEvent::ProgramStateChanged(status)).await {
            tracing::warn!(error = %e, "program_state_changed not sent");
        }
    }

    // doc/interfaces.md#launcher
    async fn request(&mut self, id: Uuid, req: LauncherRequest) -> Result<Value, ServiceError> {
        match req {
            LauncherRequest::LauncherStatus(_) | LauncherRequest::Ping(_) => {
                let programs = self.programs.iter().map(|p| p.status()).collect();
                Ok(serde_json::to_value(LauncherStatusResponse { programs }).expect("serializable"))
            }
            LauncherRequest::RestartProgram(r) => {
                if let Some(stored) = self.idem.get(&id).await.map_err(ServiceError::from)? {
                    return Ok(stored);
                }
                let i = *self.index.get(&r.name).ok_or_else(|| ServiceError::not_found(format!("program {}", r.name)))?;
                if self.stopping {
                    return Err(ServiceError::invalid_state("stopping"));
                }
                let response = serde_json::to_value(RestartProgram { name: r.name.clone() }).expect("serializable");
                let mut pipe = redis::pipe();
                self.idem.record(&mut pipe, &id, &response);
                let () = pipe
                    .query_async(&mut self.bus.connection())
                    .await
                    .map_err(|e| ServiceError::from(targetscope_common::bus::BusError::from(e)))?;
                let p = &mut self.programs[i];
                p.restarts.clear();
                p.restart_count = 0;
                p.backoff = initial_backoff(&self.config.restart);
                match (p.state, p.pid) {
                    (ProgramState::Running | ProgramState::Starting, Some(_)) => {
                        p.restart_requested = true;
                        self.terminate(i);
                    }
                    _ => self.start(i).await,
                }
                Ok(response)
            }
        }
    }

    // doc/programs/launcher.md#shutdown
    async fn shutdown(&mut self, rx: &mut mpsc::Receiver<Event>) {
        self.stopping = true;
        self.bus.stop();
        for i in (0..self.programs.len()).rev() {
            let Some(pid) = self.programs[i].pid else { continue };
            let generation = self.programs[i].generation;
            signal(pid, Signal::SIGTERM);
            let deadline = tokio::time::Instant::now() + STOP_TIMEOUT;
            let mut killed = false;
            loop {
                tokio::select! {
                    event = rx.recv() => match event {
                        Some(Event::Exited { name, generation: g }) => {
                            if let Some(j) = self.current(&name, g) {
                                self.programs[j].pid = None;
                                self.set_state(j, ProgramState::Stopped).await;
                                if j == i && g == generation {
                                    break;
                                }
                            }
                        }
                        Some(Event::Request { reply, .. }) => {
                            let _ = reply.send(Err(ServiceError::invalid_state("stopping")));
                        }
                        Some(_) => {}
                        None => break,
                    },
                    _ = tokio::time::sleep_until(deadline), if !killed => {
                        tracing::warn!(program = %self.programs[i].spec.name, "no exit after SIGTERM; killing");
                        signal(pid, Signal::SIGKILL);
                        killed = true;
                    }
                }
            }
        }
        redis_server::shutdown(&self.config.redis_url()).await;
        if let Some(pid) = self.redis_pid {
            for _ in 0..100 {
                if kill(Pid::from_raw(pid as i32), None).is_err() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        tracing::info!("stopped");
    }
}
