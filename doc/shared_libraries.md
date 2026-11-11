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
    ├── bus.rs          # Bus, RequestServer, EventReader
    ├── idempotency.rs  # RedisIdempotencyStore
    ├── frames.rs       # iceoryx2 services, FrameHeader, PixelFormat
    ├── geometry.rs     # Polar, Point, conversions
    ├── targets.rs      # ring generation, target type validation
    ├── settings.rs     # SettingsCache
    ├── config.rs       # BootstrapConfig
    └── logging.rs      # tracing setup
```

## Messages
Every program has a request enum and, if it sends events, an event enum. Example:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum AnalyzerRequest {
    Ping {},
    AnalyzeShots { shots: Vec<Shot>, target_type: TargetType, bullet_diameter_mm: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ImageProcessorEvent {
    CalibrationChanged(CalibrationStatus),
    PaperReplaced { live_paper: u32, previous_paper_staged_file: Option<String> },
    ShotDetected { hole_id: u32, live_paper: u32, angle_turns: f64, radius_mm: f64, confidence: f32, frame_seq: u64 },
}

pub struct RequestEnvelope<R> {
    pub version: u32,
    pub request_id: Uuid,
    pub reply_to: Option<String>,
    pub request: R,                // flattened: "type" + "payload"
}

pub struct ResponseEnvelope {
    pub r#type: String,
    pub version: u32,
    pub request_id: Uuid,
    pub status: Status,            // Ok | Error
    pub error: Option<ErrorBody>,  // { code: ErrorCode, message: String, details: Option<Value> }
    pub payload: Option<serde_json::Value>,
}

pub struct EventEnvelope<E> {
    pub version: u32,
    pub event_id: Uuid,
    pub source: String,
    pub timestamp_ms: i64,
    pub event: E,                  // flattened: "type" + "payload"
}
```
Each response payload has its own struct (e.g. `CreateSessionResponse { id, uuid, images }`). The trait `Request` links a request to its response type, so `Bus::request` returns the right type:
```rust
pub trait Request: Serialize {
    type Response: DeserializeOwned;
    const STREAM: &'static str;    // e.g. "analyzer:requests"
    const TIMEOUT: Duration;       // per request type, see interfaces.md#timeouts
}
```
Message structs implement `Request` and are wrapped into the program's enum when sent.

## Redis bus
```rust
pub struct Bus { /* redis::aio::ConnectionManager, program name, pending-response map */ }

impl Bus {
    /// Connects and starts a background task that reads `<program>:responses`
    /// and completes waiting requests by request_id.
    pub async fn connect(url: &str, program: &str) -> Result<Self, BusError>;

    /// Sends a request with a new request_id and waits for the response (Request::TIMEOUT).
    pub async fn request<R: Request>(&self, req: R) -> Result<R::Response, BusError>;

    /// Same, but with a given request_id, for resending.
    pub async fn request_with_id<R: Request>(&self, id: Uuid, req: R) -> Result<R::Response, BusError>;

    /// Sends an event to <program>:events.
    pub async fn publish<E: Serialize>(&self, event: E) -> Result<(), BusError>;

    /// Reads <source>:events, with a consumer group (no event lost) or as a plain reader
    /// (starting after a given id, see interfaces.md#who-listens-to-which-events).
    pub fn events<E: DeserializeOwned>(&self, source: &str, mode: ReadMode) -> EventReader<E>;

    /// Request loop: reads pending, then new requests; calls the handler; sends the response;
    /// acknowledges. Answers `ping` itself. Parsing errors are answered with invalid_message.
    pub async fn serve<Req, H>(&self, handler: H) -> Result<(), BusError>
    where Req: DeserializeOwned, H: RequestHandler<Req>;
}

#[async_trait]
pub trait RequestHandler<Req> {
    async fn handle(&self, request_id: Uuid, req: Req) -> Result<serde_json::Value, ServiceError>;
}

pub enum BusError { Redis(redis::RedisError), Timeout, Remote(ErrorBody), Decode(String) }
```
- Responses are read by the background task with `XREAD BLOCK` on `<program>:responses`, matched by `request_id`, and delivered through a `oneshot` channel; unknown request ids (e.g. late responses after a timeout) are dropped.
- Reconnects use `redis::aio::ConnectionManager` with exponential backoff.
- The consumer name is `<hostname>-<pid>`.

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
`FramePublisher::publish(header, |buf: &mut [u8]| ...)` loans a slice, lets the caller fill it, sends it and notifies the matching `/notify` event service. `FrameSubscriber::wait(timeout)` blocks on the event service and returns the newest sample. Details in [frame_transport.md](frame_transport.md).

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
`tests/contract.rs` runs, for every schema in `schemas/`: validate its examples (`jsonschema` crate), deserialize them into the Rust type, serialize back and compare as JSON values, and validate the serialized output. A message type without a schema, or a schema without a Rust type, fails the test.

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
/// (plain read, so no change between loading and listening is missed).
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

## Logging
`logging::init(program)` sets up `tracing` with two outputs: stderr (collected by the launcher) and a daily rotated file `<log_dir>/<program>.log` (`tracing-appender`). Every log line of a request contains its `request_id`.

## Dependencies
`serde`, `serde_json`, `uuid`, `redis` (features `tokio-comp`, `streams`, `connection-manager`), `tokio`, `async-trait`, `iceoryx2`, `thiserror`, `tracing`, `tracing-subscriber`, `tracing-appender`; for tests `jsonschema`.

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
| UUIDs | stduuid |
| Frames | iceoryx2 C++ bindings |
| Logging | spdlog |
| Tests | GoogleTest, json-schema-validator (pboettch) for contract tests |

Dependencies are fetched with CMake `FetchContent` (or found as system packages, where available), so the C++ programs build with one `cmake --build`.

## Contract tests
Same procedure as in Rust: every schema in `schemas/` that a C++ program uses is checked against its examples and the C++ types; `static_assert`s check size and field offsets of `TargetScopeFrameHeader`.
