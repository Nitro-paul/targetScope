# Interfaces

This document defines all messages between programs over Redis. Video frames are not sent over Redis; see [frame_transport.md](frame_transport.md).

## Concept
Programs communicate through Redis Streams. There are two kinds of messages:
- **Requests and responses**: one program asks another to do something and gets an answer (point to point).
- **Events**: a program announces that something happened, and any number of programs can listen (broadcast).

Request flow:
1. The sender adds a request to the receiver's request stream (`XADD`).
2. The receiver reads the request as part of its consumer group (`XREADGROUP`).
3. The receiver processes the request and, if requested, adds a response to the `reply_to` stream.
4. The receiver acknowledges the request (`XACK`). Redis removes it from the pending list, and the receiver reads the next queued request, if any.

A request that is never acknowledged (e.g. because the receiver crashed) stays in the pending list and is processed again after a restart.

## Language-neutral contract
Every program can be written in the language that suits its task best (Rust by default, C++ for `camera_reader` and `image_processor`). Therefore no interface depends on a language:
- Messages are JSON, and Redis Streams are available in every language.
- The **source of truth** for every message is a JSON Schema file in `schemas/` (see below). This document describes the same messages for humans; if both differ, the schema wins and this document is fixed.
- The shared libraries (`targetscope_common` for Rust, `targetscope_common_cpp` for C++; see [shared_libraries.md](shared_libraries.md)) implement the messages for their language. Contract tests check them against the schemas.
- The iceoryx2 frame header is defined as a C header (`schemas/frames/frame_header.h`); see [frame_transport.md](frame_transport.md#frame-format).

```
schemas/
├── common/            # envelopes, error codes, shared types (Session, Shot, Device, TargetType, …)
├── <program>/         # one file per request, response and event of that program
├── export/v1.schema.json   # export/import file format
├── examples/          # one valid example message per schema, used by the contract tests
└── frames/frame_header.h
```
Schemas use JSON Schema draft 2020-12. Contract tests in every language:
1. Every example in `schemas/examples/` is valid against its schema.
2. Every example can be deserialized into the language's type and serialized back to an equal JSON value.
3. Messages produced by the language's types validate against the schema.
4. Every file in `schemas/examples/invalid/` (each names its expected error) is rejected by its schema and by every language's types.

The complete contract test cases are in [testing/shared_libraries.md](testing/shared_libraries.md#contract-schemas-and-examples-contract).

A program written in a new language needs only: a Redis client, a JSON library, these schemas, and (for video) iceoryx2 bindings.

## Streams and keys
Every program `<program>` uses these streams:

| Stream | Written by | Read by | Content |
|---|---|---|---|
| `<program>:requests` | any program | `<program>`, consumer group `<program>` | Requests to this program |
| `<program>:responses` | the programs `<program>` sent requests to | `<program>` | Responses to this program's requests |
| `<program>:events` | `<program>` | any program | Events of this program |

Only the streams that a program actually uses exist (e.g. `web_server` has no request stream, `analyzer` has no event stream). Every Redis stream and key a program owns starts with its name and a colon (e.g. `session_manager:current`).

Streams are capped with `XADD ... MAXLEN ~ <n>`: 10 000 entries for requests and events, 1 000 for responses.

### Who listens to which events
| Event stream | Listeners | How |
|---|---|---|
| `database_handler:events` | every program with a settings cache (see [shared_libraries.md](shared_libraries.md#settings-cache)), `session_manager`, `web_server` | plain read |
| `camera_reader:events` | `image_processor` (camera identity for the lens calibration), `web_server` | plain read |
| `image_processor:events` | `session_manager` | consumer group |
| | `web_server` | plain read |
| `session_manager:events` | `web_server` | plain read |
| `launcher:events` | `web_server` | plain read |

- **Consumer group** (`XREADGROUP`, group named after the listener): used where no event may be lost (`session_manager` must not lose a `shot_detected`). After a restart, the listener continues where it stopped.
- **Plain read** (`XREAD BLOCK`): used where the listener reloads the full state after a (re)start anyway. The listener first notes the id of the newest entry, then loads its state, then reads all entries after the noted id, and continues from the id of the last entry it received. Nothing is lost between loading and listening.

## Message format
Each stream entry has a single field `data` that holds the message as UTF-8 JSON.

Naming conventions for all fields:
| Convention | Meaning | Examples |
|---|---|---|
| `snake_case` | all field and type names in JSON | `bullet_diameter_mm` |
| suffix `_at` | Unix timestamp in seconds | `saved_at`, `started_at` |
| suffix `_ms` | Unix timestamp in milliseconds | `timestamp_ms`, `updated_ms` |
| suffix `_s` | duration in seconds | `duration_s` |
| suffix `_mm`, `_m`, `_px` | length in millimeters, meters, pixels | `radius_mm`, `distance_m`, `width_px` |
| `staged_file` | plain file name (no path) of a file in the staging folder (see [below](#staging-folder)) | `upload_7c9e….jpg` |
| `image_path` | path of a stored image relative to the image folder | `sessions/7c9e….jpg` |

Optional fields may be missing or `null`; both mean "not set".

Messages must match their schema exactly: a field the schema doesn't define, or a value outside the range the schema allows (e.g. `angle_turns` 1.0, `paper_no` 0, an empty `name`), makes the message invalid. A request like that is answered with `validation_failed`; a listener skips such an event.

### Request envelope
```json
{
  "type": "create_session",
  "version": 1,
  "request_id": "1b4e28ba-2fa1-11d2-883f-0016d3cca427",
  "reply_to": "session_manager:responses",
  "payload": { }
}
```
- `type`: name of the request, e.g. `create_session`
- `version`: interface version, increased when the payload of that request changes incompatibly. All programs are always built and released together from one repository, so mixed versions don't occur in a running system; `version` protects against leftover messages in Redis after an update (they're answered with `invalid_message`)
- `request_id`: UUID v4 chosen by the sender; copied into the response so the sender can match them. It is also the idempotency key (see below)
- `reply_to`: stream for the response; optional, no response is sent if it is missing
- `payload`: request-specific content (below)

### Response envelope
```json
{
  "type": "create_session_response",
  "version": 1,
  "request_id": "1b4e28ba-2fa1-11d2-883f-0016d3cca427",
  "status": "ok",
  "error": null,
  "payload": { }
}
```
- `type`: request type + `_response`
- `status`: `ok` or `error`
- `error`: `null` when `status` is `ok`, otherwise `{ "code": "...", "message": "...", "details": { } }`; `details` is optional and code-specific (e.g. `{ "setting": "export_dir" }` for `setting_not_set`)
- `payload`: response-specific content; `null` when `status` is `error`

### Event envelope
```json
{
  "type": "shot_detected",
  "version": 1,
  "event_id": "9f0c1c9e-6a43-4d3a-9a51-0c3f7d1f2b11",
  "source": "image_processor",
  "timestamp_ms": 1794393600123,
  "payload": { }
}
```
- `version`: as in the request envelope; listeners skip events with another version
- `event_id`: UUID v4; listeners use it to ignore an event they already processed
- `source`: name of the program that sent the event
- `timestamp_ms`: Unix time in milliseconds when the event happened

### Timeouts
The sender waits for a response for at most the time below. If no response arrives, the sender treats it as error `timeout`. It may resend the request with the same `request_id`.

| Requests | Timeout |
|---|---|
| all not listed below | 5 s |
| `session_update_setup`, `session_calibrate`, `session_start`, `session_new_paper`, `session_pause`, `session_resume`, `session_stop`, `session_delete_shot`, `session_save`, `session_cancel` | 30 s |
| `detect_shots_in_photo`, `find_paper`, `prepare_target_photo`, `lens_calibration_finish` | 60 s |
| `session_upload_photo`, `compute_statistics` | 90 s |
| `export_sessions`, `import_file`, `change_image_dir`, `reset_settings` | 300 s |

The timeout belongs to the request type, so every sender uses the same value. A request whose handling waits for other requests (e.g. `session_save` waits for `analyze_shots`, `capture_target_image` and `create_session`) has a timeout longer than the timeouts it waits for, so the outer sender never gives up while the inner request can still succeed.

### Duplicate protection (idempotency)
A message can be delivered more than once, e.g. when a receiver crashes after processing it but before acknowledging it, or when a sender retries after a timeout. `request_id` makes sure such a message is only handled once:
- The sender creates a new `request_id` for every new request and reuses the same `request_id` when it resends the same request.
- The receiver records every processed `request_id` together with the response it sent. If a `request_id` was already processed, the receiver doesn't process it again. It sends the stored response again and acknowledges the request.
- How each program records request ids:

| Program | Record |
|---|---|
| `database_handler` | Table `processed_requests`, in the same transaction as the write |
| `session_manager`, `launcher` | Redis key `<program>:processed:<request_id>` holding the response, expiring after 24 hours. `session_manager` writes it in the same `MULTI`/`EXEC` as the state change |
| `exporter` | File names contain the start of `request_id`; a repeated request returns the existing files |
| `importer` | Not needed: every imported record has a UUID, and records that already exist are skipped |
| `analyzer`, `statistics`, `camera_reader`, `image_processor`, all read requests | Not needed: repeating them has the same effect |

- Events are deduplicated the same way by `event_id`, wherever processing an event twice would matter (`session_manager` for `shot_detected`).

A content hash is not used: two separate sessions with identical values are valid, and a hash can't tell them apart from a resend.

### Error codes
| Code | Meaning |
|---|---|
| `invalid_message` | Message cannot be parsed, or has an unknown `type` or `version` |
| `validation_failed` | Payload values are invalid (see the validation rules of each request) |
| `not_found` | The referenced record or file does not exist |
| `in_use` | The record cannot be deleted or changed because a saved session references it |
| `setting_not_set` | A setting the request needs is not set (`active = false`); `details.setting` names it, so the UI can ask the user to set it |
| `invalid_state` | The request isn't allowed in the current state (e.g. `session_pause` while not running) |
| `unavailable` | A required resource isn't ready (e.g. no camera frames, target not calibrated, analyzer not answering) |
| `processing_failed` | Image processing couldn't produce a result (e.g. no paper found in a photo) |
| `io_error` | Reading or writing a file failed |
| `database_error` | Unexpected database failure |
| `timeout` | Set by the sender when no response arrived in time; never sent by a receiver |

## Staging folder
Files that pass between programs (uploads, captured and transformed images, extracted import images, import files) are written to the staging folder (setting `staging_dir`) and referenced by their plain file name (`staged_file`). The writer first writes to a temporary name (`.<name>.part`) and then renames, so a reader never sees a half-written file. File names show who created them:

| Name | Created by | Content |
|---|---|---|
| `upload_<uuid>.<jpg\|png\|json\|zip>` | `web_server` | Uploaded file |
| `raw_<uuid>.jpg` | `image_processor` (`capture_raw_image`) | Camera frame, not transformed (lens-corrected if a calibration applies) |
| `target_<uuid>.jpg` | `image_processor` (`capture_target_image`, `live_new_paper`, `paper_replaced`) | Transformed paper from the camera, to be stored with a session |
| `photo_<uuid>.jpg` | `image_processor` (`detect_shots_in_photo`) | Transformed paper from a photo, to be stored with a session |
| `paper_<uuid>.jpg` | `image_processor` (`prepare_target_photo`) | Transformed paper for the target type editor |
| `import_<uuid>.<jpg\|png>` | `importer` | Image extracted from an import ZIP |

`database_handler` moves files it stores into the image folder. Files left in the staging folder are deleted after `staging_max_age_hours` (default 24) by `database_handler`.

## Shared types

### Session
A saved session is a **record**: once saved, it is never changed. It can only be deleted as a whole.

| Field | Type | Notes |
|---|---|---|
| `id` | integer | Set by the database handler |
| `uuid` | string (UUID) | Permanent identity, also across export and import; set by the database handler |
| `saved_at` | integer | Unix timestamp in seconds when the session was saved; set by the database handler (kept from the file on import) |
| `score` | integer | Calculated by the analyzer |
| `grouping_score` | integer | Calculated by the analyzer |
| `distance_m` | integer or null | Shooting distance in meters |
| `target_type_id` | integer | References a target type |
| `shooter_id` | integer or null | References a shooter |
| `device_id` | integer | References a device |
| `single_handed` | bool | |
| `supported` | bool | |
| `shot_count` | integer | Number of shots, at least 1; must equal the length of `shots` |
| `duration_s` | integer or null | Running time without pauses; `null` for photo sessions |
| `notes` | string or null | Free text, at most 10 000 characters |
| `paper_count` | integer | Number of target papers used, at least 1 |
| `images` | list of `{ "paper_no": integer, "image_path": string }` | Target image (transformed paper) per paper, e.g. `sessions/<uuid>_2.jpg`; papers without image are missing; set by the database handler |
| `shots` | list of SessionShot | In shot order |

### Shot
A position on the target, in polar coordinates around the ring center.
| Field | Type | Notes |
|---|---|---|
| `angle_turns` | float | Angle in turns, `0 <= angle_turns < 1`, 0 at the top, clockwise (0.25 = right) |
| `radius_mm` | float | Distance from the ring center in millimeters, `>= 0` |

Positions are converted to Cartesian coordinates (millimeters, origin at the ring center, x to the right, y up) with `x = radius_mm · sin(2π · angle_turns)` and `y = radius_mm · cos(2π · angle_turns)`.

### SessionShot
A shot of a saved session: all Shot fields plus
| Field | Type | Notes |
|---|---|---|
| `paper_no` | integer | Target paper the shot is on, `1 ..= paper_count` |

The order of the shots in a list is the shot order; the database stores it as `shots.shot_no` (1, 2, …). Shots of all papers share the same ring-center coordinates, so scores and grouping are calculated over all shots of the session.

### Shooter
| Field | Type | Notes |
|---|---|---|
| `id` | integer | Set by the database handler |
| `uuid` | string (UUID) | Permanent identity; set by the database handler |
| `name` | string | Unique (ignoring upper/lower case), required; leading and trailing spaces are removed |
| `retired` | bool | Hidden in the session setup; still shown in history, statistics and exports |

A shooter can be renamed at any time (e.g. to fix a typo); saved sessions reference the shooter by id, so they show the new name.

### Device
| Field | Type | Notes |
|---|---|---|
| `id` | integer | Set by the database handler |
| `uuid` | string (UUID) | Permanent identity; set by the database handler |
| `name` | string | Required |
| `category` | string or null | Kind of device, free text, e.g. `air pistol` |
| `description` | string or null | Free text, e.g. model, sights, modifications |
| `caliber` | string or null | Caliber designation, e.g. `.22 LR` |
| `bullet_diameter_mm` | float | Used for scoring (edge rule) and hole detection, required, `> 0`. Can't be changed once a saved session uses the device |
| `image_path` | string or null | Device image, e.g. `devices/3_9f0c1c9e.jpg`; set by the database handler, read-only |
| `shot_count` | integer | Lifetime total, maintained by the database handler, read-only, never decreases |
| `locked` | bool | Read-only: `true` if a saved session uses the device (then `bullet_diameter_mm` can't change) |
| `retired` | bool | Hidden in the session setup; still shown in history, statistics and exports |

### TargetType
A target type describes the paper target and its scoring rings. All positions are in millimeters on the paper; the paper origin is its top-left corner, x to the right, y down.

| Field | Type | Notes |
|---|---|---|
| `id` | integer | Set by the database handler |
| `uuid` | string (UUID) | Permanent identity; set by the database handler |
| `name` | string | Unique (ignoring upper/lower case), required |
| `paper_width_mm` | float | Width of the paper |
| `paper_height_mm` | float | Height of the paper |
| `center_x_mm`, `center_y_mm` | float | Position of the ring center on the paper |
| `inner_white_radius_mm` | float | Radius of the white center ring |
| `inner_white_score` | integer | Score of the white center ring, typically 12 |
| `black_radius_mm` | float | Radius of the black aiming area |
| `black_score` | integer | Score of the ring whose outer edge is the edge of the black area |
| `outer_radius_mm` | float | Radius of the outermost ring |
| `outer_score` | integer | Score of the outermost ring, typically 1 |
| `rings` | list of Ring | Read-only, generated from the anchors (see below) |
| `locked` | bool | Read-only: `true` if a saved session uses the type; then only `name` and `retired` can change |
| `retired` | bool | Hidden in the session setup; still shown in history, statistics and exports |

Validation: `inner_white_score > black_score > outer_score >= 1`; `0 < inner_white_radius_mm < black_radius_mm < outer_radius_mm`; the outermost ring lies completely on the paper.

**Ring generation:** the three anchors (inner white, black, outer) each define one ring by its score and outer radius. There is one ring for every score from `inner_white_score` down to `outer_score`. Rings between two anchors are spaced equally: for anchors `(s_a, r_a)` and `(s_b, r_b)` with `s_a > s_b`, the ring with score `s` has the radius
```
r(s) = r_a + (s_a - s) · (r_b - r_a) / (s_a - s_b)
```
Example (scores 12 / 7 / 1): rings 11–8 are spaced equally between the white and black edges, rings 6–2 between the black and outer edges.

`rings` is only output. Every program that needs rings generates them from the anchors with the shared function `generate_rings` ([shared_libraries.md](shared_libraries.md#target-types)); a `rings` field in a request is ignored. Read-only fields (`uuid` except in import requests, `rings`, `locked`, `image_path`, `shot_count` of devices, `optional`/`default_value`/`default_active`/`value_rule` of settings) are ignored in every request.

### Ring
| Field | Type | Notes |
|---|---|---|
| `score` | integer | Score of a shot in this ring |
| `radius_mm` | float | Outer radius of the ring |

Rings are ordered from the inside out (increasing radius, decreasing score).

### Setting
| Field | Type | Notes |
|---|---|---|
| `name` | string | Unique key, required |
| `display_name` | string | Required |
| `active` | bool | `true`: the setting is set and `value` is used. `false`: the setting is not set and `value` is ignored (it's kept as a suggestion for the UI). Only optional settings can be inactive |
| `value` | string | |
| `optional` | bool | Read-only: `true` if the setting may be unset (e.g. `export_dir`). A program that needs an unset optional setting answers `setting_not_set` |
| `default_value` | string | Read-only: value restored by `reset_settings` |
| `default_active` | bool | Read-only: `active` restored by `reset_settings` |
| `value_rule` | string | Read-only: what `value` must look like, e.g. `integer 1..=120`, `one of v4l2, rtsp, none`, `folder`; checked by `save_setting` (see [known settings](database.md#known-settings)) |

### Filter
Used by the read requests of the database handler for shooters, devices, target types and settings. Sessions are selected with the richer [SessionFilter](#sessionfilter).
```json
{
  "conditions": [
    { "field": "shooter_id", "op": "in", "value": [1, 4] },
    { "field": "saved_at", "op": "ge", "value": 1767225600 }
  ],
  "order_by": { "field": "saved_at", "direction": "desc" },
  "limit": 50,
  "offset": 0
}
```
- `conditions`: all conditions must match (AND). An empty list matches every record.
- `field`: a stored column of the requested record type (not `rings`, `locked` or `shots`). Unknown fields are rejected with `validation_failed`.
- `op`: `eq`, `ne`, `lt`, `le`, `gt`, `ge`, `like` (SQL `LIKE` pattern, text fields only), `in` (`value` is a list; matches any of its values; an empty list matches nothing)
- `order_by`: optional; `direction` is `asc` or `desc`
- `limit`, `offset`: optional, for paging

### SessionFilter
Selects saved sessions. Used by `read_sessions`, `export_sessions` and `compute_statistics`, and by the same filter panel in the web UI (history, export, statistics).

Every field is optional; a missing field doesn't restrict. A given list matches any of its values; an empty list matches nothing. All given fields must match (AND).
| Field | Type | Matches sessions … |
|---|---|---|
| `session_ids` | list of integer | with these ids |
| `shooter_ids` | list of integer | of these shooters |
| `device_ids` | list of integer | with these devices |
| `device_categories` | list of string | with devices of these categories |
| `calibers` | list of string | with devices of these calibers |
| `target_type_ids` | list of integer | on these target types |
| `distance_m` | `{ "min": integer, "max": integer }` | within this distance range (each bound optional) |
| `saved_at` | `{ "from": integer, "to": integer }` | saved in this time range (Unix seconds, each bound optional) |
| `single_handed` | bool | shot single-handed or not |
| `supported` | bool | shot supported or not |
| `shot_count` | `{ "min": integer, "max": integer }` | with this number of shots |
| `include_retired` | bool | default `true`; `false` leaves out sessions whose shooter, device or target type is retired |

The database handler resolves device fields (`device_categories`, `calibers`) with a join on `devices`.


### Corners
Four points in image pixel coordinates (origin top-left, x right, y down), used for the paper corners:
```json
{ "top_left": {"x": 410, "y": 95}, "top_right": {"x": 1490, "y": 120},
  "bottom_right": {"x": 1460, "y": 1010}, "bottom_left": {"x": 430, "y": 985} }
```

### AnalysisResult
| Field | Type | Notes |
|---|---|---|
| `shot_scores` | list of integer | Score of each shot, same order as the shots |
| `score` | integer | Sum of `shot_scores` |
| `grouping_score` | integer | Highest possible score after shifting the rings (see [analyzer](programs/analyzer.md)) |
| `grouping_shift` | `{ "x_mm": float, "y_mm": float }` | Shift of the ring center that gives `grouping_score`; `(0, 0)` without shots |
| `mean_point` | `{ "x_mm": float, "y_mm": float }` or null | Mean point of impact, `null` without shots |

### CurrentSession
Snapshot of the session in `session_manager` that is being recorded and not yet saved.
| Field | Type | Notes |
|---|---|---|
| `state` | string | `idle`, `setup`, `running`, `paused`, `review`, `recovered` |
| `recovered_from` | string or null | In state `recovered`: the state before the restart |
| `setup` | SessionSetup | |
| `shots` | list of CurrentShot | In shot order |
| `paper_no` | integer | Current target paper, starting at 1 |
| `paper_images` | list of `{ "paper_no": integer, "staged_file": string }` | Transformed images of papers already finished (camera: taken at the paper change; photo: one per uploaded photo) |
| `notes` | string or null | |
| `analysis` | AnalysisResult or null | Updated live after every shot change |
| `analysis_outdated` | bool | `true` while `analysis` doesn't match the current shots (analysis running, or the analyzer didn't answer) |
| `started_at` | integer or null | Unix timestamp in seconds of the first start |
| `duration_s` | integer | Running time without pauses |
| `calibration` | CalibrationStatus or null | Input mode `camera`: current calibration |

### SessionSetup
| Field | Type | Notes |
|---|---|---|
| `input_mode` | string | How shots are entered: `camera`, `manual` or `photo`. Set by `session_setup`, can't be changed afterwards |
| `shooter_id` | integer or null | |
| `device_id` | integer or null | Required before start |
| `distance_m` | integer or null | Meters |
| `target_type_id` | integer or null | Required before start |
| `single_handed` | bool | |
| `supported` | bool | |

### CurrentShot
| Field | Type | Notes |
|---|---|---|
| `angle_turns` | float | See Shot |
| `radius_mm` | float | See Shot |
| `origin` | string | How this shot was entered: `camera`, `manual` or `photo` (a shot added by hand in review has origin `manual` in every input mode) |
| `confidence` | float or null | Detection confidence (0–1) for `camera` and `photo` shots |
| `hole_id` | integer or null | Origin `camera`: id of the hole in `image_processor` |
| `paper_no` | integer | Target paper the shot is on |

In the UI, shots are labelled per paper (`<paper_no>.<n>`, e.g. `2.3` = third shot on paper 2) as soon as a session has more than one paper; the live view numbers the shots of the current paper the same way.

### CalibrationStatus
| Field | Type | Notes |
|---|---|---|
| `state` | string | `not_calibrated`, `calibrated`, `lost` |
| `method` | string or null | `auto` or `manual` |
| `target_type_id` | integer or null | Target type used for the calibration |
| `corners` | Corners or null | Paper corners in the raw camera frame |
| `px_per_mm` | float or null | Scale of the transformed image |
| `message` | string or null | Reason when not calibrated, e.g. `no camera frames`, `paper not found` |
| `updated_ms` | integer | Unix time in milliseconds |

### CameraControl
| Field | Type | Notes |
|---|---|---|
| `id` | integer | V4L2 control id |
| `name` | string | As reported by the camera, e.g. `Exposure Time, Absolute` |
| `kind` | string | `integer`, `boolean`, `menu` or `button` |
| `min`, `max`, `step`, `default` | integer | For `integer` (and `menu` indices) |
| `value` | integer | Current value |
| `menu` | list of `{ "value": integer, "label": string }` | For `menu` |
| `inactive` | bool | `true` if the control currently has no effect (e.g. exposure while auto exposure is on) |

## Requests understood by all programs

#### ping
Payload: `{}`

Response `ping_response`: `{ "program": "analyzer", "program_version": "0.1.0", "language": "rust", "started_at": <Unix seconds> }`. Used by the launcher to check that a program is ready.

## database_handler
Stream `database_handler:requests`. Design: [programs/database_handler.md](programs/database_handler.md).

Images are passed as `staged_file`: the sender writes the file to the staging folder and sends its name. The database handler checks it (plain name, exists, JPEG or PNG by magic bytes) and moves it into the image folder. A missing `staged_file` gives `not_found`, a wrong one `validation_failed`.

### Sessions
There is no request to change a saved session.

#### create_session
Saves a new session record together with its shots and its target images.

Payload: all Session fields except `id`, `uuid`, `saved_at` and `images`, plus:
- `images`: list of `{ "paper_no": integer, "staged_file": string }`, the transformed target image per paper (may be empty)

Validation: at least one shot; `shot_count` must equal the length of `shots`; every `paper_no` (of shots and images) is within `1 ..= paper_count`, at most one image per paper; `notes` at most 10 000 characters; `device_id`, `target_type_id` and (if given) `shooter_id` must reference existing records; every Shot must be within its ranges; `distance_m` and `duration_s` are `>= 0` if given.

Response `create_session_response`: `{ "id": <new session id>, "uuid": <uuid>, "images": [ { "paper_no", "image_path" }, ... ] }`

Error `not_found` with `details: { "staged_file": <name> }` if a staged image doesn't exist (also for `set_device_image`, `import_device` and `import_session`).

#### read_sessions
Returns all sessions that match a filter, each with its shots.

Payload:
- `filter`: SessionFilter
- `order_by`: optional `{ "field": "saved_at" | "score" | "shot_count" | "id", "direction": "asc" | "desc" }`, default `saved_at desc`
- `limit`, `offset`: optional, for paging
- `include_shots`: bool, default `true`; set to `false` for overview lists

Response `read_sessions_response`: `{ "sessions": [Session, ...], "total": integer }`; `total` is the number of matching sessions without `limit`/`offset`, for paging.

#### delete_session
Deletes a session record completely: the session, its shots and its target images. `devices.shot_count` is not reduced.

Payload: `{ "id": <session id> }`

Response `delete_session_response`: `{ "id": <deleted id> }`. Error `not_found` if the session does not exist.

### Shooters

#### save_shooter
Creates a shooter, or updates it (name, retired) if `id` is given.

Payload: Shooter fields; `id` optional.

Response `save_shooter_response`: `{ "id": <shooter id> }`. Error `validation_failed` if the name is empty or exists already.

#### read_shooters
Payload: `{ "filter": Filter on Shooter fields }`

Response `read_shooters_response`: `{ "shooters": [Shooter, ...] }`

#### delete_shooter
Payload: `{ "id": <shooter id> }`

Response `delete_shooter_response`: `{ "id": <deleted id> }`. Error `in_use` if a session references the shooter.

### Devices

#### save_device
Creates a device, or updates it if `id` is given.

Payload: Device fields; `id` optional.

Response `save_device_response`: `{ "id": <device id> }`. Error `not_found` if `id` is given and does not exist; `in_use` if `bullet_diameter_mm` changes on a locked device.

#### read_devices
Payload: `{ "filter": Filter on Device fields }`

Response `read_devices_response`: `{ "devices": [Device, ...] }`

#### delete_device
Deletes a device and its image.

Payload: `{ "id": <device id> }`

Response `delete_device_response`: `{ "id": <deleted id> }`. Error `in_use` if a session references the device; `not_found` if it does not exist.

#### set_device_image
Sets or replaces the image of a device.

Payload: `{ "device_id": <id>, "staged_file": <file name> }`

Response `set_device_image_response`: `{ "image_path": <path in image folder> }`

#### delete_device_image
Payload: `{ "device_id": <id> }`

Response `delete_device_image_response`: `{ "device_id": <id> }`

### Target types

#### save_target_type
Creates a target type, or updates it if `id` is given.

Payload: TargetType fields; `id` optional.

Validation: see TargetType.

Response `save_target_type_response`: `{ "id": <target type id> }`. Error `in_use` if anything but `name` and `retired` changes on a locked target type. To change a locked type, the UI saves a copy as a new type.

#### read_target_types
Payload: `{ "filter": Filter on TargetType fields }`

Response `read_target_types_response`: `{ "target_types": [TargetType, ...] }`, each with generated `rings` and `locked`.

#### delete_target_type
Payload: `{ "id": <target type id> }`

Response `delete_target_type_response`: `{ "id": <deleted id> }`. Error `in_use` if a session references the target type.

### Import
Used only by `importer`. Records keep the `uuid` from the file, so importing the same file again changes nothing. All import requests answer `{ "id": <local id>, "created": bool }`; if the uuid already exists, the existing id is returned with `created = false` and nothing is changed.

#### import_shooter
Payload: Shooter with `uuid`.

If the uuid is new but a shooter with the same name exists, that shooter is used (it's the same person on another installation): the existing id is returned with `created = false`.

#### import_target_type
Payload: TargetType with `uuid`.

If the uuid is new but the name exists, the type is created with the name suffix ` (imported)`, or ` (imported 2)`, ` (imported 3)`, … if that exists too.

#### import_device
Payload: Device with `uuid` and `shot_count`; optional `staged_file` (device image).

A created device takes `shot_count` from the file and gets the image, if given.

#### import_session
Payload: Session with `uuid` and `saved_at` from the file, `shooter_id`, `device_id` and `target_type_id` already mapped to local ids; `images` as in `create_session`.

Importing a session never changes `devices.shot_count`.

### Settings

#### save_setting
Creates the setting with the given `name`, or updates it if it exists.

Payload: Setting fields

Validation: the value matches the setting's `value_rule`; `active = false` only for optional settings. For the folder settings `staging_dir` and `export_dir`: the folder exists or can be created and is writable; `staging_dir` is on the same file system as `image_dir`. `image_dir` can't be changed with `save_setting` (`validation_failed`); use `change_image_dir`.

Response `save_setting_response`: `{ "name": <setting name> }`

#### read_settings
Payload: `{ "filter": Filter on Setting fields }`

Response `read_settings_response`: `{ "settings": [Setting, ...] }`

#### delete_setting
Payload: `{ "name": <setting name> }`

Response `delete_setting_response`: `{ "name": <deleted name> }`. Error `not_found` if it does not exist; `validation_failed` for the [known settings](database.md#known-settings), which can't be deleted.

#### reset_settings
Restores settings to their default value and default `active` state.

Payload: `{ "names": [<setting name>, ...] or null, "move_images": bool }`. `names = null` resets all known settings except `admin_password_hash` and `lens_calibrations` (a password and measured calibrations aren't preferences; they can only be reset by name). `move_images` is used if `image_dir` is reset (see `change_image_dir`).

Response `reset_settings_response`: `{ "names": [<reset setting names>], "images": { "moved": integer, "failed": [<image_path>, ...] } or null }`

#### change_image_dir
Changes the image folder, and optionally moves the stored images there.

Payload: `{ "path": <new folder>, "move_images": bool }`

1. Check the new folder: exists or can be created, writable, on the same file system as `staging_dir`.
2. If `move_images`: copy every file of the old folder's `devices/` and `sessions/` into the new folder. This runs in the background, so all other database requests keep working meanwhile.
3. Switch: save the setting; copy files that were added in the meantime and remove copies of files that were deleted in the meantime. From now on, new images go to the new folder.
4. If `move_images`: delete the files in the old folder that were copied. A file that couldn't be copied stays in the old folder and is reported.

Stored `image_path`s don't change, because they're relative to the image folder.

Response `change_image_dir_response`: `{ "path": <new folder>, "images": { "moved": integer, "failed": [<image_path>, ...] } or null }` (`null` without `move_images`). Images that weren't moved are shown as missing until the user moves them by hand.

### Events (`database_handler:events`)

#### data_changed
Sent after every successful write.

Payload: `{ "table": "sessions" | "shooters" | "devices" | "target_types" | "settings", "key": <id, or the name for settings>, "action": "created" | "updated" | "deleted" }`

After every start and every reconnect to Redis, the database handler also sends `{ "table": "all", "key": null, "action": "reload" }`: events of writes processed just before a crash or a lost connection may not have been sent. Listeners then reload everything they keep from the database (settings, cached records, lists in the UI).

## camera_reader
Stream `camera_reader:requests`. Implemented in C++. Design: [programs/camera_reader.md](programs/camera_reader.md).

#### camera_status
Payload: `{}`

Response `camera_status_response`:
```json
{ "state": "connected", "camera_source": "v4l2", "device": "/dev/video0", "camera_key": "usb-0000:00:14.0-2", "width_px": 1920, "height_px": 1080, "fps": 30.0, "frames_published": 123456 }
```
`state` is `connected`, `disconnected` (configured but not reachable; retrying) or `disabled` (setting `camera_source` is `none`). `device` is the V4L2 device the selected camera currently has. `camera_key` identifies the camera for stored per-camera data (control values, lens calibration): the V4L2 `bus_info` for USB cameras (stable when the device number changes), the RTSP URL for network cameras.

#### camera_list_devices
Lists the USB cameras and the formats they support, for the camera settings menu.

Payload: `{}`

Response `camera_list_devices_response`:
```json
{ "devices": [ { "path": "/dev/video0", "name": "HD Pro Webcam C920", "bus_info": "usb-0000:00:14.0-2",
                 "formats": [ { "width_px": 1920, "height_px": 1080, "fps": [30, 15] }, { "width_px": 1280, "height_px": 720, "fps": [60, 30] } ] } ] }
```

#### camera_list_controls
Lists the controls the current camera offers. Network cameras (RTSP) offer none.

Payload: `{}`

Response `camera_list_controls_response`: `{ "controls": [CameraControl, ...] }`. Error `unavailable` if no camera is connected.

#### camera_set_control
Sets a control on the camera and stores the value, so it's applied again after a reconnect or restart.

Payload: `{ "id": <control id>, "value": integer }`

Response `camera_set_control_response`: `{ "controls": [CameraControl, ...] }`. All controls are returned, because changing one can change others (e.g. auto exposure makes exposure inactive). Error `validation_failed` if the value is out of range.

### Events (`camera_reader:events`)

#### camera_state_changed
Payload: `{ "state": "connected" | "disconnected" | "disabled", "message": <string>, "camera_key": <string or null>, "width_px": <integer or null>, "height_px": <integer or null> }`

## image_processor
Stream `image_processor:requests`. Implemented in C++. Design: [programs/image_processor.md](programs/image_processor.md).

The image processor has two modes:
- **Live mode** (requests `live_*`): the camera feed is transformed to show only the target paper; all holes are detected and drawn; while counting is on, new holes are reported as shots and highlighted.
- **Capture mode** (requests `capture_*`, `find_paper`, `prepare_target_photo`, `detect_shots_in_photo`): one image is transformed to show only the target paper, without any drawing. These images are stored (target image of a session) or used to create target types.

### Live mode

#### live_start
Starts live mode for a target type, or restarts it with new values if it's already active. The paper is searched automatically about once per second until it's found; without camera frames, live mode waits for them (`calibration.message` says why it isn't calibrated). Holes already in the target are detected and shown, but never counted.

Payload: `{ "target_type": TargetType, "bullet_diameter_mm": float }`

Response `live_start_response`: CalibrationStatus. `live_paper` starts at 1.

#### live_calibrate
Calibrates live mode from the four paper corners the user clicked in the raw camera feed. Corners are always given in pixels of the raw camera frame as `camera_reader` publishes it (without lens correction); `image_processor` converts them itself if a lens calibration applies. `CalibrationStatus.corners` uses the same coordinates, so the UI can draw them on the raw feed.

Payload: `{ "corners": Corners }`

Response `live_calibrate_response`: CalibrationStatus. Error `invalid_state` if live mode isn't active.

#### live_set_counting
Turns counting on or off. While on, every new hole is sent as `shot_detected` and highlighted. While off, new holes are only shown like holes that were there before.

Payload: `{ "counting": bool }`

Response `live_set_counting_response`: `{}`. Error `unavailable` if not calibrated and `counting` is `true`.

#### live_uncount_hole
Marks a counted hole as not counted (drawn gray, removed from the shot numbering). `session_manager` sends it when a camera shot isn't part of the session (shot deleted, or detected while the session wasn't running), so the live view always shows exactly the session's shots.

Payload: `{ "hole_id": integer }`

Response `live_uncount_hole_response`: `{}`. Unknown ids are ignored.

#### live_new_paper
The user has put up a new target paper. Saves the last good frame of the old paper as transformed image, forgets all holes, and searches the paper again; holes already on the new paper are shown but not counted. Shot numbering in the live view starts again at 1.

Live mode counts its papers in `live_paper` (1 at `live_start`, +1 at every paper change, whether requested or detected automatically).

Payload: `{ "from_live_paper": integer }`, the paper the sender believes is current. If the paper was already changed automatically in the meantime (current `live_paper` is higher), nothing happens again; the response reports the current paper, and the matching `paper_replaced` event carries the image.

Response `live_new_paper_response`: `{ "changed": bool, "live_paper": integer, "previous_paper_staged_file": "target_<uuid>.jpg" or null }` (`null` if not changed, or if the old paper was never calibrated).

#### live_stop
Ends live mode.

Payload: `{}`

Response `live_stop_response`: `{}`

#### live_calibration_status
Payload: `{}`

Response `live_calibration_status_response`: CalibrationStatus (`not_calibrated` with message `live mode not active` outside live mode)

### Lens calibration
Corrects the lens distortion of a camera (straight lines that look bent). Optional; stored per camera in the setting `lens_calibrations` and applied to all camera frames of that camera at the same resolution. Only allowed while live mode isn't active (`invalid_state`). Details: [programs/image_processor.md](programs/image_processor.md#lens-calibration).

| Request | Payload | Response |
|---|---|---|
| `lens_calibration_start` | `{}` | `{ "views": 0 }`; starts collecting views of the printed checkerboard (9 × 6 inner corners) |
| `lens_calibration_capture` | `{}` | `{ "found": bool, "views": integer }`; looks for the checkerboard in the newest frame and keeps the view if found |
| `lens_calibration_finish` | `{}` | `{ "rms_px": float, "views": integer }`; computes and stores the calibration. `validation_failed` with fewer than 10 views |
| `lens_calibration_cancel` | `{}` | `{}`; discards the collected views |
| `lens_calibration_delete` | `{}` | `{}`; removes the stored calibration of the current camera |
| `lens_calibration_status` | `{}` | `{ "camera_key": string or null, "calibrated": bool, "applied": bool, "rms_px": float or null, "width_px": integer or null, "height_px": integer or null, "collecting_views": integer or null }`; `applied` is `false` if the camera now runs at another resolution than the calibration |

### Capture mode

#### capture_target_image
Saves the last good frame of the current paper (calibrated, nothing in front of the paper), transformed to the target paper, without drawing.

Payload: `{}`

Response `capture_target_image_response`: `{ "staged_file": "target_<uuid>.jpg" }`. Error `unavailable` if there is no good frame of the current paper (never calibrated, or live mode not active).

#### capture_raw_image
Saves the newest camera frame, not transformed (used to take a picture for a new target type). Works with and without live mode. If the camera has an applied lens calibration, the frame is corrected.

Payload: `{}`

Response `capture_raw_image_response`: `{ "staged_file": "raw_<uuid>.jpg", "lens_corrected": bool }`. Error `unavailable` if there are no frames.

#### find_paper
Finds the corners of the target paper in a photo.

Payload: `{ "staged_file": <file name> }`

Response `find_paper_response`: `{ "corners": Corners or null, "width_px": integer, "height_px": integer }`. `corners` is `null` if no paper was found; the user then places the corners by hand.

#### prepare_target_photo
Transforms a photo to the target paper and proposes the ring parameters for a new target type.

Payload: `{ "staged_file": <file name>, "corners": Corners, "paper_width_mm": float, "paper_height_mm": float }`

Response `prepare_target_photo_response`:
```json
{ "staged_file": "paper_6f1c….jpg", "px_per_mm": 6.0,
  "proposal": { "center_x_mm": 85.0, "center_y_mm": 85.0,
                "inner_white_radius_mm": 2.5, "black_radius_mm": 29.75, "outer_radius_mm": 77.75 } }
```
Every proposed value can be `null` if it wasn't found.

#### detect_shots_in_photo
Transforms a photo of a shot target to the paper and detects all holes.

Payload: `{ "staged_file": <file name>, "target_type": TargetType, "bullet_diameter_mm": float, "corners": Corners (optional) }`. Without `corners`, the paper is found automatically.

Response `detect_shots_in_photo_response`: `{ "shots": [ { "angle_turns", "radius_mm", "confidence" }, ... ], "staged_file": "photo_<uuid>.jpg", "warnings": [string, ...] }`. Error `processing_failed` if no paper is found (the UI then asks for the corners).

### Events (`image_processor:events`)

#### calibration_changed
Payload: CalibrationStatus

#### paper_replaced
Sent in live mode when the paper was taken away and a paper without the counted holes was put up (detected automatically after the calibration was `lost` and is found again). Does the same as `live_new_paper`.

Payload: `{ "live_paper": integer, "previous_paper_staged_file": "target_<uuid>.jpg" or null }` (`live_paper` is the new paper)

#### shot_detected
Sent while counting is on, once per new hole.

Payload: `{ "hole_id": integer, "live_paper": integer, "angle_turns": float, "radius_mm": float, "confidence": float (0–1), "frame_seq": integer }`

`hole_id` is unique within one live mode (from `live_start` to `live_stop`).

## session_manager
Stream `session_manager:requests`. Design and state machine: [programs/session_manager.md](programs/session_manager.md).

Every successful response has the payload `{ "session": CurrentSession }`, the state after the request. Every request that's not allowed in the current state returns `invalid_state`.

| Request | Payload | Allowed in | Result |
|---|---|---|---|
| `session_get` | `{}` | any | no change |
| `session_setup` | `{ "input_mode": "camera" \| "manual" \| "photo" }` | idle | → setup |
| `session_update_setup` | SessionSetup fields except `input_mode`, all optional | setup | setup updated; input mode `camera`: live mode (re)started |
| `session_calibrate` | `{ "corners": Corners }` | setup, paused (input mode `camera`) | live mode calibrated from clicked corners |
| `session_start` | `{}` | setup (input mode `camera` or `manual`) | → running |
| `session_upload_photo` | `{ "staged_file": <file name>, "corners": Corners (optional) }` | setup, review (input mode `photo`) | photo transformed, shots detected and added as a new paper; from setup → review |
| `session_pause` | `{}` | running | → paused |
| `session_resume` | `{}` | paused | → running |
| `session_add_shot` | Shot, plus optional `paper_no` (default: the current paper; in review: the last paper) | running (input mode `manual`), review | shot added |
| `session_update_shot` | `{ "index": integer, "angle_turns": float, "radius_mm": float }` | running (input mode `manual`), review | shot changed |
| `session_delete_shot` | `{ "index": integer }` | running, paused, review | shot removed |
| `session_new_paper` | `{}` | running, paused (input mode `camera` or `manual`) | next paper; camera: `live_new_paper` |
| `session_set_notes` | `{ "notes": string or null }` | setup, running, paused, review, recovered | notes changed |
| `session_stop` | `{}` | running, paused | → review; camera: the last paper's image is taken, live mode ends |
| `session_save` | `{}` | review, recovered | saved as record, → idle; response also has `"session_id"` and `"warnings"` (e.g. saved without target image) |
| `session_cancel` | `{}` | setup, running, paused, review, recovered | discarded, → idle |

In `session_update_setup`, a field that is present sets the value, also to `null` (e.g. `"shooter_id": null` removes the shooter); a missing field keeps its value.

`index` counts from 0 in the `shots` list. A request that's allowed in the state but whose guard fails answers `validation_failed` if a value is missing or invalid (e.g. `session_start` without device, `session_save` without shots) and `unavailable` if a resource isn't ready (e.g. `session_start` in input mode `camera` before calibration). `session_update_setup` rejects retired shooters, devices and target types (`validation_failed`). Shot changes are only possible before saving. A saved session can't be changed.

### Events (`session_manager:events`)

#### session_changed
Sent after every change, including every new analysis result.

Payload: `{ "session": CurrentSession }`

#### session_saved
Payload: `{ "session_id": <id> }`

## analyzer
Stream `analyzer:requests`. Design and algorithms: [programs/analyzer.md](programs/analyzer.md).

#### analyze_shots
Payload: `{ "shots": [Shot, ...], "target_type": TargetType, "bullet_diameter_mm": float }`

Response `analyze_shots_response`: AnalysisResult

## exporter
Stream `exporter:requests`. Design and file formats: [programs/exporter.md](programs/exporter.md).

#### export_sessions
Payload: `{ "filter": SessionFilter, "format": "csv" | "json" | "png", "include_images": bool }`

- `include_images` (default `false`): only for `csv` and `json`. The data files and the session target images and device images are written into one ZIP file (layout in [programs/exporter.md](programs/exporter.md#zip-with-images)). With `png`: `validation_failed`.

Response `export_sessions_response`: `{ "files": [<file name in export folder>, ...] }` (with `include_images`: exactly one `.zip`). Error `not_found` if no session matches; `setting_not_set` if `export_dir` isn't set.

#### list_exports
Payload: `{}`

Response `list_exports_response`: `{ "files": [{ "name": string, "size_bytes": integer, "created_ms": integer }, ...] }`, newest first. Error `setting_not_set` if `export_dir` isn't set.

#### delete_export
Payload: `{ "name": <file name in export folder> }`

Response `delete_export_response`: `{ "name": <deleted file> }`

## statistics
Stream `statistics:requests`. Design: [programs/statistics.md](programs/statistics.md).

#### compute_statistics
Payload:
```json
{ "filter": SessionFilter,
  "group_by": "none" | "shooter" | "device" | "device_category" | "distance" | "single_handed" | "supported",
  "period": "session" | "day" | "week" | "month" }
```
- `group_by`: splits the result into groups for comparison. Results are always also split by target type, because scores of different target types can't be compared.
- `period`: resolution of the trend (`session`: one point per session).

Response `compute_statistics_response`: `{ "groups": [StatisticsGroup, ...], "session_count": integer }`. Ids in the filter that don't exist simply match no session.

### StatisticsGroup
| Field | Type | Notes |
|---|---|---|
| `target_type_id` | integer | |
| `group_key` | object or null | The value the group stands for, by `group_by`: `shooter` → `{ "shooter_id": 4 }` (`null` id for sessions without shooter), `device` → `{ "device_id": 2 }`, `device_category` → `{ "category": "air pistol" }`, `distance` → `{ "distance_m": 10 }`, `single_handed` / `supported` → `{ "single_handed": true }`; `null` for `group_by = none` |
| `summary` | StatisticsValues | Over all sessions of the group |
| `trend` | list of `{ "period_start": integer, "values": StatisticsValues }` | One entry per period with sessions, sorted by time |
| `shots` | list of Shot | All shots of the group (for the shot distribution plot), at most 5 000; the newest are kept |

### StatisticsValues
| Field | Type | Notes |
|---|---|---|
| `session_count`, `shot_count` | integer | |
| `score_avg` | float | Average session score |
| `score_min`, `score_max` | integer | Lowest and highest session score |
| `score_per_shot_avg` | float | Comparable between sessions with different shot counts |
| `grouping_score_per_shot_avg` | float | |
| `extreme_spread_mm_avg` | float | Largest distance between two shot centers of a session, averaged |
| `mean_radius_mm_avg` | float | Average distance of the shots from their mean point of impact, averaged over sessions |
| `mean_point_avg` | `{ "x_mm": float, "y_mm": float }` | Average offset of the mean point of impact from the ring center (sight adjustment hint) |

## importer
Stream `importer:requests`. Design: [programs/importer.md](programs/importer.md).

#### import_file
Imports a JSON export file, or a ZIP of a JSON export with images (formats in [programs/exporter.md](programs/exporter.md#json)). Images in the ZIP are restored with their sessions and devices.

Payload: `{ "staged_file": <file name, .json or .zip> }`

Response `import_file_response`:
```json
{ "shooters":     { "created": 1, "existing": 0 },
  "target_types": { "created": 1, "existing": 2 },
  "devices":      { "created": 0, "existing": 1 },
  "sessions":     { "created": 14, "existing": 3 },
  "images":       { "restored": 15 },
  "warnings": [] }
```
Error `validation_failed` if the file (or the JSON inside the ZIP) doesn't match the export schema, or the ZIP contains unexpected entries (the message names the first problem); nothing is imported then.

## launcher
Stream `launcher:requests`. Design: [programs/launcher.md](programs/launcher.md).

#### launcher_status
Payload: `{}`

Response `launcher_status_response`:
```json
{ "programs": [ { "name": "analyzer", "state": "running", "pid": 4242, "restarts": 0, "since_ms": 1794393600000 } ] }
```
`state` is `starting`, `running`, `backoff` (waiting to restart), `failed` (gave up restarting) or `stopped`.

#### restart_program
Payload: `{ "name": <program name> }`

Response `restart_program_response`: `{ "name": <program name> }`

### Events (`launcher:events`)

#### program_state_changed
Payload: one entry of `programs` from `launcher_status_response`.
