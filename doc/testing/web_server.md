# Tests: web_server

Tests of the program `web_server`, its frontend and the web UI.

Design: [web_server.md](../programs/web_server.md), [frame_transport.md](../frame_transport.md#live-feed-to-the-browser-mjpeg). Reference data and tools: [overview.md](overview.md).

## Setup
- **API tests** (integration) run `web_server` as a process with `RedisTestServer` and `FakeProgram`s for every program it talks to; an HTTP client and a WebSocket client drive it. The fakes record each request, so the tests check what reaches Redis.
- **Feed tests** use `FrameFeeder` on `targetscope/frames/raw` and `targetscope/frames/live`.
- **Frontend unit tests** run the ES modules with `node --test` (`programs/web_server/static/tests/`).
- **Manual UI checklist** on a desktop (Firefox and Chromium) and a phone (portrait, 360 px wide), against a full system.

## REST API mapping (integration)
What is tested: every endpoint sends the right Redis request and returns its response ([web_server.md](../programs/web_server.md#rest-api)). For every row: HTTP 200 with the response payload as JSON body, unless stated otherwise.

| ID | Endpoint and body | Expected Redis request |
|---|---|---|
| WS-API-01 | `GET /api/session` | `session_get` `{}` |
| WS-API-02 | `POST /api/session/setup` `{ "input_mode": "camera" }` | `session_setup` with the same payload |
| WS-API-03 | `PATCH /api/session/setup` `{ "device_id": 1, "target_type_id": 1 }` | `session_update_setup` with the same payload |
| WS-API-04 | `PUT /api/session/notes` `{ "notes": "x" }` | `session_set_notes` |
| WS-API-05 | `POST /api/session/new-paper` | `session_new_paper` `{}` |
| WS-API-06 | `POST /api/session/calibrate` with Corners | `session_calibrate` `{ corners }` |
| WS-API-07 | `POST /api/session/start`, `/pause`, `/resume`, `/stop`, `/save`, `/cancel` | `session_start`, `session_pause`, `session_resume`, `session_stop`, `session_save`, `session_cancel` |
| WS-API-08 | `POST /api/session/shots` `{ "angle_turns": 0.25, "radius_mm": 12 }` | `session_add_shot` with the same payload |
| WS-API-09 | `PUT /api/session/shots/3` `{ "angle_turns": 0.5, "radius_mm": 20 }` | `session_update_shot` `{ index: 3, … }` |
| WS-API-10 | `DELETE /api/session/shots/3` | `session_delete_shot` `{ index: 3 }` |
| WS-API-11 | `POST /api/session/photo` multipart (JPEG and Corners) | File staged as `upload_<uuid>.jpg`; `session_upload_photo` `{ staged_file, corners }` |
| WS-API-12 | `POST /api/session/photo` JSON `{ "staged_file": "upload_….jpg", "corners": … }` (retry) | `session_upload_photo` with the same payload; no upload |
| WS-API-13 | `POST /api/sessions/query` `{ filter, order_by, limit, offset }` | `read_sessions` with the same values and `include_shots` false |
| WS-API-14 | `GET /api/sessions/7` | `read_sessions` `{ filter: { session_ids: [7] }, include_shots: true }`; 404 `not_found` if empty |
| WS-API-15 | `DELETE /api/sessions/7` | `delete_session` `{ id: 7 }` |
| WS-API-16 | `GET`, `POST /api/shooters`; `PUT`, `DELETE /api/shooters/4` | `read_shooters` (empty filter), `save_shooter` without id, `save_shooter` with id 4, `delete_shooter` `{ id: 4 }` |
| WS-API-17 | The same four for `/api/devices` | `read_devices`, `save_device`, `save_device` with id, `delete_device` |
| WS-API-18 | `PUT /api/devices/3/image` multipart | Upload, then `set_device_image` `{ device_id: 3, staged_file }` |
| WS-API-19 | `DELETE /api/devices/3/image` | `delete_device_image` `{ device_id: 3 }` |
| WS-API-20 | The same four for `/api/target-types` | `read_target_types`, `save_target_type`, `save_target_type` with id, `delete_target_type` |
| WS-API-21 | `POST /api/target-types/photo` multipart | Upload, then `find_paper`; body `{ staged_file, corners, width_px, height_px }` (`corners` null passed on) |
| WS-API-22 | `POST /api/target-types/photo/capture` | `capture_raw_image`, then `find_paper` with its `staged_file`; same body |
| WS-API-23 | `POST /api/target-types/photo/prepare` | `prepare_target_photo` |
| WS-API-24 | `GET /api/camera`, `/api/camera/devices`, `/api/camera/controls` | `camera_status`, `camera_list_devices`, `camera_list_controls` |
| WS-API-25 | `PUT /api/camera/controls/9963776` `{ "value": 5 }` | `camera_set_control` `{ id: 9963776, value: 5 }` |
| WS-API-26 | `GET /api/camera/lens`; `POST …/start`, `/capture`, `/finish`, `/cancel`; `DELETE /api/camera/lens` | `lens_calibration_status`, `lens_calibration_start`, `_capture`, `_finish`, `_cancel`, `lens_calibration_delete` |
| WS-API-27 | `GET /api/calibration` | `live_calibration_status` |
| WS-API-28 | `GET /api/settings` | `read_settings`; body without `admin_password_hash` |
| WS-API-29 | `PUT /api/settings/camera_fps` `{ "value": "15", "active": true }` | `save_setting` with name, value, active and the setting's display name |
| WS-API-30 | `POST /api/settings/reset` `{ "names": ["camera_fps"], "move_images": false }` | `reset_settings` with the same payload |
| WS-API-31 | `PUT /api/settings/image_dir` `{ "path": "/srv/img", "move_images": true }` | `change_image_dir` with the same payload, not `save_setting` |
| WS-API-32 | `POST /api/statistics` `{ filter, group_by, period }` | `compute_statistics` |
| WS-API-33 | `POST /api/analyze` | `analyze_shots` |
| WS-API-34 | `POST /api/exports`; `GET /api/exports`; `DELETE /api/exports/a.csv` | `export_sessions`; `list_exports`; `delete_export` `{ name: "a.csv" }` |
| WS-API-35 | `POST /api/imports` multipart `.zip` | Upload, then `import_file` `{ staged_file }` |
| WS-API-36 | `GET /api/system`; `POST /api/system/programs/analyzer/restart` | `launcher_status`; `restart_program` `{ name: "analyzer" }` |
| WS-API-37 | Request timeouts | The fake for `export_sessions` answers after 100 s | The browser gets the answer (the web server waits the request type's timeout, 300 s, not 5 s) |
| WS-API-38 | No way to change a saved session | `PUT` and `PATCH /api/sessions/7` | 405; no Redis request |

## Idempotency-Key (integration)
What is tested: the header that makes resending safe ([web_server.md](../programs/web_server.md#rest-api)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| WS-IDK-01 | Header given | `POST /api/session/shots` with `Idempotency-Key: <uuid>` | `request_id` of the Redis request = that UUID |
| WS-IDK-02 | No header | Same without the header | A new UUID v4 as `request_id` |
| WS-IDK-03 | Invalid header | `Idempotency-Key: abc` | 400 `validation_failed`; no Redis request |
| WS-IDK-04 | Resend | The same request twice with the same key | Both Redis requests have the same `request_id` |

## Errors (integration)
What is tested: mapping of error codes to HTTP statuses ([web_server.md](../programs/web_server.md#error-mapping)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| WS-ERR-01 | Every Redis error code | A fake answers with each code | `invalid_message` and `validation_failed` 400; `not_found` 404; `in_use`, `invalid_state`, `setting_not_set` 409; `unavailable` 503; `processing_failed` 422; `io_error`, `database_error` 500 |
| WS-ERR-02 | Body | Any error | Body = the error object `{ code, message, details }`, `details` passed on unchanged |
| WS-ERR-03 | Timeout | The fake doesn't answer | 504 `timeout` after the request type's timeout |
| WS-ERR-04 | Redis down | Stop Redis; call any API endpoint | 503 `unavailable` |
| WS-ERR-05 | Web-only codes | Missing login; wrong password; too many attempts; upload too large | 401 `auth_required`; 401 `wrong_password`; 429 `too_many_attempts`; 413 `too_large` |

## Admin password (integration)
What is tested: optional protection of destructive actions ([web_server.md](../programs/web_server.md#admin-password)). Uses `FakeClock`.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| WS-ADM-01 | No password | `GET /api/admin`; every endpoint | `{ protected: false, logged_in: false }`; nothing needs a login |
| WS-ADM-02 | Set a password | `PUT /api/admin/password` `{ current_password: null, new_password: "pw" }` | 200; `save_setting` for `admin_password_hash` with an Argon2id hash (starts with `$argon2id$`), active; `protected` true |
| WS-ADM-03 | Protected without login | With a password, without cookie: `DELETE` session, shooter, device, device image, target type, export; `PUT /api/settings/<name>`; `POST /api/settings/reset`; `PUT /api/settings/image_dir`; lens `start`, `finish`, `DELETE`; program restart | Each 401 `auth_required`; no Redis request |
| WS-ADM-04 | Open without login | With a password, without cookie: session requests, creating and changing shooters, devices and target types, camera controls, lens `capture` and `cancel`, statistics, creating exports, imports, all reads | All pass |
| WS-ADM-05 | Wrong password | `POST /api/admin/login` with a wrong password | 401 `wrong_password`; no cookie |
| WS-ADM-06 | Login | Correct password | 200; cookie `HttpOnly`, `SameSite=Strict`, 256-bit random token; protected endpoints pass with it |
| WS-ADM-07 | Idle timeout | No request for 30 min; then one at 29 min, wait 29 min more | After 30 idle minutes: 401; a request at 29 min keeps the session alive |
| WS-ADM-08 | Web server restart | Log in; restart `web_server` | The old cookie gets 401 |
| WS-ADM-09 | Logout | `POST /api/admin/logout` | The cookie gets 401 |
| WS-ADM-10 | Too many attempts | 5 wrong passwords within a minute, then the right one | 6th attempt 429 `too_many_attempts`; after 60 s the right password works |
| WS-ADM-11 | Change the password | With a wrong current password; with the right one | 401 `wrong_password`; then the new password works and the old one doesn't |
| WS-ADM-12 | Remove the password | `new_password: null` with the current password | Setting inactive; everything open again |
| WS-ADM-13 | Hash never readable | `GET /api/settings` with and without password | Never contains `admin_password_hash` |
| WS-ADM-14 | Hash not writable over settings | `PUT /api/settings/admin_password_hash` | 400 `validation_failed`; no Redis request |

## Uploads (integration)
What is tested: size limits, type checks and staging ([web_server.md](../programs/web_server.md#uploads)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| WS-UPL-01 | Image size limit | Image of 20 MB; of 20 MB + 1 byte | Accepted; 413 `too_large` and nothing staged |
| WS-UPL-02 | Import size limits | JSON of 100 MB + 1 byte; ZIP of 2 GB + 1 byte | 413 `too_large` |
| WS-UPL-03 | Streamed to disk | Upload a 1.9 GB ZIP | The web server's memory grows by less than 100 MB |
| WS-UPL-04 | Wrong type | Text file named `.jpg`; text file named `.zip` | 400 `validation_failed`; nothing staged |
| WS-UPL-05 | Staged safely | Watch the staging folder during an upload | Only `.upload_<uuid>.jpg.part` while writing; then `upload_<uuid>.jpg` |
| WS-UPL-06 | Failed request deletes the file | `set_device_image` answers `validation_failed` | The staged file is deleted |
| WS-UPL-07 | Files kept for retries | `find_paper` fails or finds nothing; `session_upload_photo` answers `processing_failed` | Staged file kept; the error `details` contain `staged_file` |
| WS-UPL-08 | Two files | Multipart with two files | 400 `validation_failed` |

## Files (integration)
What is tested: serving images, staged files and exports ([web_server.md](../programs/web_server.md#files-and-streams)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| WS-FIL-01 | Images | `GET /images/sessions/<file>`, `/images/devices/<file>` | The files from the image folder |
| WS-FIL-02 | Path out of the folder | `/images/sessions/..%2F..%2Fetc%2Fpasswd` and similar | 400 or 404; nothing outside the folder served |
| WS-FIL-03 | Staged files | `upload_*.jpg`, `upload_*.png`, `raw_*`, `paper_*`, `photo_*` | Served |
| WS-FIL-04 | Other staged files | `target_*.jpg`, `import_*.jpg`, `upload_*.zip` | 404 |
| WS-FIL-05 | Exports | `GET /exports/<file>` | `Content-Disposition: attachment` |
| WS-FIL-06 | Frontend | `GET /` | `index.html` |

## MJPEG feeds (integration)
What is tested: `/stream/live.mjpg` and `/stream/raw.mjpg` ([frame_transport.md](../frame_transport.md#live-feed-to-the-browser-mjpeg)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| WS-MJP-01 | Format | `GET /stream/live.mjpg` with live frames at 30 fps | `multipart/x-mixed-replace; boundary=frame`; every part a valid JPEG of the live frame size |
| WS-MJP-02 | Frame rate | `live_feed_fps` 15 | 14–15 parts per second |
| WS-MJP-03 | Quality | `live_feed_quality` 50 instead of 80 | JPEG quality 50 (smaller parts) |
| WS-MJP-04 | Nobody watching | No client connected | No subscriber on the frame services; no encoding |
| WS-MJP-05 | Three viewers | Three clients | Each frame encoded once, not three times |
| WS-MJP-06 | Slow client | A client reads one part per second | Gets the newest frames; memory stays flat; other clients unaffected |
| WS-MJP-07 | Raw feed size | `/stream/raw.mjpg` with 1920 × 1080 raw frames | 1280 × 720 |
| WS-MJP-08 | Fallback | Live mode active; live frames stop for 2 s; then start again | `/stream/live.mjpg` delivers raw frames after 2 s, live frames again when they return |
| WS-MJP-09 | Setting change | `live_feed_fps` 5 while streaming | About 5 parts per second, without reconnecting |
| WS-MJP-10 | Client leaves | Close the last client | Subscriber removed |

## WebSocket (integration)
What is tested: `/ws` ([web_server.md](../programs/web_server.md#websocket-events)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| WS-WSK-01 | Every event stream | Publish one event on each of the 5 event streams | Each delivered as a text message, byte-equal to the event envelope |
| WS-WSK-02 | Nothing lost | 1 000 events in bursts over all streams | Each delivered once, in order per stream |
| WS-WSK-03 | Client too slow | A client that doesn't read while 300 events arrive | It gets `{"type": "resync"}`, then the newer events |
| WS-WSK-04 | Restart | Restart `web_server`; reconnect | Only events published after the restart |
| WS-WSK-05 | Several clients | Three clients | Each gets every event |
| WS-WSK-06 | Reload event | `data_changed` with `table: "all"` | Forwarded like any event |

## Frontend unit tests (unit)
What is tested: the logic of the ES modules ([web_server.md](../programs/web_server.md#frontend)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| WS-JS-01 | Ring generation | Every file in `testdata/target_types/` | Same rings and validation results as the shared libraries ([LIB-TGT-08](shared_libraries.md#ring-generation-and-target-type-validation-unit)) |
| WS-JS-02 | Click to shot | `TT-REF` in the SVG; click at paper (110, 100) | `angle_turns` 0.25, `radius_mm` 10 |
| WS-JS-03 | Click scaling | Raw image 1920 wide shown 640 wide; click at (320, 180) | Raw pixel (960, 540) |
| WS-JS-04 | Filter in the URL | Every SessionFilter field, including an empty list and a missing field; filter → URL → filter | Identical; an empty list stays an empty list |
| WS-JS-05 | Shot labels | One paper; two papers | `3`; `2.3` |
| WS-JS-06 | Resend after a network error | `api.js` request fails with a network error | Sent again with the same `Idempotency-Key` |
| WS-JS-07 | WebSocket reconnect | Close the connection | Reconnects with growing delays; reloads the state afterwards |
| WS-JS-08 | Setting value rules | The valid and invalid values of [DB-SET-02 and DB-SET-03](database_handler.md#settings-integration) | The UI accepts and rejects the same values as `database_handler` |

## Manual UI checklist
What is tested: the web UI as a user sees it, before a release, on a desktop (Firefox and Chromium) and on a phone (portrait). A full system with a test camera ([system tests](system.md#setup)).

| ID | Case | Steps | Expected outcome |
|---|---|---|---|
| WS-UI-01 | New session | Open the session view | Choice of input mode: camera, manual, photo |
| WS-UI-02 | Setup with camera | Choose shooter (also "new shooter" inline), device, target type | Live view appears with the calibration status; retired entries aren't offered |
| WS-UI-03 | Manual calibration | Cover the paper edge so automatic calibration fails; "Calibrate manually"; click 4 corners on the raw image | Calibrated; live view straight |
| WS-UI-04 | Shooting | Start; shots appear | Shots highlighted and numbered; score and grouping score update; "updating…" while the analyzer is slow |
| WS-UI-05 | New paper | Press "New paper" | Numbering per paper (`2.1`, …) |
| WS-UI-06 | Pause and stop | Pause, resume, stop | Running time stops during the pause; review opens after stop |
| WS-UI-07 | Review | Drag a shot, add one by clicking, delete one, write notes | Scores update; labels per paper |
| WS-UI-08 | Save | Save | Note "can't be changed"; the session appears in the history |
| WS-UI-09 | Recovered session | Kill `session_manager` during a session | Dialog "Unsaved session … Save / Discard"; both work |
| WS-UI-10 | Photo session | Upload a photo; one where the paper isn't found | Shots on the photo; the corner editor opens for the second; "Add photo" in review |
| WS-UI-11 | Target type editor | Take a picture; correct the corners; drag center and radii | Rings recalculated while dragging; number fields in sync; saved |
| WS-UI-12 | Locked target type | Open a used target type | Read-only; "Copy and edit" opens a copy |
| WS-UI-13 | Camera settings | Change source, resolution; drag a control slider | Raw image shows the effect; inactive controls greyed out |
| WS-UI-14 | Lens calibration | Download the checkerboard; capture 10 views; finish | Count of views; RMS with a rating; status "applied" |
| WS-UI-15 | Statistics | Set filters, group by shooter, period week; reload the page | Tables and charts per target type; the same view after reloading (filter in the URL) |
| WS-UI-16 | History | Filter, page, open a session, delete it | Images per paper shown; deleted after confirming |
| WS-UI-17 | Shooters and devices | Create, rename, retire | Retired ones hidden in the setup, still in the history |
| WS-UI-18 | Export and import | Export CSV, JSON, PNG, with and without images; download; import a ZIP | Files downloadable; import result with counts and warnings |
| WS-UI-19 | Settings | Enter an invalid value; switch an optional setting off; restore one default and all defaults | Error shown before saving; restore keeps password and lens calibration |
| WS-UI-20 | Image folder | Change the image folder | Popup "Move the existing images…"; result with moved and failed images |
| WS-UI-21 | Unset setting | First export without export folder | Dialog to set it; the export runs afterwards |
| WS-UI-22 | System view | Kill a program | State changes to `backoff`, then `running`; restart button works |
| WS-UI-23 | Admin password | Set a password; delete a session; log out | Password asked for the delete; asked again after logging out |
| WS-UI-24 | Feed fallback | Restart `image_processor` during live view | Feed marked "not transformed", then normal again |
| WS-UI-25 | Phone | Every view on a phone in portrait | Usable at 360 px width; dragging works by touch |
