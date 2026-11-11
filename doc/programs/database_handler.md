# database_handler

Program `database_handler` (Rust): the only program that accesses the SQLite database ([database.md](../database.md)) and the only one that writes to the image folder. It serves the `database_handler` requests in [interfaces.md](../interfaces.md#database_handler).

## Responsibilities
- Owns the SQLite database file: creates it, applies schema migrations, inserts default settings, and enables foreign keys on its connection.
- Owns the image folder: moves images from the staging folder into it, names them, and deletes them with their record.
- Reads requests from the `database_handler:requests` stream (consumer group `database_handler`).
- Validates every request before touching the database.
- Makes sure no request is handled twice (duplicate protection by `request_id`).
- Executes create, read and delete operations on sessions, shooters, devices, target types and settings, and the import requests of `importer`.
- Treats saved sessions as records: there is no update of sessions or shots; values a saved session depends on (target type geometry, device bullet diameter) are locked (see [database.md](../database.md#records)).
- Creates `uuid`s and sets `sessions.saved_at` when a session is saved; keeps both from the file on import.
- Maintains `devices.shot_count` by adding the session's `shot_count` when a session is saved; never decreases it.
- Adds the generated `rings` and the `locked` flag to target types and devices it returns.
- Translates Filters into safe, parameterized SQL.
- Sends a response to the request's `reply_to` stream and acknowledges the request.
- Sends a `data_changed` event after every successful write.
- Defines the known settings (default value, default `active`, optional, value rule); checks every saved value against its rule; protects the known settings from deletion; resets settings to their defaults (`reset_settings`).
- Changes the image folder and moves the images on request (`change_image_dir`).
- Cleans up: old `processed_requests` entries and old files in the staging folder.
- Logs every request and error.

It does not compute scores or analyze data; that is the analyzer's job.

## Technology
| Purpose | Crate |
|---|---|
| Shared types, Redis helpers | `targetscope_common` ([shared_libraries.md](../shared_libraries.md)) |
| Async runtime | `tokio` |
| SQLite | `rusqlite` (feature `bundled`) |
| Errors | `thiserror` |
| Logging | `tracing` |

## Architecture
`rusqlite` is synchronous and its `Connection` cannot be shared between threads. The handler therefore runs in two parts:

```
                 tokio runtime                          dedicated thread
 +-------------------------------------------+      +----------------------+
 |  StreamWorker                              |      |  DbWorker            |
 |  XREADGROUP -> parse -> validate --------- | ---> |  owns Connection     |
 |                                  DbCommand |      |  runs queries        |
 |  XADD reply_to <- Response <-------------- | <--- |  moves image files   |
 |  XADD database_handler:events              |      |  (oneshot reply)     |
 |  XACK                                      |      |                      |
 +-------------------------------------------+      +----------------------+
```

- **StreamWorker** (async): reads from Redis, parses and validates requests, sends a `DbCommand` over a `tokio::sync::mpsc` channel, awaits the result on a `tokio::sync::oneshot` channel, writes the response and event, and acknowledges. Uses the request loop of `targetscope_common::bus`.
- **DbWorker** (blocking, `std::thread`): owns the only `rusqlite::Connection` and processes commands one at a time. This serializes all writes, which SQLite requires anyway.

## Module structure
```
programs/database_handler/
├── Cargo.toml
└── src/
    ├── main.rs            # load config, init logging, open DB, start workers and maintenance
    ├── error.rs           # HandlerError
    ├── filter_sql.rs      # Filter -> SQL builder
    ├── validation.rs      # validation rules per request
    ├── stream_worker.rs   # request handling on top of targetscope_common::bus
    ├── db_worker.rs       # DbCommand, DbWorker thread
    ├── images.rs          # ImageStore: staging -> image folder, delete, move to a new image folder
    ├── known_settings.rs  # list of known settings: name, display name, default value, default active, optional, value rule
    ├── image_dir_change.rs  # change_image_dir: background copy, switch, cleanup
    ├── maintenance.rs     # daily purge of processed_requests and staging folder
    └── db/
        ├── mod.rs         # Database: open connection, pragmas
        ├── schema.rs      # migrations, default settings
        ├── sessions.rs    # session queries
        ├── shooters.rs    # shooter queries
        ├── devices.rs     # device queries
        ├── target_types.rs  # target type queries
        ├── settings.rs    # setting queries
        └── processed_requests.rs  # processed_requests queries
```
Model types (`Session`, `Shot`, `Device`, `TargetType`, `Ring`, `Setting`), `Filter`, the message enums and envelopes come from `targetscope_common`.

## Core types

### Configuration
Bootstrap values come from environment variables set by the launcher (see [shared_libraries.md](../shared_libraries.md#bootstrap-configuration)), because they're needed before the database is open: `TARGETSCOPE_DB_PATH`, `TARGETSCOPE_DATA_DIR` (for the default folder settings) and `TARGETSCOPE_REDIS_URL`. All other configuration is in the settings table.

### Filter to SQL
```rust
/// Builds "WHERE ... ORDER BY ... LIMIT ... OFFSET ..." and its parameters.
/// `columns` is the column whitelist of the queried table with each column's type;
/// any other field is rejected, and `like` is only allowed on text columns.
/// Values are always bound as parameters, never formatted into SQL.
pub fn filter_to_sql(filter: &Filter, columns: &[(&str, ColumnType)])
    -> Result<(String, Vec<rusqlite::types::Value>), HandlerError>;

/// Same for sessions: builds the WHERE clause for a SessionFilter, with a join on devices
/// for device_categories and calibers, and on shooters/devices/target_types for include_retired.
/// An empty list becomes a condition that matches nothing.
pub fn session_filter_to_sql(filter: &SessionFilter)
    -> (String, Vec<rusqlite::types::Value>);
```

### Errors
```rust
#[derive(thiserror::Error, Debug)]
pub enum HandlerError {
    #[error("invalid message: {0}")]     InvalidMessage(String),
    #[error("validation failed: {0}")]   ValidationFailed(String),
    #[error("not found: {0}")]           NotFound(String),
    #[error("in use: {0}")]              InUse(String),
    #[error("io error: {0}")]            Io(#[from] std::io::Error),
    #[error("database error: {0}")]      Database(#[from] rusqlite::Error),
}

impl HandlerError {
    /// Error code sent in the response, see interfaces.md
    pub fn code(&self) -> ErrorCode;
}
```

### Database and queries
```rust
pub struct Database { conn: rusqlite::Connection }

impl Database {
    /// Opens the file, sets pragmas, runs migrations, inserts default settings.
    pub fn open(path: &Path) -> Result<Self, HandlerError>;
    pub fn transaction(&mut self) -> Result<rusqlite::Transaction<'_>, HandlerError>;
}

// All query functions take a connection or transaction (`&rusqlite::Connection`;
// a Transaction derefs to it), so the DbWorker controls the transaction.

pub mod sessions {
    /// Inserts session (uuid created by the caller, saved_at = now), shots and session_images
    /// (image paths already known), adds shot_count to the device. Rows are never updated afterwards.
    pub fn create(c: &Connection, uuid: Uuid, s: &NewSession, images: &[(u32, String)]) -> Result<i64, HandlerError>;
    /// Inserts an imported session with its uuid and saved_at; does not touch devices.shot_count.
    /// Returns (id, created); created = false if the uuid exists.
    pub fn import(c: &Connection, s: &Session, images: &[(u32, String)]) -> Result<(i64, bool), HandlerError>;
    pub fn exists(c: &Connection, uuid: Uuid) -> Result<Option<i64>, HandlerError>;
    /// Returns the page of sessions and the total number of matches.
    pub fn find(c: &Connection, f: &SessionFilter, order: &SessionOrder, page: Page, include_shots: bool) -> Result<(Vec<Session>, u64), HandlerError>;
    /// Returns the deleted session's image paths, so the files can be removed after commit.
    pub fn delete(c: &Connection, id: i64) -> Result<Vec<String>, HandlerError>;
}

pub mod shooters {
    /// Insert (new uuid) or update (name, retired); the name is trimmed and must be unique ignoring case.
    pub fn save(c: &Connection, s: &Shooter) -> Result<i64, HandlerError>;
    /// Uses an existing shooter with the same uuid or, failing that, the same name (ignoring case); returns (id, created).
    pub fn import(c: &Connection, s: &Shooter) -> Result<(i64, bool), HandlerError>;
    pub fn find(c: &Connection, f: &Filter) -> Result<Vec<Shooter>, HandlerError>;
    /// Fails with InUse if a session references the shooter.
    pub fn delete(c: &Connection, id: i64) -> Result<(), HandlerError>;
}

pub mod devices {
    /// Insert (new uuid) if `id` is None, otherwise update; never overwrites uuid, shot_count or
    /// image fields; returns InUse if bullet_diameter_mm changes on a locked device.
    pub fn save(c: &Connection, d: &Device) -> Result<i64, HandlerError>;
    /// Creates the device with its uuid and shot_count unless the uuid exists; returns (id, created).
    pub fn import(c: &Connection, d: &Device) -> Result<(i64, bool), HandlerError>;
    pub fn is_locked(c: &Connection, id: i64) -> Result<bool, HandlerError>;
    pub fn set_image(c: &Connection, id: i64, image_path: Option<&str>) -> Result<Option<String>, HandlerError>;
    pub fn find(c: &Connection, f: &Filter) -> Result<Vec<Device>, HandlerError>;
    /// Fails with InUse if a session references the device; returns image_path.
    pub fn delete(c: &Connection, id: i64) -> Result<Option<String>, HandlerError>;
}

pub mod target_types {
    /// Insert (new uuid) or update; returns InUse if anything but the name changes on a locked type.
    pub fn save(c: &Connection, t: &TargetType) -> Result<i64, HandlerError>;
    /// Creates the type with its uuid unless the uuid exists; renames on name conflict; returns (id, created).
    pub fn import(c: &Connection, t: &TargetType) -> Result<(i64, bool), HandlerError>;
    /// Returns the types with generated rings (targetscope_common::targets::generate_rings) and `locked`.
    pub fn find(c: &Connection, f: &Filter) -> Result<Vec<TargetType>, HandlerError>;
    /// Fails with InUse if a session references the type.
    pub fn delete(c: &Connection, id: i64) -> Result<(), HandlerError>;
}

pub mod settings {
    /// INSERT ... ON CONFLICT(name) DO UPDATE
    pub fn save(c: &Connection, s: &Setting) -> Result<(), HandlerError>;
    /// Sets value and active of the given known settings back to their defaults.
    pub fn reset(c: &Connection, names: &[&str]) -> Result<(), HandlerError>;
    pub fn find(c: &Connection, f: &Filter) -> Result<Vec<Setting>, HandlerError>;
    pub fn get(c: &Connection, name: &str) -> Result<Option<String>, HandlerError>;
    pub fn delete(c: &Connection, name: &str) -> Result<(), HandlerError>;
}

pub mod processed_requests {
    /// Stored response payload if `request_id` was already processed.
    pub fn find(c: &Connection, request_id: &Uuid) -> Result<Option<serde_json::Value>, HandlerError>;
    /// Must be called inside the transaction of the write it records.
    pub fn record(c: &Connection, request_id: &Uuid, r#type: &str,
                  response: &serde_json::Value) -> Result<(), HandlerError>;
    /// Deletes entries older than `max_age_days`; returns the number deleted.
    pub fn purge(c: &Connection, max_age_days: u32) -> Result<usize, HandlerError>;
}
```

### ImageStore
```rust
pub struct ImageStore { image_dir: PathBuf, staging_dir: PathBuf }

impl ImageStore {
    /// Checks that `file` is a plain file name (no path separators, no ".."; else ValidationFailed),
    /// exists in the staging folder (else NotFound), and is a JPEG or PNG by magic bytes (else ValidationFailed).
    pub fn check_staged(&self, file: &str) -> Result<ImageKind, HandlerError>;
    /// Atomic rename from the staging folder to `image_dir/<sub>/<name>`.
    pub fn move_in(&self, file: &str, sub: &str, name: &str) -> Result<String, HandlerError>;
    /// Moves a file back to the staging folder (used when a transaction fails).
    pub fn move_back(&self, image_path: &str, file: &str) -> Result<(), HandlerError>;
    /// Deletes a file from the image folder; a missing file is only logged.
    pub fn delete(&self, image_path: &str);
    /// Switches to a new image folder (used by the image folder change, see below).
    pub fn set_image_dir(&mut self, new_dir: PathBuf);
}
```
Folder paths come from the settings `image_dir` and `staging_dir`; the handler updates them when it saves these settings. `image_dir` is only changed through `change_image_dir` (or `reset_settings`), which can move the images: the UI asks the user in a popup (see [web_server](web_server.md#settings-view)). Images that weren't moved are shown as "image missing" until the user moves them by hand.

### Image folder change
Copying thousands of images can take minutes. So that sessions can still be saved meanwhile, the copy doesn't run in the DbWorker:
1. **Copy** (background task, `spawn_blocking`): copy all files of `devices/` and `sessions/` into the new folder. The DbWorker keeps handling requests; new images still go to the old folder.
2. **Switch** (one DbWorker command, short): copy the files added since step 1, delete copies of files deleted since step 1, save the setting, `ImageStore::set_image_dir`. From now on new images go to the new folder.
3. **Clean up** (background task): delete the copied files in the old folder. Files that couldn't be copied stay there and are reported in the response.

Without `move_images`, only step 2 (save the setting) runs. Only one image folder change can run at a time; a second one gets `invalid_state`.

### DbWorker
```rust
pub struct DbCommand {
    pub request_id: Uuid,
    pub request: DatabaseRequest,          // from targetscope_common::messages
    pub reply: oneshot::Sender<Result<DbOutcome, HandlerError>>,
}

pub struct DbOutcome {
    pub response: serde_json::Value,
    pub event: Option<DataChanged>,        // None for reads and duplicates
}

/// Starts the thread; it owns `db` and `images` and runs until the sender is dropped.
pub fn spawn_db_worker(db: Database, images: ImageStore) -> mpsc::Sender<DbCommand>;
```

## Startup
1. Initialize logging and read the bootstrap configuration.
2. `Database::open`: create the file and parent directory if missing; set `PRAGMA foreign_keys = ON`, `PRAGMA journal_mode = WAL`, `PRAGMA busy_timeout = 5000`; run migrations; insert default settings (folder defaults from `TARGETSCOPE_DATA_DIR`). No target types are inserted: they are always created by the user from a photo (see [target type editor](web_server.md#target-type-editor)) or imported.
3. Create the image folder (`devices/`, `sessions/`) and the staging folder if missing.
4. Start the DbWorker thread.
5. Connect to Redis, create the consumer group (`XGROUP CREATE ... $ MKSTREAM`, ignoring `BUSYGROUP`), and start the StreamWorker. From now on `ping` is answered, which tells the launcher the program is ready.
6. Run maintenance at startup and then once an hour: purge `processed_requests` older than `processed_requests_retention_days`, and delete staging files (including leftover `.part` files) older than `staging_max_age_hours`.
7. On SIGTERM/SIGINT: finish the current request and exit.

## Request processing
For each request:
1. Read pending requests of this consumer first (`XREADGROUP ... 0`), then new ones (`XREADGROUP ... BLOCK 5000 ... >`).
2. Parse the `data` field into a request envelope. On failure: log, send `invalid_message` if `reply_to` can be read, `XACK`.
3. Answer `ping` directly.
4. Validate (`validation.rs`). On failure: respond `validation_failed`, `XACK`.
5. Send a `DbCommand` to the DbWorker and await the result. For write requests the DbWorker:
   1. Begins a transaction.
   2. Looks up `request_id` in `processed_requests`. If found: rolls back and returns the stored response payload (the request is a duplicate and is not executed again).
   3. Executes the write. If the request includes an image, moves the image from the staging folder into the image folder.
   4. Records `request_id`, type and response payload in `processed_requests`.
   5. Commits. If anything fails, the transaction is rolled back and a moved image is moved back to the staging folder.
   6. After the commit, deletes image files of deleted records.

   Read requests are executed directly; running them twice is harmless.
6. Respond to `reply_to` (if set) with `ok` or the error code.
7. After a successful write: send `data_changed` to `database_handler:events` (one event per changed record; `reset_settings` sends one per reset setting).
8. `XACK` the request.

### Image handling per request
| Request | Image file action |
|---|---|
| `create_session` / `import_session` with `images` | Moves each to `sessions/<uuid>_<paper_no>.<ext>`. The uuid is known before the insert, so the image paths are part of the insert and the record is never updated. `import_session` for an existing uuid leaves the files in the staging folder for the importer to delete |
| `delete_session` | Deletes all `sessions/<uuid>_<paper_no>.<ext>` of the session after commit |
| `import_device` with `staged_file` | Only if the device is created: moves it to `devices/<id>_<unix seconds>.<ext>` and sets `image_path`; otherwise the file is left in the staging folder for the importer to delete |
| `set_device_image` | Moves it to `devices/<id>_<unix seconds>.<ext>`; sets `image_path`; deletes the previous image after commit |
| `delete_device_image` | Sets `image_path = NULL`; deletes the file after commit |
| `delete_device` | Deletes the device's image after commit |
| `change_image_dir` / `reset_settings` with `move_images` | Copies all images to the new folder in the background, switches, then deletes the old copies (see [Image folder change](#image-folder-change)); other requests keep working meanwhile |

### Validation rules
- `create_session`: at least one shot; `shot_count == shots.len()`; `paper_count >= 1`; every `paper_no` of shots and images in `1 ..= paper_count`, at most one image per paper; `notes` at most 10 000 characters; every image passes `check_staged`; `device_id`, `target_type_id` and (if given) `shooter_id` exist; for every shot `0 <= angle_turns < 1` and `radius_mm >= 0`; `distance_m` and `duration_s` are `>= 0` if given.
- `save_device`: `name` not empty; `bullet_diameter_mm > 0`; unchanged `bullet_diameter_mm` if the device is locked.
- `import_*`: valid uuid; same rules as the corresponding save or create request; `import_session` additionally needs `saved_at`; `staged_file`, if given, passes `check_staged`.
- `set_device_image`: device exists; `staged_file` passes `check_staged`.
- `save_target_type`: `name` not empty and unique; all TargetType rules in [interfaces.md](../interfaces.md#targettype) (paper size, center and outermost ring on the paper, radii increasing, scores decreasing); only `name` changed if the type is locked.
- `save_shooter`: name not empty after trimming, unique ignoring case.
- `save_setting`: `name` and `display_name` not empty; the value matches the setting's value rule (see [known settings](../database.md#known-settings)); `active = false` only for optional settings; not `image_dir` (use `change_image_dir`); for `staging_dir`, `export_dir`: the folder exists or can be created and is writable; `staging_dir` is on the same file system as `image_dir` (same `st_dev`).
- `change_image_dir`: the new folder exists or can be created, is writable, and is on the same file system as `staging_dir`.
- `reset_settings`: every name is a known setting. Without names: all known settings except `admin_password_hash` and `lens_calibrations`.
- `delete_setting`: not one of the [known settings](../database.md#known-settings).
- All read requests: every filter field is a column of the table; `like` only on text columns.

## Error handling
| Situation | Response | Acknowledge |
|---|---|---|
| Unparseable message, validation error, not found, in use | error response | yes, retrying cannot succeed |
| Database or file error (e.g. disk full, database locked after timeout) | `database_error` / `io_error` | yes, after logging |
| Redis connection lost | none | no; reconnect with backoff and reprocess pending requests |
| Handler crashes while processing | none | no; the request is pending and is reprocessed on restart |

Duplicates are safe: if the handler crashes after committing but before `XACK`, the request is reprocessed, its `request_id` is found in `processed_requests`, and the stored response is sent again without a second write. Only successful writes are recorded. Failed requests leave no record, so a resend is validated and executed again; deterministic errors (`validation_failed`, `not_found`, `in_use`) then give the same result, and a transient `database_error` may now succeed.

If the handler crashes between commit and deleting an image file, the file stays as an orphan. Orphans are only logged at startup (files in the image folder without a record), never deleted automatically.

## Schema migrations
The schema version is stored in `PRAGMA user_version`. `schema.rs` holds an ordered list of migration SQL scripts. On startup, every script with an index greater than `user_version` runs in its own transaction, and `user_version` is updated after each one. Migration 1 is the schema in [database.md](../database.md#schema-sql). Default settings are inserted after the migrations with `INSERT ... ON CONFLICT(name) DO NOTHING`, so user changes are kept.

## Testing
- Query and filter tests against an in-memory database (`Connection::open_in_memory`) with all migrations applied.
- Validation tests without a database.
- Duplicate tests: sending each write request twice with the same `request_id` stores it once and returns the same response both times.
- Paper tests: images and shots with `paper_no` outside `1 ..= paper_count` are rejected; all images of a session are moved and deleted together.
- Record tests: every way of changing a saved session, a used target type's geometry or a used device's bullet diameter is rejected (by the handler with `in_use`, and directly in SQL by the triggers); `shot_count` stays after `delete_session`.
- Import tests: same uuid twice creates one record; name conflicts get the suffix.
- Image tests in a temporary folder: move in, move back on failed transaction, delete; rejection of `../` and non-image files.
- Integration tests with a local Redis: send each request type and check the response and the `data_changed` event.

## Future: backups
Not part of this version; see [database.md](../database.md#future-backups). A `backup` request can be added to this program without changing anything else.
