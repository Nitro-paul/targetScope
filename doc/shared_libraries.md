# Shared libraries

The interfaces between programs are defined language-neutrally in `schemas/` (JSON Schema for messages, a C header for the frame header; see [interfaces.md](interfaces.md#language-neutral-contract)). Each language has one shared library that implements this contract, so programs never define message types themselves:

| Library | Language | Used by |
|---|---|---|
| `targetscope_common` | Rust | launcher, database_handler, session_manager, analyzer, statistics, exporter, importer, web_server |
| `targetscope_common_cpp` | C++ | camera_reader, image_processor |

Both are tested against the same schemas and examples (contract tests), so a Rust program and a C++ program always agree on every message. A program in a further language would get its own library of the same shape.

# targetscope_common (Rust)

Library crate shared by all Rust programs. A change of a message or frame type is a compile error in every Rust program that uses it, and a contract test failure if it no longer matches the schema.

## Responsibilities
- Model types: `Session`, `NewSession`, `Shot`, `SessionShot`, `PaperImage`, `Shooter`, `Device`, `TargetType`, `Ring`, `Setting`, `Filter`, `SessionFilter`, `Corners`, `AnalysisResult`, `CurrentSession`, `SessionSetup`, `CurrentShot`, `CalibrationStatus`, `CameraControl`, `StatisticsGroup`, `StatisticsValues`
- Ring generation from the three anchors of a target type
- Message types: request, response and event envelopes, and one request enum per receiving program
- Redis bus: connecting, consumer groups, sending requests and waiting for responses, serving requests, publishing and reading events
- iceoryx2 frame services: names, `FrameHeader`, publisher and subscriber setup
- Geometry: polar ↔ Cartesian conversion
- Bootstrap configuration, settings access and logging setup
- Time source (`Clock`) and failpoints for tests

It contains no program logic.

## Module structure
```
crates/targetscope_common/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── model.rs        # model types (serde)
    ├── messages/
    │   ├── mod.rs      # RequestEnvelope, ResponseEnvelope, EventEnvelope, ErrorCode, ErrorBody
    │   ├── database_handler.rs
    │   ├── camera_reader.rs
    │   ├── image_processor.rs
    │   ├── session_manager.rs
    │   ├── analyzer.rs
    │   ├── statistics.rs
    │   ├── exporter.rs
    │   ├── importer.rs
    │   └── launcher.rs
    ├── bus.rs          # Bus, RequestHandler, EventReader
    ├── idempotency.rs  # RedisIdempotencyStore
    ├── failpoints.rs   # failpoint setup and evaluation
    ├── frames.rs       # iceoryx2 services, FrameHeader, PixelFormat
    ├── geometry.rs     # Polar, Point, conversions
    ├── targets.rs      # ring generation, target type validation
    ├── settings.rs     # SettingsCache
    ├── config.rs       # BootstrapConfig
    ├── export.rs       # export file format v1 (exporter, importer)
    ├── program.rs      # program startup, shutdown signal
    ├── clock.rs        # Clock, SystemClock, local time
    └── logging.rs      # tracing setup, daily log file
```

## Messages
Every program has a request enum and, if it sends events, an event enum, in `messages/<program>.rs`. The enums are adjacently tagged (`type` + `payload`), so they match the JSON of the envelopes. Example:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum AnalyzerRequest {
    Ping(Ping),
    AnalyzeShots(AnalyzeShots),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ImageProcessorEvent {
    CalibrationChanged(CalibrationStatus),
    PaperReplaced(PaperReplaced),
    ShotDetected(ShotDetected),
}
```
Every request is a struct (its payload) that implements `Request`, which links it to its type name, the receiver's stream, its response type and its timeout:
```rust
pub trait Request: Serialize {
    type Response: DeserializeOwned;
    const TYPE: &'static str;      // e.g. "analyze_shots"
    const STREAM: &'static str;    // e.g. "analyzer:requests"
    const TIMEOUT: Duration;       // per request type, see interfaces.md#timeouts
}
```
Each response payload has its own struct (e.g. `CreateSessionResponse { id, uuid, images }`); requests whose response is a shared type use it directly (`analyze_shots` → `AnalysisResult`), empty responses use `Empty {}`.

The envelopes keep the payload as a JSON value, so they can be read before the request type is known:
```rust
pub struct RequestEnvelope { r#type: String, version: u32, request_id: Uuid, reply_to: Option<String>, payload: Value }
pub struct ResponseEnvelope { r#type: String, version: u32, request_id: Uuid, status: Status, error: Option<ErrorBody>, payload: Option<Value> }
pub struct EventEnvelope { r#type: String, version: u32, event_id: Uuid, source: String, timestamp_ms: i64, payload: Value }
```
- `RequestEnvelope::decode::<R>()` turns `type` + `payload` into the program's request enum. An unknown `type` or a `version` other than 1 gives `invalid_message`; a payload that doesn't fit its type gives `validation_failed`.
- `ok_response(payload)` and `error_response(&ServiceError)` build the response with `type` = request type + `_response` and the same `request_id` and `version`.
- `EventEnvelope::new(source, timestamp_ms, &event)` sets a new `event_id`; `decode::<E>()` gives the event enum.

**Optional fields:** a missing field and `null` both deserialize to `None`. Fields that are always part of a type (e.g. `Session.notes`) are written as `null` when not set; optional request fields (e.g. `reply_to`, `corners` of `detect_shots_in_photo`, the SessionFilter fields) are left out when not set.

## Redis bus
```rust
#[derive(Clone)]
pub struct Bus { /* shared: command connection, program name, consumer name, clock, pending-response map */ }

impl Bus {
    /// Connects and starts a background task that reads `<program>:responses` (starting after its
    /// newest entry) and completes waiting requests by request_id.
    pub async fn connect(url: &str, program: &str) -> Result<Self, BusError>;
    pub async fn connect_with_clock(url: &str, program: &str, clock: Arc<dyn Clock>) -> Result<Self, BusError>;

    /// Sends a request with a new request_id and waits for the response (Request::TIMEOUT).
    pub async fn request<R: Request>(&self, req: R) -> Result<R::Response, BusError>;
    /// Same, but with a given request_id, for resending.
    pub async fn request_with_id<R: Request>(&self, id: Uuid, req: R) -> Result<R::Response, BusError>;
    /// Sends `ping` to any program (used by the launcher).
    pub async fn ping(&self, program: &str, timeout: Duration) -> Result<PingResponse, BusError>;
    /// Sends a prepared envelope and returns the full response envelope (used by web_server
    /// and the test kit, which forward requests they don't decode).
    pub async fn send_envelope(&self, stream: &str, env: RequestEnvelope, timeout: Duration)
        -> Result<ResponseEnvelope, BusError>;

    /// Sends an event to <program>:events.
    pub async fn publish<E: Serialize>(&self, event: &E) -> Result<EventEnvelope, BusError>;
    /// Id of the newest entry of <source>:events ("0-0" if empty), noted before loading state.
    pub async fn newest_event_id(&self, source: &str) -> Result<String, BusError>;
    /// Reads <source>:events, with a consumer group named after this program (no event lost)
    /// or as a plain reader after a given id (see interfaces.md#who-listens-to-which-events).
    pub fn events<E: DeserializeOwned>(&self, source: &str, mode: ReadMode) -> EventReader<E>;

    /// Request loop: reads pending, then new requests; calls the handler; sends the response;
    /// acknowledges. Answers `ping` itself. Parsing errors are answered with invalid_message.
    pub async fn serve<Req, H>(&self, handler: H) -> Result<(), BusError>
    where Req: DeserializeOwned, H: RequestHandler<Req>;
    /// Same, but handles up to `limit` requests at the same time, for stateless programs.
    pub async fn serve_concurrent<Req, H>(&self, handler: Arc<H>, limit: usize) -> Result<(), BusError>;
}

pub enum ReadMode { ConsumerGroup, After(String) }

/// EventReader::next() returns the next event; ack(entry_id) acknowledges it in group mode.
pub struct Event<E> { pub entry_id: String, pub envelope: EventEnvelope, pub event: E }

#[async_trait]
pub trait RequestHandler<Req> {
    async fn handle(&self, request_id: Uuid, req: Req) -> Result<serde_json::Value, ServiceError>;
}

pub enum BusError { Redis(redis::RedisError), Timeout, Remote(ErrorBody), Decode(String) }
```
- Every stream entry has one field `data` with the envelope as JSON. Writes use `XADD … MAXLEN ~ n` with the caps of [interfaces.md](interfaces.md#streams-and-keys).
- Responses are read by the background task with `XREAD BLOCK` on `<program>:responses`, matched by `request_id`, and delivered through a `oneshot` channel; unknown request ids (e.g. late responses after a timeout) are dropped and logged.
- Blocking reads (`XREAD BLOCK`, `XREADGROUP BLOCK`, 1 s) use their own connections, so they never delay other commands.
- Reconnects use `redis::aio::ConnectionManager`; read loops retry with exponential backoff (100 ms up to 5 s), timed by the `Clock`.
- `serve` handles requests one at a time, in stream order (session_manager, launcher, importer). `serve_concurrent` starts a task per request, at most `limit` at once, so `ping` and short requests are answered while long ones run (analyzer: 16, database_handler: 8, statistics: 4, exporter: 2); each task sends its response and acknowledges its entry.
- `Bus::stop()` ends `serve` and `serve_concurrent` after the requests being handled are answered (used on SIGTERM); `Bus::reconnects()` is a `watch` channel that changes every time the request loop had to reconnect to Redis (database_handler then sends `data_changed` `all`/`reload`).
- `RequestEnvelope::decode` answers a request whose `type` the program doesn't know, or whose `version` isn't 1, with `invalid_message`; a known type whose payload doesn't fit (missing or wrongly typed field, unknown enum value) with `validation_failed`. An entry whose `data` isn't JSON is logged and acknowledged without a response. A request with an unknown `type` or `version` is answered with `invalid_message` (if it has `reply_to`) and acknowledged. Every request is handled inside a `tracing` span with its `request_id`, so all log lines of a request contain it.
- The consumer name is `<hostname>-<pid>`. Because a restarted program has a new pid, `serve` and group-mode event readers first claim all pending entries of the group (`XAUTOCLAIM … 0`) and handle them before new ones (only one instance of each program runs). The consumer group of a request stream is created with `XGROUP CREATE … 0 MKSTREAM`, so requests written before the program first started are handled too; the consumer group of an event reader starts at `$` (only events after its creation). `BUSYGROUP` is ignored.
- Event entries whose `type` the reader doesn't know are skipped (and acknowledged in group mode).

### Idempotency
```rust
/// Redis key "<program>:processed:<request_id>" with the response, TTL 24 h.
pub struct RedisIdempotencyStore { .. }
impl RedisIdempotencyStore {
    pub async fn get(&self, id: &Uuid) -> Result<Option<serde_json::Value>, BusError>;
    /// Adds "SET key value EX 86400" to an existing MULTI pipeline.
    pub fn record(&self, pipe: &mut redis::Pipeline, id: &Uuid, response: &serde_json::Value);
}
```
Used by `session_manager` and `launcher`; `database_handler` uses its own SQLite table instead.

## Frames
```rust
pub const FRAMES_RAW: &str = "targetscope/frames/raw";
pub const FRAMES_LIVE: &str = "targetscope/frames/live";

pub struct FrameHeader { .. }      // mirror of schemas/frames/frame_header.h, #[type_name("TargetScopeFrameHeader")]
pub enum PixelFormat { Bgr8 = 1 }

pub fn frame_publisher(node: &Node<ipc::Service>, service: &str, max_len: usize) -> Result<FramePublisher, FrameError>;
pub fn frame_subscriber(node: &Node<ipc::Service>, service: &str) -> Result<FrameSubscriber, FrameError>;
/// Removes resources of crashed iceoryx2 nodes.
pub fn cleanup_dead_nodes() -> Result<(), FrameError>;
```
`FramePublisher::publish(header, |buf: &mut [u8]| ...)` loans a slice, lets the caller fill it, sends it and notifies the matching `/notify` event service. `FrameSubscriber::wait(timeout)` blocks on the event service and returns the newest sample; if the subscriber already holds the maximum number of samples (2), it returns the newest one it could take. iceoryx2 `ipc` ports can't move between threads, so a subscriber is created in the thread that uses it. Details in [frame_transport.md](frame_transport.md).

## Export file
`export.rs` holds the types of the export file format version 1 (`ExportFile`, `ExportShooter`, `ExportTargetType`, `ExportDevice`, `ExportSession`, `ExportImage`), written by `exporter` and read by `importer`, so both always agree. The format is defined by `schemas/export/v1.schema.json` (see [exporter.md](programs/exporter.md#json)); `session_image_entry` and `device_image_entry` build the image paths inside a ZIP.

## Geometry
```rust
pub struct Polar { pub angle_turns: f64, pub radius_mm: f64 }  // same fields as Shot
pub struct Point { pub x: f64, pub y: f64 }     // mm, origin at ring center, x right, y up

impl From<Polar> for Point;   // x = radius_mm·sin(2π·angle_turns), y = radius_mm·cos(2π·angle_turns)
impl From<Point> for Polar;   // radius_mm = √(x²+y²), angle_turns = atan2(x, y) / 2π, normalized to [0, 1); radius 0 → angle 0
```

## Target types
```rust
/// One ring per score from inner_white_score down to outer_score, radii spaced equally
/// between neighboring anchors (formula in interfaces.md#targettype).
pub fn generate_rings(t: &TargetType) -> Vec<Ring>;
/// All TargetType rules: radii increasing, scores decreasing, outermost ring on the paper.
pub fn validate_target_type(t: &TargetType) -> Result<(), String>;
```
The same two functions exist in `targetscope_common_cpp`; both are tested with the same examples (`schemas/examples/target_types/*.json` with their expected rings).

## Contract tests
`tests/contract.rs` runs, for every schema in `schemas/`: validate its examples (`jsonschema` crate), deserialize them into the Rust type, serialize back and compare as JSON values, and validate the serialized output; every invalid example in `schemas/examples/invalid/` must be rejected. A message type without a schema, or a schema without a Rust type, fails the test. All cases: [testing/shared_libraries.md](testing/shared_libraries.md).

## Bootstrap configuration
Values needed before the database can be read. The launcher sets them as environment variables for every program (see [launcher](programs/launcher.md#bootstrap-environment)).
```rust
pub struct BootstrapConfig {
    pub redis_url: String,     // TARGETSCOPE_REDIS_URL, default redis://127.0.0.1:6379/
    pub data_dir: PathBuf,     // TARGETSCOPE_DATA_DIR, default ~/.local/share/targetscope
    pub db_path: PathBuf,      // TARGETSCOPE_DB_PATH (database_handler only), default <data_dir>/targetscope.db
    pub log_dir: PathBuf,      // TARGETSCOPE_LOG_DIR, default ~/.local/state/targetscope/logs
    pub log_level: String,     // TARGETSCOPE_LOG, default "info" (tracing EnvFilter syntax)
}
```

## Settings cache
```rust
/// Notes the newest id of database_handler:events, reads all settings through database_handler,
/// then keeps them up to date by reading data_changed events for the settings table after that id
/// (plain read, so no change between loading and listening is missed). A `data_changed` with
/// table "all" reloads every setting.
pub struct SettingsCache { .. }
impl SettingsCache {
    pub async fn load(bus: &Bus) -> Result<Self, BusError>;
    /// None if the setting is unset (active = false) or unknown.
    pub fn get(&self, name: &str) -> Option<String>;
    pub fn get_parsed<T: FromStr>(&self, name: &str) -> Option<T>;
    pub fn get_path(&self, name: &str) -> Option<PathBuf>;   // expands "~"
    /// Notifies the program when one of the given settings changes.
    pub fn watch(&self, names: &[&str]) -> tokio::sync::watch::Receiver<()>;
}
```

## Clock and failpoints
All code that depends on time (timeouts, backoff, retention, periods, running time) reads it through a `Clock`, so tests can control time:
```rust
pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;              // wall clock (timestamps, saved_at, periods)
    fn monotonic(&self) -> Instant;           // durations and timeouts
    async fn sleep(&self, d: Duration);
}
pub struct SystemClock;                       // the real clock, used in production
```
The test kit (`crates/targetscope_testkit`, see [testing/overview.md](testing/overview.md#test-kit)) adds `FakeClock`, whose time only moves when a test advances it.

**Failpoints:** test builds contain named failpoints (`fail` crate, enabled by the cargo feature `failpoints`) at the places listed in [testing/overview.md](testing/overview.md#failpoints). A program calls `failpoints::setup()` at startup, which reads the `FAILPOINTS` environment variable (e.g. `FAILPOINTS="db.before_commit=return"`). `failpoints::hit(name)` evaluates a failpoint: `sleep(<ms>)` waits, `return` makes the code at that place fail, and `return(abort)` aborts the process. Release builds don't contain them; `hit` is then always `false`.

### Local time
Periods, log file dates and exported timestamps use the local time zone of the PC:
```rust
pub fn unix_s(t: SystemTime) -> i64;              // and unix_ms
pub fn local(unix_s: i64) -> LocalDateTime;       // date, time, weekday (Monday = 0), UTC offset
pub fn local_midnight(date: LocalDate) -> i64;    // Unix seconds of 00:00 local time on that day
pub fn rfc3339_local(unix_s: i64) -> String;      // e.g. 2026-11-11T14:30:00+01:00
```
They use the C library (`localtime_r`, `mktime`), so they follow the system's time zone, including daylight saving time.

## Program startup
Every Rust program starts the same way with `program::start(name)`: `failpoints::setup()`, `BootstrapConfig::from_env()`, `logging::init`, then connect the bus, retrying with backoff (100 ms up to 5 s) until Redis answers. `program::shutdown_signal()` resolves on SIGTERM or SIGINT; `program::stop_on_signal(&bus)` calls `bus.stop()` then, so the program finishes the requests it is handling, answers them and exits with code 0.

## Logging
`logging::init(program, &config)` sets up `tracing` with two outputs, both filtered by `log_level` (`EnvFilter` syntax): stderr (collected by the launcher) and the file `<log_dir>/<program>.log`. The file is rotated daily by the shared `Clock`: at the first line of a new local day, the old file is renamed to `<program>.log.<yyyy-mm-dd>` (its date) and a new `<program>.log` is started; old files are kept. Every log line of a request contains its `request_id` (see [Redis bus](#redis-bus)).

## Dependencies
`serde`, `serde_json`, `uuid`, `redis` (features `tokio-comp`, `streams`, `connection-manager`), `tokio`, `async-trait`, `iceoryx2`, `thiserror`, `tracing`, `tracing-subscriber`, `fail`, `hostname`, `libc`; for tests `jsonschema`.

# C++ library (targetscope_common_cpp)

Static library shared by the C++ programs. It offers the same functions as the Rust crate, in C++ style.

## Module structure
```
cpp/targetscope_common/
├── CMakeLists.txt
├── include/targetscope/
│   ├── model.hpp        # model types as structs, with nlohmann::json to_json/from_json
│   ├── messages.hpp     # envelopes, ErrorCode, request/event types of camera_reader and image_processor,
│   │                    # plus the requests these programs send (save_setting, read_settings, …)
│   ├── bus.hpp          # Bus: requests with timeout, serve loop, publish/read events
│   ├── frames.hpp       # iceoryx2 services, FrameHeader wrapper with IOX2_TYPE_NAME
│   ├── geometry.hpp     # Polar, Point, conversions
│   ├── targets.hpp      # generate_rings, validate_target_type
│   ├── settings.hpp     # SettingsCache
│   ├── config.hpp       # BootstrapConfig (same environment variables)
│   ├── clock.hpp        # Clock, SystemClock (same as in Rust)
│   └── logging.hpp      # spdlog setup: stderr + daily file
├── src/
└── tests/               # contract tests and unit tests (GoogleTest)
```

## Bus
```cpp
class Bus {
public:
    Bus(const std::string& redis_url, std::string program);

    /// Sends a request and waits for the response (timeout per request type).
    template <class Req> typename Req::Response request(const Req& req);

    /// Request loop on <program>:requests in its own thread: reads pending, then new requests,
    /// calls the handler, sends the response, acknowledges. Answers ping itself.
    void serve(std::function<nlohmann::json(const std::string& type, const nlohmann::json& payload)> handler);

    void publish(const std::string& type, const nlohmann::json& payload);
};
```
Handlers report errors by throwing `ServiceError{ErrorCode, message}`; the serve loop turns them into error responses. Responses to the program's own requests are read by a background thread from `<program>:responses` and matched by `request_id`.

## Libraries
| Purpose | Library |
|---|---|
| Redis | redis-plus-plus (on hiredis) |
| JSON | nlohmann/json |
| UUIDs | own UUID v4 generator (`new_uuid()` in `messages.hpp`), no library needed |
| Frames | iceoryx2 C++ bindings |
| Logging | spdlog |
| Tests | GoogleTest, json-schema-validator (pboettch) for contract tests |

Dependencies are fetched with CMake `FetchContent` (or found as system packages, where available), so the C++ programs build with one `cmake --build`.

## Contract tests
Same procedure as in Rust: every schema in `schemas/` that a C++ program uses is checked against its valid and invalid examples and the C++ types; `static_assert`s check size and field offsets of `TargetScopeFrameHeader`. Failpoints use the `TS_FAILPOINT` macro, compiled into test builds only. All cases: [testing/shared_libraries.md](testing/shared_libraries.md) and [testing/frame_transport.md](testing/frame_transport.md).
