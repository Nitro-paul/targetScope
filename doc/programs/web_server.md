# web_server

Program `web_server` (Rust): serves the web UI in the local network. It translates browser actions into Redis requests, forwards live events over WebSocket, and streams the camera feeds as MJPEG. It has no logic or state of its own beyond that.

## Responsibilities
- Serves the frontend (static HTML/JS/CSS).
- REST API: one endpoint per user action, each mapped to one Redis request.
- WebSocket: forwards the events of all programs to the browsers.
- Live feeds: MJPEG of the transformed live view and of the raw camera image ([frame_transport.md](../frame_transport.md#live-feed-to-the-browser-mjpeg)).
- Uploads (photos of shot targets, photos for target types, device images, import files): writes them into the staging folder.
- Serves files from the image folder and the export folder (read-only), and photos in the staging folder that the target type editor shows.

## Technology
| Purpose | Crate |
|---|---|
| HTTP server, WebSocket, multipart | `axum` (features `ws`, `multipart`) |
| Static files | `tower-http` (`ServeDir`, compression, tracing) |
| JPEG encoding | `turbojpeg` |
| Image scaling | `fast_image_resize` |
| Redis, frames, settings | `targetscope_common` |

Frontend: plain HTML, CSS and JavaScript (ES modules), no build step, no framework. Targets are drawn as SVG. All texts are in English.

## Access
Listens on the setting `web_bind_address` (default `0.0.0.0:8080`), so phones and tablets in the local network can be used at the range. The UI shows the URL to open on other devices (`http://<PC IP>:8080`).

Without an admin password, everyone in the network can use and change everything. Destructive actions can optionally be protected (see [Admin password](#admin-password)).

### Admin password
Optional. While the setting `admin_password_hash` is inactive (default), everything is open. When the user sets a password (settings view), these actions need it:
- deleting sessions, shooters, devices, device images, target types and exports,
- changing settings, restoring defaults, changing the image folder, camera source and resolution,
- lens calibration (start, finish, delete),
- restarting programs.

Recording sessions, viewing, statistics, exports and imports stay open.

| Method and path | Purpose |
|---|---|
| `GET /api/admin` | `{ "protected": bool, "logged_in": bool }` |
| `POST /api/admin/login` | body `{ "password" }`; sets the admin cookie |
| `POST /api/admin/logout` | ends the admin session |
| `PUT /api/admin/password` | body `{ "current_password" or null, "new_password" or null }`; sets, changes or (`new_password = null`) removes the password; needs the current password if one is set |

- The password is stored as an Argon2id hash (`argon2` crate) in the setting `admin_password_hash`; the web server never sends it to the browser and filters it from `GET /api/settings`.
- Login creates a random 256-bit token, kept in the web server's memory, sent as cookie (`HttpOnly`, `SameSite=Strict`). It expires after 30 minutes without use and when the web server restarts.
- A protected endpoint without a valid cookie answers 401 with `{ "code": "auth_required" }`; the UI asks for the password and repeats the action.
- At most 5 wrong passwords per minute per client address; then 429.
- Forgotten password: `targetscope --reset-admin-password` on the PC (see [launcher](launcher.md#usage)) removes it.
- The connection is plain HTTP in the local network, so the password protects against accidental or casual changes, not against someone who records the network traffic.

## REST API
All bodies are JSON. Every write endpoint accepts an optional `Idempotency-Key` header (UUID); it's used as `request_id`, so the browser can safely resend a request after a network error. Without it, a new `request_id` is created.

| Method and path | Redis request |
|---|---|
| **Current session** | |
| `GET /api/session` | `session_get` |
| `POST /api/session/setup` | `session_setup` |
| `PATCH /api/session/setup` | `session_update_setup` |
| `PUT /api/session/notes` | `session_set_notes` |
| `POST /api/session/new-paper` | `session_new_paper` |
| `POST /api/session/calibrate` | `session_calibrate` (body: Corners, in raw frame pixels) |
| `POST /api/session/start` · `/pause` · `/resume` · `/stop` · `/save` · `/cancel` | `session_start` … `session_cancel` |
| `POST /api/session/shots` | `session_add_shot` |
| `PUT /api/session/shots/{index}` | `session_update_shot` |
| `DELETE /api/session/shots/{index}` | `session_delete_shot` |
| `POST /api/session/photo` (multipart, optional Corners) | upload, then `session_upload_photo` |
| **Saved sessions** (records: read and delete only) | |
| `GET /api/sessions?shooter_id=…&device_id=…&target_type_id=…&from=…&to=…&limit=…&offset=…` | `read_sessions` (`include_shots = false`; `from`/`to` filter `saved_at`) |
| `GET /api/sessions/{id}` | `read_sessions` (filter `id eq`) |
| `DELETE /api/sessions/{id}` | `delete_session` |
| **Shooters** | |
| `GET /api/shooters` · `POST /api/shooters` · `PUT /api/shooters/{id}` · `DELETE /api/shooters/{id}` | `read_shooters` / `save_shooter` / `delete_shooter` |
| **Devices** | |
| `GET /api/devices` · `POST /api/devices` · `PUT /api/devices/{id}` · `DELETE /api/devices/{id}` | `read_devices` / `save_device` / `delete_device` |
| `PUT /api/devices/{id}/image` (multipart) | upload, then `set_device_image` |
| `DELETE /api/devices/{id}/image` | `delete_device_image` |
| **Target types** | |
| `GET` · `POST /api/target-types`, `PUT` · `DELETE /api/target-types/{id}` | `read_target_types` / `save_target_type` / `delete_target_type` |
| `POST /api/target-types/photo` (multipart) | upload, then `find_paper`; returns `{ staged_file, corners, width_px, height_px }` (`corners` is `null` if no paper was found) |
| `POST /api/target-types/photo/capture` | `capture_raw_image`, then `find_paper`; same response |
| `POST /api/target-types/photo/prepare` | `prepare_target_photo` |
| **Camera** | |
| `GET /api/camera` | `camera_status` |
| `GET /api/camera/devices` | `camera_list_devices` |
| `GET /api/camera/controls` | `camera_list_controls` |
| `PUT /api/camera/controls/{id}` | `camera_set_control` |
| `GET /api/camera/lens` | `lens_calibration_status` |
| `POST /api/camera/lens/start` · `/capture` · `/finish` · `/cancel` | `lens_calibration_start` … `lens_calibration_cancel` |
| `DELETE /api/camera/lens` | `lens_calibration_delete` |
| `GET /api/calibration` | `live_calibration_status` |
| **Settings** | |
| `GET /api/settings` · `PUT /api/settings/{name}` | `read_settings` / `save_setting` |
| `POST /api/settings/reset` (body: names or `null`, `move_images`) | `reset_settings` |
| `PUT /api/settings/image_dir` (body: path, `move_images`) | `change_image_dir` (instead of `save_setting`) |
| **Statistics** | |
| `POST /api/statistics` (body: filter, `group_by`, `period`) | `compute_statistics` |
| **Analysis** | |
| `POST /api/analyze` | `analyze_shots` (used by the history view to show shot scores) |
| **Export and import** | |
| `POST /api/exports` | `export_sessions` (body: filter, format, `include_images`) |
| `GET /api/exports` · `DELETE /api/exports/{name}` | `list_exports` / `delete_export` |
| `POST /api/imports` (multipart, `.json` or `.zip`) | upload, then `import_file` |
| **System** | |
| `GET /api/system` | `launcher_status` |
| `POST /api/system/programs/{name}/restart` | `restart_program` |

Query parameters of `GET /api/sessions` are translated into a Filter; only the listed parameters are accepted. There is no endpoint that changes a saved session.

### Files and streams
| Path | Content |
|---|---|
| `GET /` | Frontend (`static/`) |
| `GET /images/{devices\|sessions}/{file}` | Image folder, read-only |
| `GET /staging/{file}` | Images in the [staging folder](../interfaces.md#staging-folder) (only `upload_*.jpg/png`, `raw_*`, `paper_*`, `photo_*`), for the paper corner editor, the target type editor and the review of photo sessions |
| `GET /exports/{file}` | Export folder, read-only, `Content-Disposition: attachment` |
| `GET /stream/live.mjpg` | Live view: transformed paper with holes and shots |
| `GET /stream/raw.mjpg` | Raw camera image, for manual calibration and the camera settings |
| `GET /ws` | WebSocket with live events |

File paths are checked: plain file names only, no `..`.

### Error mapping
| Redis error code | HTTP status |
|---|---|
| `invalid_message`, `validation_failed` | 400 |
| `not_found` | 404 |
| `in_use`, `invalid_state`, `setting_not_set` | 409 |
| (web only) `auth_required` | 401 |
| `unavailable` | 503 |
| `processing_failed` | 422 |
| `timeout` | 504 |
| `io_error`, `database_error` | 500 |

The response body is always the error object `{ "code": "...", "message": "...", "details": { } }`.

## Uploads
1. Accept `multipart/form-data` with one file: images (JPEG or PNG, checked by magic bytes) up to 20 MB, import files up to 100 MB (JSON) or 2 GB (ZIP, checked by magic bytes). Uploads are streamed to disk, never held in memory.
2. Write it to the staging folder as `upload_<uuid>.<ext>` (temporary name first, then rename).
3. Send the Redis request with that file name. If the request fails, delete the file (except for the target type editor, which keeps working with it).

## WebSocket events
`/ws` sends every event from `session_manager:events`, `image_processor:events`, `camera_reader:events`, `launcher:events` and `database_handler:events` as JSON text messages, unchanged (event envelope). One background task per event stream reads with `XREAD BLOCK ... $` and writes into a `tokio::sync::broadcast` channel; each WebSocket connection subscribes to it. A client that falls behind by more than 256 events is sent `{"type": "resync"}` and reloads the state through the REST API. The browser reconnects automatically with backoff and also reloads the state after reconnecting.

## Frontend
```
programs/web_server/static/
├── index.html          # shell with navigation
├── css/app.css
└── js/
    ├── main.js         # router (hash-based), navigation
    ├── api.js          # fetch wrapper (JSON, Idempotency-Key, error handling)
    ├── events.js       # WebSocket with reconnect, event dispatch
    ├── target.js       # SVG target: paper, black area, rings; click → paper mm → polar
    ├── overlay.js      # draggable handles (corners, center, radii) over an image
    └── views/
        ├── session.js  # current session: setup, live view, live score, shots, review, recovery dialog
        ├── history.js  # saved sessions: list, filter, detail with target plot and target image, delete
        ├── shooters.js # shooters: create, rename, retire, delete
        ├── devices.js  # devices with description, image upload, retire
        ├── targets.js  # target types: list, target type editor, copy of locked types, retire
        ├── camera.js   # camera settings menu
        ├── statistics.js # statistics over many sessions (own page, see below)
        ├── data.js     # export (format, "include images" for CSV/JSON, list, download, delete) and import (.json/.zip)
        ├── settings.js # general settings, restore defaults, image folder popup
        └── system.js   # program status, restart
```

### Session view per state
| State | View |
|---|---|
| idle | "New session" with the choice of input mode: camera, manual, photo |
| setup | Form (shooter from a list, with "new shooter" inline; device, distance, target type, single-handed, supported; retired entries aren't offered). Camera: live view as soon as device and target type are chosen, with calibration status; if the paper isn't found: "Calibrate manually" (click the 4 paper corners on the raw image). Photo: upload field; if the paper isn't found, the corner editor opens on the photo. |
| running / paused | Camera: live view (transformed paper; old holes gray, session shots highlighted and numbered per paper). Manual: SVG target, a click adds a shot. Button "New paper" (camera, manual). Notes field. Always: shot list, **live score and grouping score** (marked "updating…" while `analysis_outdated`), running time, pause/resume/stop |
| review | SVG target with all shots (drag to move, click to add, delete), labelled per paper; input mode `photo`: on top of the transformed photo of the selected paper, and "Add photo" for another paper. Notes field. Shot scores, score, grouping score and shift, all updated live. Shots with confidence < 0.5 are highlighted. Save / Cancel. Note: "After saving, the session can't be changed." |
| recovered | Dialog: "Unsaved session from … with n shots: Save / Discard" |

### Target type editor
Target types can only be created this way, from a picture of the target; there is no form to enter a target type without a picture (imported target types are the only other source):
1. **Picture:** take one with the camera (`capture_raw_image`) or upload one.
2. **Paper:** the photo is shown with the four proposed paper corners (`find_paper`) as draggable handles; if no paper was found, the handles start at the corners of the photo. The user corrects them and enters the paper width and height in mm.
3. **Rings:** `prepare_target_photo` returns the transformed paper image and proposed values. The image is shown with an overlay of the target type:
   - ring center: draggable point,
   - inner white, black and outermost ring: circles with a draggable handle each for the radius,
   - all generated rings in between, recalculated live while dragging,
   - number fields for every value (center x/y, three radii, three scores; defaults 12 for inner white, 1 for outermost), in sync with the overlay.
   Everything can be shifted until the overlay matches the paper.
4. **Name** and save (`save_target_type`).

The picture is only used to fill in the parameters; it isn't stored. A **locked** target type (used by a saved session) opens read-only, with a "Copy and edit" button that opens the editor with a copy (new name required).

### Statistics page
Own page `#/statistics` in the navigation, separate from the session history.
- **Filter panel:** multi-select for shooters, devices, device categories, calibers and target types; ranges for distance, time (with shortcuts: last 7 days, 30 days, year) and shot count; three-way switches (any / yes / no) for single-handed and supported. The filter is kept in the page URL, so a view can be bookmarked and reopened.
- **Group by:** none, shooter, device, device category, distance, single-handed, supported. **Period:** session, day, week, month.
- **Per target type** (results are never mixed across target types):
  - summary table per group (sessions, shots, score avg/min/max, score per shot, grouping score per shot, extreme spread, mean radius, mean point offset),
  - trend chart over time per group (score per shot and grouping score per shot; extreme spread on a second axis),
  - shot distribution: all shots of the selected group on the target, as dots or as a density map.
- Charts are drawn as SVG by the page itself (no chart library), like the targets.

### Settings view
- All settings with their display name and value. Optional settings have an "active" switch; when it's off, the value field shows the stored suggestion greyed out.
- **Restore default:** per setting, and "Restore all defaults" for all settings (with a confirmation). Both use `reset_settings`.
- **Image folder:** changing `image_dir` (or restoring its default) opens a popup: "Move the existing images to the new folder?" with *Move images* / *Don't move* / *Cancel*. The answer is sent as `move_images`. Afterwards the UI shows how many images were moved and lists those that couldn't be moved.
- **Unset settings:** when a request answers `setting_not_set` (e.g. the first export without `export_dir`), the UI opens a dialog to set that setting (prefilled with the suggestion), saves it as active and repeats the request.

### Camera settings menu
- Source: USB camera (list from `camera_list_devices`), network camera (RTSP URL), or none.
- Resolution and frame rate: choices from the formats of the selected camera.
- Controls: built from `camera_list_controls`. Integer controls as sliders with min/max/step and a "default" button, booleans as switches, menus as drop-downs; inactive controls are greyed out. Every change is sent immediately (`camera_set_control`, rate-limited to 5 per second while dragging) and the raw image next to the controls shows the effect. Network cameras show "No controls available".
- Changes of source, resolution and frame rate are saved as settings; the camera reopens.
- **Lens calibration** (USB cameras): shows the status (calibrated, RMS error, applies to the current resolution). "Calibrate lens" offers the checkerboard to print (`/static/calibration/checkerboard_9x6.pdf`), then shows the raw feed with a "Capture view" button and the number of views found (at least 10; hint to cover the image corners and use different angles). "Finish" shows the RMS error with a rating. "Delete calibration" removes it. Not possible while a camera session is in setup or running.

### Coordinate conversion in the UI
- SVG target: `viewBox` in paper millimeters (`0 0 paper_width_mm paper_height_mm`), so a click converts directly to paper mm, then to ring-center coordinates (`x = px - center_x_mm`, `y = center_y_mm - py`) and to polar with the formulas in [interfaces.md](../interfaces.md#shot).
- Manual calibration and the paper corner editor: click positions on the displayed image are scaled to the image's real pixels (`real_width / displayed_width`).

## Error handling
| Situation | Behavior |
|---|---|
| A program doesn't answer | 504 to the browser; the UI shows the message and the program status from `/api/system` |
| Redis down | API returns 503; WebSocket clients reconnect until Redis is back |
| No frames | MJPEG connection stays open and shows nothing new; the UI shows the camera state |
| Saving a locked target type or device bullet diameter | 409; the UI offers "Copy and edit" |

## Testing
- API tests with a fake bus (request → expected Redis message, error code → HTTP status).
- Upload tests: size limit, wrong type, cleanup on failure.
- Target type editor: ring generation in JavaScript gives the same rings as the shared libraries for the examples in `schemas/examples/target_types/`.
- Manual UI test checklist per view, on desktop and phone.
