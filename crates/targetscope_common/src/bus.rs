// doc/shared_libraries.md#redis-bus
// doc/interfaces.md#concept

use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use async_trait::async_trait;
use redis::aio::{ConnectionManager, ConnectionManagerConfig};
use redis::streams::{StreamId, StreamReadReply};
use redis::{FromRedisValue, Value as RedisValue};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::oneshot;
use tracing::Instrument;
use uuid::Uuid;

use crate::clock::{unix_ms, unix_s, Clock, SystemClock};
use crate::failpoints;
use crate::messages::{
    event_stream, request_stream, response_stream, ErrorBody, ErrorCode, EventEnvelope, PingResponse, Request,
    RequestEnvelope, ResponseEnvelope, ServiceError, Status,
};

// doc/interfaces.md#streams-and-keys
pub const MAXLEN_REQUESTS: usize = 10_000;
pub const MAXLEN_EVENTS: usize = 10_000;
pub const MAXLEN_RESPONSES: usize = 1_000;
// doc/interfaces.md#timeouts
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

const BLOCK_MS: usize = 1000;
const BATCH: usize = 100;
const MIN_BACKOFF: Duration = Duration::from_millis(100);
const MAX_BACKOFF: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub enum BusError {
    #[error("redis: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("timeout")]
    Timeout,
    #[error("{}: {}", .0.code.as_str(), .0.message)]
    Remote(ErrorBody),
    #[error("decode: {0}")]
    Decode(String),
}

impl BusError {
    // doc/interfaces.md#error-codes
    pub fn to_service_error(&self) -> ServiceError {
        match self {
            BusError::Remote(body) => body.clone().into(),
            BusError::Timeout => ServiceError::new(ErrorCode::Timeout, "no response in time"),
            BusError::Redis(e) => ServiceError::unavailable(format!("redis: {e}")),
            BusError::Decode(e) => ServiceError::new(ErrorCode::InvalidMessage, e.clone()),
        }
    }
}

impl From<BusError> for ServiceError {
    fn from(e: BusError) -> Self {
        e.to_service_error()
    }
}

#[async_trait]
pub trait RequestHandler<Req>: Send + Sync {
    async fn handle(&self, request_id: Uuid, req: Req) -> Result<Value, ServiceError>;
}

// doc/interfaces.md#who-listens-to-which-events
#[derive(Debug, Clone)]
pub enum ReadMode {
    ConsumerGroup,
    After(String),
}

#[derive(Clone)]
pub struct Bus {
    inner: Arc<Inner>,
}

struct Inner {
    client: redis::Client,
    conn: ConnectionManager,
    program: String,
    consumer: String,
    started_at: i64,
    clock: Arc<dyn Clock>,
    pending: Mutex<HashMap<Uuid, oneshot::Sender<ResponseEnvelope>>>,
}

impl Bus {
    pub async fn connect(url: &str, program: &str) -> Result<Self, BusError> {
        Self::connect_with_clock(url, program, Arc::new(SystemClock)).await
    }

    pub async fn connect_with_clock(url: &str, program: &str, clock: Arc<dyn Clock>) -> Result<Self, BusError> {
        let client = redis::Client::open(url)?;
        let conn = command_connection(&client).await?;
        let mut c = conn.clone();
        let start_id = newest_id(&mut c, &response_stream(program)).await?;
        let inner = Arc::new(Inner {
            client,
            conn,
            program: program.to_string(),
            consumer: consumer_name(),
            started_at: unix_s(clock.now()),
            clock,
            pending: Mutex::new(HashMap::new()),
        });
        let reader_conn = blocking_connection(&inner.client).await?;
        tokio::spawn(read_responses(Arc::downgrade(&inner), reader_conn, start_id));
        Ok(Bus { inner })
    }

    pub fn program(&self) -> &str {
        &self.inner.program
    }

    pub fn clock(&self) -> Arc<dyn Clock> {
        self.inner.clock.clone()
    }

    pub fn connection(&self) -> ConnectionManager {
        self.inner.conn.clone()
    }

    pub async fn request<R: Request>(&self, req: R) -> Result<R::Response, BusError> {
        self.request_with_id(Uuid::new_v4(), req).await
    }

    pub async fn request_with_id<R: Request>(&self, id: Uuid, req: R) -> Result<R::Response, BusError> {
        let env = RequestEnvelope::new(id, Some(response_stream(&self.inner.program)), &req);
        let resp = self.send_envelope(R::STREAM, env, R::TIMEOUT).await?;
        decode_response(resp)
    }

    // doc/interfaces.md#ping
    pub async fn ping(&self, program: &str, timeout: Duration) -> Result<PingResponse, BusError> {
        let env = RequestEnvelope {
            r#type: "ping".into(),
            version: crate::messages::VERSION,
            request_id: Uuid::new_v4(),
            reply_to: Some(response_stream(&self.inner.program)),
            payload: serde_json::json!({}),
        };
        decode_response(self.send_envelope(&request_stream(program), env, timeout).await?)
    }

    pub async fn send_envelope(
        &self,
        stream: &str,
        mut env: RequestEnvelope,
        timeout: Duration,
    ) -> Result<ResponseEnvelope, BusError> {
        let id = env.request_id;
        if env.reply_to.is_none() {
            env.reply_to = Some(response_stream(&self.inner.program));
        }
        let (tx, rx) = oneshot::channel();
        self.inner.pending.lock().unwrap().insert(id, tx);
        let data = serde_json::to_string(&env).map_err(|e| BusError::Decode(e.to_string()))?;
        if let Err(e) = xadd(&mut self.inner.conn.clone(), stream, MAXLEN_REQUESTS, &data).await {
            self.inner.pending.lock().unwrap().remove(&id);
            return Err(e);
        }
        tokio::select! {
            resp = rx => resp.map_err(|_| BusError::Timeout),
            _ = self.inner.clock.sleep(timeout) => {
                self.inner.pending.lock().unwrap().remove(&id);
                Err(BusError::Timeout)
            }
        }
    }

    // doc/interfaces.md#event-envelope
    pub async fn publish<E: Serialize + Sync>(&self, event: &E) -> Result<EventEnvelope, BusError> {
        let env = EventEnvelope::new(&self.inner.program, unix_ms(self.inner.clock.now()), event)
            .map_err(|e| BusError::Decode(e.to_string()))?;
        let data = serde_json::to_string(&env).map_err(|e| BusError::Decode(e.to_string()))?;
        xadd(&mut self.inner.conn.clone(), &event_stream(&self.inner.program), MAXLEN_EVENTS, &data).await?;
        Ok(env)
    }

    // doc/interfaces.md#who-listens-to-which-events
    pub async fn newest_event_id(&self, source: &str) -> Result<String, BusError> {
        newest_id(&mut self.inner.conn.clone(), &event_stream(source)).await
    }

    pub fn events<E: DeserializeOwned>(&self, source: &str, mode: ReadMode) -> EventReader<E> {
        EventReader {
            client: self.inner.client.clone(),
            conn: None,
            stream: event_stream(source),
            group: self.inner.program.clone(),
            consumer: self.inner.consumer.clone(),
            last_id: match &mode {
                ReadMode::After(id) => id.clone(),
                ReadMode::ConsumerGroup => "0".into(),
            },
            mode,
            claimed: false,
            buffer: VecDeque::new(),
            clock: self.inner.clock.clone(),
            _marker: PhantomData,
        }
    }

    // doc/shared_libraries.md#redis-bus
    pub async fn serve<Req, H>(&self, handler: H) -> Result<(), BusError>
    where
        Req: DeserializeOwned + Send,
        H: RequestHandler<Req>,
    {
        let stream = request_stream(&self.inner.program);
        let group = self.inner.program.clone();
        let mut backoff = MIN_BACKOFF;
        let mut conn: Option<ConnectionManager> = None;
        let mut pending_phase = true;
        loop {
            let c = match conn.as_mut() {
                Some(c) => c,
                None => match self.prepare_group(&stream, &group).await {
                    Ok(c) => {
                        pending_phase = true;
                        conn.insert(c)
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "redis not reachable");
                        self.inner.clock.sleep(backoff).await;
                        backoff = (backoff * 2).min(MAX_BACKOFF);
                        continue;
                    }
                },
            };
            let start = if pending_phase { "0" } else { ">" };
            match xreadgroup(c, &stream, &group, &self.inner.consumer, start, !pending_phase).await {
                Ok(entries) => {
                    backoff = MIN_BACKOFF;
                    if pending_phase && entries.is_empty() {
                        pending_phase = false;
                    }
                    for entry in entries {
                        self.handle_entry(c, &stream, &group, entry, &handler).await;
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "reading requests failed");
                    conn = None;
                    self.inner.clock.sleep(backoff).await;
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                }
            }
        }
    }

    async fn prepare_group(&self, stream: &str, group: &str) -> Result<ConnectionManager, BusError> {
        let mut c = blocking_connection(&self.inner.client).await?;
        create_group(&mut c, stream, group).await?;
        claim_all(&mut c, stream, group, &self.inner.consumer).await?;
        Ok(c)
    }

    async fn handle_entry<Req, H>(
        &self,
        conn: &mut ConnectionManager,
        stream: &str,
        group: &str,
        entry: StreamId,
        handler: &H,
    ) where
        Req: DeserializeOwned + Send,
        H: RequestHandler<Req>,
    {
        let response = match entry_data(&entry).and_then(|d| serde_json::from_str::<Value>(&d).ok()) {
            None => {
                tracing::error!(entry = %entry.id, "unparseable request entry");
                None
            }
            Some(raw) => self.process(raw, handler).await,
        };
        if failpoints::hit("bus.before_response") {
            return;
        }
        if let Some((reply_to, resp)) = response {
            match serde_json::to_string(&resp) {
                Ok(data) => {
                    if let Err(e) = xadd(conn, &reply_to, MAXLEN_RESPONSES, &data).await {
                        tracing::error!(error = %e, request_id = %resp.request_id, "sending response failed");
                        return;
                    }
                }
                Err(e) => tracing::error!(error = %e, "encoding response failed"),
            }
        }
        if failpoints::hit("bus.before_ack") {
            return;
        }
        let ack: Result<i64, _> = redis::cmd("XACK").arg(stream).arg(group).arg(&entry.id).query_async(conn).await;
        if let Err(e) = ack {
            tracing::error!(error = %e, entry = %entry.id, "acknowledging request failed");
        }
    }

    async fn process<Req, H>(&self, raw: Value, handler: &H) -> Option<(String, ResponseEnvelope)>
    where
        Req: DeserializeOwned + Send,
        H: RequestHandler<Req>,
    {
        let env: RequestEnvelope = match serde_json::from_value(raw.clone()) {
            Ok(env) => env,
            Err(e) => {
                tracing::error!(error = %e, "invalid request envelope");
                let id = raw.get("request_id").and_then(Value::as_str).and_then(|s| Uuid::parse_str(s).ok())?;
                let reply_to = raw.get("reply_to").and_then(Value::as_str)?.to_string();
                let r#type = raw.get("type").and_then(Value::as_str).unwrap_or("unknown").to_string();
                let error = ServiceError::new(ErrorCode::InvalidMessage, e.to_string());
                return Some((
                    reply_to,
                    ResponseEnvelope {
                        r#type: format!("{type}_response"),
                        version: crate::messages::VERSION,
                        request_id: id,
                        status: Status::Error,
                        error: Some(error.body()),
                        payload: None,
                    },
                ));
            }
        };
        let span = tracing::info_span!("request", request_id = %env.request_id, r#type = %env.r#type);
        let result = async {
            if env.r#type == "ping" && env.version == crate::messages::VERSION {
                return Ok(serde_json::to_value(self.ping_response()).unwrap_or(Value::Null));
            }
            let req: Req = env.decode()?;
            handler.handle(env.request_id, req).await
        }
        .instrument(span.clone())
        .await;
        let resp = match &result {
            Ok(payload) => env.ok_response(payload.clone()),
            Err(e) => {
                span.in_scope(|| tracing::info!(code = e.code.as_str(), message = %e.message, "request failed"));
                env.error_response(e)
            }
        };
        env.reply_to.clone().map(|r| (r, resp))
    }

    fn ping_response(&self) -> PingResponse {
        PingResponse {
            program: self.inner.program.clone(),
            program_version: env!("CARGO_PKG_VERSION").to_string(),
            language: "rust".into(),
            started_at: self.inner.started_at,
        }
    }
}

fn decode_response<T: DeserializeOwned>(resp: ResponseEnvelope) -> Result<T, BusError> {
    match resp.status {
        Status::Ok => serde_json::from_value(resp.payload.unwrap_or(Value::Null))
            .map_err(|e| BusError::Decode(format!("{}: {e}", resp.r#type))),
        Status::Error => Err(BusError::Remote(resp.error.unwrap_or(ErrorBody {
            code: ErrorCode::InvalidMessage,
            message: "error response without error".into(),
            details: None,
        }))),
    }
}

async fn read_responses(inner: Weak<Inner>, mut conn: ConnectionManager, mut last_id: String) {
    let stream = match inner.upgrade() {
        Some(i) => response_stream(&i.program),
        None => return,
    };
    let mut backoff = MIN_BACKOFF;
    loop {
        let result = xread(&mut conn, &stream, &last_id).await;
        let Some(inner) = inner.upgrade() else { return };
        match result {
            Ok(entries) => {
                backoff = MIN_BACKOFF;
                for entry in entries {
                    last_id = entry.id.clone();
                    let Some(resp) = entry_data(&entry).and_then(|d| serde_json::from_str::<ResponseEnvelope>(&d).ok())
                    else {
                        tracing::error!(entry = %entry.id, "unparseable response entry");
                        continue;
                    };
                    let waiting = inner.pending.lock().unwrap().remove(&resp.request_id);
                    match waiting {
                        Some(tx) => {
                            let _ = tx.send(resp);
                        }
                        None => tracing::warn!(request_id = %resp.request_id, "late response dropped"),
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "reading responses failed");
                let clock = inner.clock.clone();
                drop(inner);
                clock.sleep(backoff).await;
                backoff = (backoff * 2).min(MAX_BACKOFF);
            }
        }
    }
}

pub struct Event<E> {
    pub entry_id: String,
    pub envelope: EventEnvelope,
    pub event: E,
}

pub struct EventReader<E> {
    client: redis::Client,
    conn: Option<ConnectionManager>,
    stream: String,
    group: String,
    consumer: String,
    mode: ReadMode,
    last_id: String,
    claimed: bool,
    buffer: VecDeque<StreamId>,
    clock: Arc<dyn Clock>,
    _marker: PhantomData<E>,
}

impl<E: DeserializeOwned> EventReader<E> {
    pub async fn next(&mut self) -> Result<Event<E>, BusError> {
        loop {
            while let Some(entry) = self.buffer.pop_front() {
                if let ReadMode::After(_) = self.mode {
                    self.last_id = entry.id.clone();
                }
                let decoded = entry_data(&entry)
                    .and_then(|d| serde_json::from_str::<EventEnvelope>(&d).ok())
                    .and_then(|env| env.decode::<E>().ok().map(|ev| (env, ev)));
                match decoded {
                    Some((envelope, event)) => return Ok(Event { entry_id: entry.id, envelope, event }),
                    None => {
                        tracing::debug!(entry = %entry.id, stream = %self.stream, "event skipped");
                        if let ReadMode::ConsumerGroup = self.mode {
                            self.ack(&entry.id).await?;
                        }
                    }
                }
            }
            self.fill().await?;
        }
    }

    // doc/interfaces.md#who-listens-to-which-events
    pub async fn ack(&mut self, entry_id: &str) -> Result<(), BusError> {
        let (stream, group) = (self.stream.clone(), self.group.clone());
        let conn = self.connection().await?;
        let _: i64 = redis::cmd("XACK").arg(stream).arg(group).arg(entry_id).query_async(conn).await?;
        Ok(())
    }

    async fn connection(&mut self) -> Result<&mut ConnectionManager, BusError> {
        if self.conn.is_none() {
            self.conn = Some(blocking_connection(&self.client).await?);
        }
        Ok(self.conn.as_mut().unwrap())
    }

    async fn fill(&mut self) -> Result<(), BusError> {
        let mut backoff = MIN_BACKOFF;
        loop {
            match self.try_fill().await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    tracing::warn!(error = %e, stream = %self.stream, "reading events failed");
                    self.conn = None;
                    self.claimed = false;
                    self.clock.sleep(backoff).await;
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                }
            }
        }
    }

    async fn try_fill(&mut self) -> Result<(), BusError> {
        let (stream, group, consumer) = (self.stream.clone(), self.group.clone(), self.consumer.clone());
        match self.mode.clone() {
            ReadMode::After(_) => {
                let last = self.last_id.clone();
                let conn = self.connection().await?;
                let entries = xread(conn, &stream, &last).await?;
                self.buffer.extend(entries);
            }
            ReadMode::ConsumerGroup => {
                if !self.claimed {
                    let conn = self.connection().await?;
                    create_group(conn, &stream, &group).await?;
                    claim_all(conn, &stream, &group, &consumer).await?;
                    self.claimed = true;
                    self.last_id = "0".into();
                }
                let pending = self.last_id == "0";
                let conn = self.connection().await?;
                let entries = xreadgroup(conn, &stream, &group, &consumer, if pending { "0" } else { ">" }, !pending).await?;
                if pending && entries.is_empty() {
                    self.last_id = ">".into();
                }
                self.buffer.extend(entries);
            }
        }
        Ok(())
    }
}

fn consumer_name() -> String {
    let host = hostname::get().map(|h| h.to_string_lossy().into_owned()).unwrap_or_else(|_| "localhost".into());
    format!("{host}-{}", std::process::id())
}

async fn command_connection(client: &redis::Client) -> Result<ConnectionManager, BusError> {
    let config = ConnectionManagerConfig::new()
        .set_number_of_retries(1)
        .set_max_delay(MAX_BACKOFF)
        .set_connection_timeout(Some(Duration::from_secs(2)))
        .set_response_timeout(Some(Duration::from_secs(5)));
    Ok(client.get_connection_manager_with_config(config).await?)
}

async fn blocking_connection(client: &redis::Client) -> Result<ConnectionManager, BusError> {
    let config = ConnectionManagerConfig::new()
        .set_number_of_retries(1)
        .set_max_delay(MAX_BACKOFF)
        .set_connection_timeout(Some(Duration::from_secs(2)))
        .set_response_timeout(Some(Duration::from_millis(BLOCK_MS as u64 + 4000)));
    Ok(client.get_connection_manager_with_config(config).await?)
}

// doc/interfaces.md#message-format
pub(crate) async fn xadd(conn: &mut ConnectionManager, stream: &str, maxlen: usize, data: &str) -> Result<String, BusError> {
    Ok(redis::cmd("XADD")
        .arg(stream)
        .arg("MAXLEN")
        .arg("~")
        .arg(maxlen)
        .arg("*")
        .arg("data")
        .arg(data)
        .query_async(conn)
        .await?)
}

async fn newest_id(conn: &mut ConnectionManager, stream: &str) -> Result<String, BusError> {
    let v: RedisValue = redis::cmd("XREVRANGE").arg(stream).arg("+").arg("-").arg("COUNT").arg(1).query_async(conn).await?;
    let ids: Vec<StreamId> = match v {
        RedisValue::Nil => Vec::new(),
        v => redis::streams::StreamRangeReply::from_redis_value(v).map_err(redis::RedisError::from)?.ids,
    };
    Ok(ids.first().map(|e| e.id.clone()).unwrap_or_else(|| "0-0".into()))
}

async fn xread(conn: &mut ConnectionManager, stream: &str, last_id: &str) -> Result<Vec<StreamId>, BusError> {
    let v: RedisValue = redis::cmd("XREAD")
        .arg("COUNT")
        .arg(BATCH)
        .arg("BLOCK")
        .arg(BLOCK_MS)
        .arg("STREAMS")
        .arg(stream)
        .arg(last_id)
        .query_async(conn)
        .await?;
    stream_entries(v)
}

async fn xreadgroup(
    conn: &mut ConnectionManager,
    stream: &str,
    group: &str,
    consumer: &str,
    start: &str,
    block: bool,
) -> Result<Vec<StreamId>, BusError> {
    let mut cmd = redis::cmd("XREADGROUP");
    cmd.arg("GROUP").arg(group).arg(consumer).arg("COUNT").arg(BATCH);
    if block {
        cmd.arg("BLOCK").arg(BLOCK_MS);
    }
    let v: RedisValue = cmd.arg("STREAMS").arg(stream).arg(start).query_async(conn).await?;
    stream_entries(v)
}

fn stream_entries(v: RedisValue) -> Result<Vec<StreamId>, BusError> {
    if let RedisValue::Nil = v {
        return Ok(Vec::new());
    }
    let reply = StreamReadReply::from_redis_value(v).map_err(redis::RedisError::from)?;
    Ok(reply.keys.into_iter().flat_map(|k| k.ids).filter(|e| !e.map.is_empty()).collect())
}

async fn create_group(conn: &mut ConnectionManager, stream: &str, group: &str) -> Result<(), BusError> {
    let r: Result<(), redis::RedisError> =
        redis::cmd("XGROUP").arg("CREATE").arg(stream).arg(group).arg("$").arg("MKSTREAM").query_async(conn).await;
    match r {
        Err(e) if e.code() == Some("BUSYGROUP") => Ok(()),
        r => Ok(r?),
    }
}

// doc/shared_libraries.md#redis-bus
async fn claim_all(conn: &mut ConnectionManager, stream: &str, group: &str, consumer: &str) -> Result<(), BusError> {
    let mut start = "0-0".to_string();
    loop {
        let v: RedisValue = redis::cmd("XAUTOCLAIM")
            .arg(stream)
            .arg(group)
            .arg(consumer)
            .arg(0)
            .arg(&start)
            .arg("COUNT")
            .arg(BATCH)
            .arg("JUSTID")
            .query_async(conn)
            .await?;
        let next = match &v {
            RedisValue::Array(items) => items.first().cloned().map(String::from_redis_value).transpose().map_err(redis::RedisError::from)?,
            _ => None,
        };
        match next {
            Some(n) if n != "0-0" => start = n,
            _ => return Ok(()),
        }
    }
}

fn entry_data(entry: &StreamId) -> Option<String> {
    entry.map.get("data").and_then(|v| String::from_redis_value(v.clone()).ok())
}
