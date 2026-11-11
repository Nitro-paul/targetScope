# Database

## Concept
All persistent values are stored in a single SQLite database file. It contains the tables `settings`, `sessions`, `shots`, `session_images`, `shooters`, `devices`, `target_types` and `processed_requests`, which are filled during setup or at runtime. Only the database handler ([programs/database_handler.md](programs/database_handler.md)) accesses the database directly; all other programs go through it using the messages in [interfaces.md](interfaces.md).

Images (device photos and session target images) are stored as files in the image folder (setting `image_dir`); the database only holds their path relative to that folder (`image_path`). Only the database handler writes to the image folder; `web_server` (to serve images) and `exporter` (to put images into ZIP exports) only read it.

```
image_dir/
├── devices/
│   └── <device id>_<unix seconds>.<jpg|png>    # timestamp changes on replace, so browsers don't show a cached old image
└── sessions/
    └── <session uuid>_<paper_no>.<jpg|png>      # transformed target paper, one per paper of the session
```

SQLite specifics:
- Types are `INTEGER`, `REAL` and `TEXT`. Booleans are stored as `INTEGER` (`0` = false, `1` = true).
- `id INTEGER PRIMARY KEY` columns are assigned automatically on insert.
- Foreign keys are only enforced with `PRAGMA foreign_keys = ON`, which the database handler sets on every connection.

## Records
Saved sessions with their shots and images are **records**: they are never changed after saving. The database enforces this with triggers (see [schema](#schema-sql)):
- `UPDATE` on `sessions`, `shots` and `session_images` is rejected. A session can only be deleted as a whole (with its shots and images).
- Values that the scoring of a saved session depends on are locked as soon as a session uses them:
  - the geometry and scores of a target type (only `name` and `retired` stay editable),
  - the `bullet_diameter_mm` of a device.
- `devices.shot_count` never decreases, also not when a session is deleted.

To change a locked target type, the user saves a copy as a new target type.

Every session, shooter, device and target type has a `uuid`, its permanent identity across export and import.

## Relations
- `sessions.shooter_id` → `shooters.id` (optional). A shooter that is used by a session cannot be deleted.
- `sessions.device_id` → `devices.id`. A device that is used by a session cannot be deleted.
- `sessions.target_type_id` → `target_types.id`. A target type that is used by a session cannot be deleted.
- `shots.session_id` → `sessions.id`, `ON DELETE CASCADE`: deleting a session also deletes its shots.
- `session_images.session_id` → `sessions.id`, `ON DELETE CASCADE`: deleting a session also deletes its image rows (the database handler deletes the files).

## Tables

### settings
Stores all application settings, such as paths and names. Programs look up a setting by `name`, its unique machine-readable key (e.g. `image_dir`); `display_name` is the human-readable label shown in the UI. See [known settings](#known-settings).

`active` says whether the setting is set:
- `true`: the setting is set, programs use `value`.
- `false`: the setting is not set; `value` is ignored (it's kept as a suggestion that the UI prefills). Only **optional** settings can be inactive. Normal use works without them; a program that needs one answers `setting_not_set`, and the UI asks the user to set it (e.g. `export_dir` before the first export).

Whether a setting is optional, and its default value and default `active` state, are defined by the database handler for every known setting (table below) and returned with every setting. `reset_settings` restores the defaults.
```
+-------------------------------+
|           settings            |
+-------------------------------+
|   id: INTEGER (primary key)   |
|   name: TEXT (unique)         |
|   display_name: TEXT          |
|   active: BOOLEAN             |
|   value: TEXT                 |
+-------------------------------+
```

### sessions
Holds one record per shooting session. `device_id` links the session to the device used, `target_type_id` to the target type.
- `uuid`: permanent identity, created when the session is saved (kept on import)
- `saved_at`: Unix timestamp in seconds, set by the database handler when the session is saved (kept on import)
- `score`, `grouping_score`: calculated by the analyzer (see [programs/analyzer.md](programs/analyzer.md))
- `distance_m`: shooting distance in meters
- `shooter_id`: the shooter, optional
- `shot_count`: number of shots in the session, at least 1
- `duration_s`: running time in seconds, without pauses; `NULL` for photo sessions
- `notes`: free text, at most 10 000 characters
- `paper_count`: number of target papers used in the session (a paper can be changed during a session), at least 1
```
+-------------------------------+
|           sessions            |
+-------------------------------+
|   id: INTEGER (primary key)   |
|   uuid: TEXT (unique)         |
|   saved_at: INTEGER           |
|   score: INTEGER              |
|   grouping_score: INTEGER     |
|   distance_m: INTEGER         |
|   target_type_id: INTEGER (FK)|
|   shooter_id: INTEGER (FK)    |
|   device_id: INTEGER (FK)     |
|   single_handed: BOOLEAN      |
|   supported: BOOLEAN          |
|   shot_count: INTEGER         |
|   duration_s: INTEGER         |
|   notes: TEXT                 |
|   paper_count: INTEGER        |
+-------------------------------+
```

### shots
Stores every shot of every session, linked to its session by `session_id`. A shot's position is stored in polar coordinates around the ring center:
- `angle_turns`: angle in turns (1 turn = full circle), starting at 0 at the top and increasing clockwise (0.25 = right); range `0 <= angle_turns < 1`
- `radius_mm`: distance from the ring center in millimeters
- `shot_no`: order of the shot in the session, starting at 1
- `paper_no`: target paper the shot is on, `1 ..= sessions.paper_count`
```
+-------------------------------+
|             shots             |
+-------------------------------+
|   id: INTEGER (primary key)   |
|   session_id: INTEGER (FK)    |
|   shot_no: INTEGER            |
|   paper_no: INTEGER           |
|   angle_turns: REAL           |
|   radius_mm: REAL             |
+-------------------------------+
```

### session_images
Stores the target image of each paper of a session: the transformed paper (straight view, cropped to the paper, without drawing). Input mode `camera`: taken at the paper change, or at `session_stop` for the last paper; input mode `photo`: the transformed photo. A paper can have no image (e.g. input mode `manual`).
- `image_path`: e.g. `sessions/<uuid>_2.jpg`
```
+-------------------------------+
|        session_images         |
+-------------------------------+
|   session_id: INTEGER (FK)    |
|   paper_no: INTEGER           |
|   image_path: TEXT            |
+-------------------------------+
   primary key (session_id, paper_no)
```

### shooters
Stores the people who shoot. Sessions reference them by id, so renaming a shooter (e.g. to fix a typo) shows the new name in all sessions without changing any record.
- `uuid`: permanent identity
- `name`: unique, ignoring upper/lower case (`Alex` and `alex` can't both exist); leading and trailing spaces are removed before saving
- `retired`: hidden in the session setup, still shown in history, statistics and exports
```
+-------------------------------+
|           shooters            |
+-------------------------------+
|   id: INTEGER (primary key)   |
|   uuid: TEXT (unique)         |
|   name: TEXT (unique)         |
|   retired: BOOLEAN            |
+-------------------------------+
```

### devices
Stores all devices (weapons) in use.
- `uuid`: permanent identity
- `category`: kind of device, e.g. `air pistol` (free text)
- `description`: free text, e.g. model, sights, modifications
- `caliber`: caliber designation, e.g. `.22 LR`
- `image_path`: device image, e.g. `devices/3_1794393600.jpg`; `NULL` if there is none
- `bullet_diameter_mm`: bullet diameter in millimeters, used for scoring (a shot scores a ring if its edge touches the ring) and hole detection; locked once a session uses the device
- `shot_count`: lifetime total of shots fired with the device; the database handler adds the session's shots each time a session is saved; never decreases
- `retired`: hidden in the session setup (e.g. a sold device), still shown in history, statistics and exports
```
+-------------------------------+
|            devices            |
+-------------------------------+
|   id: INTEGER (primary key)   |
|   uuid: TEXT (unique)         |
|   name: TEXT                  |
|   category: TEXT              |
|   description: TEXT           |
|   caliber: TEXT               |
|   image_path: TEXT            |
|   bullet_diameter_mm: REAL    |
|   shot_count: INTEGER         |
|   retired: BOOLEAN            |
+-------------------------------+
```

### target_types
Stores the paper targets that can be used in a session. `name` is unique ignoring upper/lower case. Target types are always created by the user in the web UI from a photo of the target (see [web_server](programs/web_server.md#target-type-editor)), or come from an import. No target types are inserted on first start.

All positions are in millimeters on the paper, measured from its top-left corner (x to the right, y down):
- `paper_width_mm`, `paper_height_mm`: size of the paper; the transformed images show exactly this area
- `center_x_mm`, `center_y_mm`: position of the ring center on the paper
- three anchor rings, each with radius and score:
  - `inner_white_radius_mm`, `inner_white_score`: white center ring (typically score 12)
  - `black_radius_mm`, `black_score`: edge of the black aiming area and the score of the ring ending there
  - `outer_radius_mm`, `outer_score`: outermost ring (typically score 1)

The scoring rings are not stored. They are generated from the anchors: one ring per score, spaced equally between two anchors (formula in [interfaces.md](interfaces.md#targettype)). Once a session uses a target type, all columns except `name` and `retired` are locked. `retired` hides the type in the session setup.
```
+---------------------------------+
|          target_types           |
+---------------------------------+
|   id: INTEGER (primary key)     |
|   uuid: TEXT (unique)           |
|   name: TEXT (unique)           |
|   paper_width_mm: REAL          |
|   paper_height_mm: REAL         |
|   center_x_mm: REAL             |
|   center_y_mm: REAL             |
|   inner_white_radius_mm: REAL   |
|   inner_white_score: INTEGER    |
|   black_radius_mm: REAL         |
|   black_score: INTEGER          |
|   outer_radius_mm: REAL         |
|   outer_score: INTEGER          |
|   retired: BOOLEAN              |
+---------------------------------+
```

### processed_requests
Internal table of the database handler for [duplicate protection](interfaces.md#duplicate-protection-idempotency). Stores the `request_id` of every successfully processed write request together with the response payload that was sent, so a resent request gets the same response without being processed again.
- `type`: interface name, e.g. `create_session`
- `processed_at`: Unix timestamp in seconds
- `response`: payload of the `ok` response as JSON, e.g. `{"id": 42}`

Entries older than the retention period (setting `processed_requests_retention_days`, default 30) are deleted by the database handler.
```
+-------------------------------+
|      processed_requests       |
+-------------------------------+
|   request_id: TEXT (PK)       |
|   type: TEXT                  |
|   processed_at: INTEGER       |
|   response: TEXT              |
+-------------------------------+
```

## Schema (SQL)
The database handler creates this schema on first start (see [schema migrations](programs/database_handler.md#schema-migrations)).
```sql
CREATE TABLE settings (
    id            INTEGER PRIMARY KEY,
    name          TEXT    NOT NULL UNIQUE,
    display_name  TEXT    NOT NULL,
    active        INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    value         TEXT
);

CREATE TABLE shooters (
    id       INTEGER PRIMARY KEY,
    uuid     TEXT    NOT NULL UNIQUE,
    name     TEXT    NOT NULL UNIQUE COLLATE NOCASE CHECK (name <> '' AND name = trim(name)),
    retired  INTEGER NOT NULL DEFAULT 0 CHECK (retired IN (0, 1))
);

CREATE TABLE devices (
    id                  INTEGER PRIMARY KEY,
    uuid                TEXT    NOT NULL UNIQUE,
    name                TEXT    NOT NULL,
    category            TEXT,
    description         TEXT,
    caliber             TEXT,
    image_path          TEXT,
    bullet_diameter_mm  REAL    NOT NULL CHECK (bullet_diameter_mm > 0),
    shot_count          INTEGER NOT NULL DEFAULT 0 CHECK (shot_count >= 0),
    retired             INTEGER NOT NULL DEFAULT 0 CHECK (retired IN (0, 1))
);

CREATE TABLE target_types (
    id                     INTEGER PRIMARY KEY,
    uuid                   TEXT    NOT NULL UNIQUE,
    name                   TEXT    NOT NULL UNIQUE COLLATE NOCASE CHECK (name <> '' AND name = trim(name)),
    paper_width_mm         REAL    NOT NULL CHECK (paper_width_mm > 0),
    paper_height_mm        REAL    NOT NULL CHECK (paper_height_mm > 0),
    center_x_mm            REAL    NOT NULL,
    center_y_mm            REAL    NOT NULL,
    inner_white_radius_mm  REAL    NOT NULL CHECK (inner_white_radius_mm > 0),
    inner_white_score      INTEGER NOT NULL,
    black_radius_mm        REAL    NOT NULL,
    black_score            INTEGER NOT NULL,
    outer_radius_mm        REAL    NOT NULL,
    outer_score            INTEGER NOT NULL CHECK (outer_score >= 1),
    retired                INTEGER NOT NULL DEFAULT 0 CHECK (retired IN (0, 1)),
    CHECK (inner_white_radius_mm < black_radius_mm AND black_radius_mm < outer_radius_mm),
    CHECK (inner_white_score > black_score AND black_score > outer_score),
    CHECK (center_x_mm - outer_radius_mm >= 0 AND center_x_mm + outer_radius_mm <= paper_width_mm),
    CHECK (center_y_mm - outer_radius_mm >= 0 AND center_y_mm + outer_radius_mm <= paper_height_mm)
);

CREATE TABLE sessions (
    id              INTEGER PRIMARY KEY,
    uuid            TEXT    NOT NULL UNIQUE,
    saved_at        INTEGER NOT NULL,
    score           INTEGER NOT NULL,
    grouping_score  INTEGER NOT NULL,
    distance_m      INTEGER CHECK (distance_m >= 0),
    target_type_id  INTEGER NOT NULL REFERENCES target_types(id),
    shooter_id      INTEGER REFERENCES shooters(id),
    device_id       INTEGER NOT NULL REFERENCES devices(id),
    single_handed   INTEGER NOT NULL DEFAULT 0 CHECK (single_handed IN (0, 1)),
    supported       INTEGER NOT NULL DEFAULT 0 CHECK (supported IN (0, 1)),
    shot_count      INTEGER NOT NULL CHECK (shot_count >= 1),
    duration_s      INTEGER CHECK (duration_s >= 0),
    notes           TEXT    CHECK (length(notes) <= 10000),
    paper_count     INTEGER NOT NULL CHECK (paper_count >= 1)
);

CREATE TABLE shots (
    id          INTEGER PRIMARY KEY,
    session_id  INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    shot_no     INTEGER NOT NULL CHECK (shot_no >= 1),
    paper_no    INTEGER NOT NULL CHECK (paper_no >= 1),
    angle_turns REAL    NOT NULL CHECK (angle_turns >= 0 AND angle_turns < 1),
    radius_mm   REAL    NOT NULL CHECK (radius_mm >= 0),
    UNIQUE (session_id, shot_no)
);

CREATE TABLE session_images (
    session_id  INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    paper_no    INTEGER NOT NULL CHECK (paper_no >= 1),
    image_path  TEXT    NOT NULL,
    PRIMARY KEY (session_id, paper_no)
);

CREATE TABLE processed_requests (
    request_id    TEXT    PRIMARY KEY,
    type          TEXT    NOT NULL,
    processed_at  INTEGER NOT NULL,
    response      TEXT    NOT NULL
);

CREATE INDEX idx_sessions_shooter_id     ON sessions(shooter_id);
CREATE INDEX idx_sessions_device_id      ON sessions(device_id);
CREATE INDEX idx_sessions_target_type_id ON sessions(target_type_id);
CREATE INDEX idx_sessions_saved_at       ON sessions(saved_at);
CREATE INDEX idx_shots_session_id        ON shots(session_id);
CREATE INDEX idx_processed_requests_processed_at ON processed_requests(processed_at);

-- Records: saved sessions, their shots and their images are never changed.
CREATE TRIGGER sessions_are_records BEFORE UPDATE ON sessions
BEGIN SELECT RAISE(ABORT, 'record: sessions cannot be changed'); END;

CREATE TRIGGER shots_are_records BEFORE UPDATE ON shots
BEGIN SELECT RAISE(ABORT, 'record: shots cannot be changed'); END;

CREATE TRIGGER session_images_are_records BEFORE UPDATE ON session_images
BEGIN SELECT RAISE(ABORT, 'record: session images cannot be changed'); END;

-- Values the scoring of saved sessions depends on are locked once used.
CREATE TRIGGER target_types_locked_when_used
BEFORE UPDATE OF paper_width_mm, paper_height_mm, center_x_mm, center_y_mm,
                 inner_white_radius_mm, inner_white_score, black_radius_mm, black_score,
                 outer_radius_mm, outer_score, uuid ON target_types
WHEN EXISTS (SELECT 1 FROM sessions WHERE target_type_id = OLD.id)
BEGIN SELECT RAISE(ABORT, 'in_use: target type is used by a session'); END;

CREATE TRIGGER devices_bullet_locked_when_used
BEFORE UPDATE OF bullet_diameter_mm, uuid ON devices
WHEN EXISTS (SELECT 1 FROM sessions WHERE device_id = OLD.id)
BEGIN SELECT RAISE(ABORT, 'in_use: bullet diameter is used by a session'); END;

CREATE TRIGGER devices_shot_count_never_decreases
BEFORE UPDATE OF shot_count ON devices
WHEN NEW.shot_count < OLD.shot_count
BEGIN SELECT RAISE(ABORT, 'record: shot_count cannot decrease'); END;
```
`paper_no` of shots and images not exceeding `paper_count` is checked by the database handler (SQLite `CHECK` can't refer to another table).

The database handler checks these rules itself before writing and returns `in_use`; the triggers are the last line of defense against programming errors. Because `UPDATE OF` triggers fire whenever a column is in the `SET` list (even with an unchanged value), the handler only includes locked columns in an `UPDATE` when their value actually changes.

A future migration that has to restructure `sessions`, `shots` or `session_images` (SQLite needs a table copy for many changes) drops the record triggers, copies the data unchanged, and creates the triggers again, all in one transaction.

## Known settings
The database handler inserts these settings with their default value and default `active` state on first start (`INSERT ... ON CONFLICT(name) DO NOTHING`), so user changes are kept. Known settings can be changed and reset (`reset_settings`), but not deleted. `save_setting` rejects values that don't match the column *Values* (`validation_failed`), so no program ever reads an unusable value. Paths may start with `~`, which is replaced by the user's home folder; `<data_dir>` is the data folder from the launcher configuration (`TARGETSCOPE_DATA_DIR`).

| name | Default | Optional (default `active`) | Values | Used by | Meaning |
|---|---|---|---|---|---|
| `image_dir` | `<data_dir>/images` | no | folder | database_handler, web_server, exporter | Image folder; changed with `change_image_dir` |
| `staging_dir` | `<data_dir>/staging` | no | folder, same file system as `image_dir` | all programs that pass files | [Staging folder](interfaces.md#staging-folder) |
| `export_dir` | `~/TargetScope/exports` | yes (`false`) | folder | exporter, web_server | Export folder; must be set before the first export |
| `processed_requests_retention_days` | `30` | no | integer 1..=3650 | database_handler | Retention of `processed_requests` |
| `staging_max_age_hours` | `24` | no | integer 1..=8760 | database_handler | Files older than this are deleted from the staging folder |
| `camera_source` | `v4l2` | no | one of `v4l2`, `rtsp`, `none` | camera_reader | Kind of camera |
| `camera_device` | empty | no | text | camera_reader | USB camera, identified by its `bus_info` (from `camera_list_devices`), so it's found again when its device number `/dev/videoN` changes; empty = the first camera found |
| `camera_rtsp_url` | empty | yes (`false`) | URL starting with `rtsp://` | camera_reader | RTSP URL of a network camera; must be set for `camera_source = rtsp` |
| `camera_width_px` | `1920` | no | integer 160..=7680 | camera_reader | Requested frame width |
| `camera_height_px` | `1080` | no | integer 120..=4320 | camera_reader | Requested frame height |
| `camera_fps` | `30` | no | integer 1..=120 | camera_reader | Requested frame rate |
| `camera_controls` | `{}` | no | JSON object | camera_reader | Stored control values per camera: `{ "<camera_key>": { "<control id>": value } }`; written by camera_reader itself; after a reset to `{}`, camera_reader sets every control to the camera's own default |
| `lens_calibrations` | `{}` | no | JSON object | image_processor | Lens calibration per camera: `{ "<camera_key>": { "width_px", "height_px", "camera_matrix": [9 floats], "dist_coeffs": [5 floats], "rms_px" } }`; written by image_processor itself; not reset by "restore all defaults" |
| `live_long_side_px` | `1200` | no | integer 400..=4000 | image_processor | Long side of the transformed live image |
| `capture_long_side_px` | `2400` | no | integer 400..=8000 | image_processor | Long side of stored target images |
| `detection_confirm_frames` | `5` | no | integer 1..=60 | image_processor | Frames a new hole must be visible before it counts as a hole |
| `live_feed_fps` | `15` | no | integer 1..=60 | web_server | Maximum frame rate of the MJPEG feeds |
| `live_feed_quality` | `80` | no | integer 1..=100 | web_server | JPEG quality of the MJPEG feeds |
| `web_bind_address` | `0.0.0.0:8080` | no | `<IPv4 address>:<port>` | web_server | Address and port of the web UI; `0.0.0.0` makes it reachable in the LAN. Takes effect when `web_server` restarts |
| `admin_password_hash` | empty | yes (`false`) | Argon2id hash (PHC string) | web_server | Admin password. Active: destructive actions in the web UI need the password ([web_server](programs/web_server.md#admin-password)). Inactive: everything is open. Never sent to the browser; only changed through the web server's password endpoints; not reset by "restore all defaults" |

The *Values* text is also returned as `value_rule` with every setting, so the UI can show it.

All other settings take effect immediately: the programs listen to `data_changed` for the settings table.

## Future: backups
Backups are not part of this version. The structure keeps them simple to add: all persistent data is in the data folder (database file and image folder), and SQLite's online backup API can copy the database while it's in use. A later backup feature fits in as its own program or as a `backup` request of the database handler.
