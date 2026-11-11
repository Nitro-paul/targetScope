# Tests: whole system

End-to-end tests of all programs together, started by the launcher and driven through the REST API and the WebSocket, as the web UI would.

Design: [software_description.md](../software_description.md) and all documents it links. Reference data and tools: [overview.md](overview.md).

## Setup
- Full build of all programs; the launcher with a temporary `HOME` and its default configuration (Redis on a free port).
- Camera: a `v4l2loopback` device fed by `ffmpeg` with videos from `testdata/videos/` (each with ground truth); for RTSP cases `mediamtx`.
- The test client uses the REST API, `/ws` and `/stream/*.mjpg`; it checks files in the data folder directly.
- Unless a case says otherwise: shooter `SH-REF`, device `DEV-REF` and target type `TT-REF` exist (created through the API).
- The suite lives in `tests/system/` and runs nightly and before a release.

## Scenarios (system)
What is tested: the main flows across programs, and the behavior when parts fail.

| ID | Case | Steps | Expected outcome |
|---|---|---|---|
| SYS-01 | First start | Empty `HOME`; start `targetscope` | Within 60 s all 9 programs `running` (`/api/system`); default settings; `GET /` serves the UI; no target types, no sessions |
| SYS-02 | Target type from a photo | `POST /api/target-types/photo` with a photo of `TT-REF`; `prepare` with 200 × 200 mm; save with scores 12 / 7 / 1 | Proposal within 1 mm of `TT-REF`; the saved type has the 12 rings of `TT-REF` |
| SYS-03 | Manual session | Manual mode; start; add the `S-REF` shots; stop; save | WebSocket shows `session_changed` with score 38 after the last shot; history shows the session with score 38, 5 shots, 1 paper, no images; `DEV-REF.shot_count` 5 |
| SYS-04 | Camera session | Camera mode with video `tt_ref_5shots`; start before the first shot; stop after the last; save | Live feed shows the transformed paper; 5 `shot_detected`; the saved session has 5 shots within 1 mm of the truth and one image `images/sessions/<uuid>_1.jpg`; `duration_s` matches the time between start and stop |
| SYS-05 | Paper change | Video with a paper swap after 3 shots, then 2 more | Session with 5 shots, `paper_count` 2, papers [1, 1, 1, 2, 2], 2 images |
| SYS-06 | Photo session | Photo mode; upload 2 photos | `paper_count` 2; shots as in the truth; `duration_s` null; 2 images |
| SYS-07 | `session_manager` crash | `kill -9 session_manager` during a running session | Restarted within 2 s; `GET /api/session` shows `recovered` with the shots; save stores exactly one session; a second save answers `invalid_state` |
| SYS-08 | Redis crash | `kill -9` Redis during a running session | Redis restarted; `session_manager:current` restored from the AOF; the session continues; further shots counted; save works |
| SYS-09 | Power loss | `kill -9` all processes during a running session; start again | Recovered session offered with the shots up to about 1 s before; save works; no stale shared memory; the lock isn't stuck |
| SYS-10 | `database_handler` crash while saving | `db.after_commit` aborts `database_handler` during `session_save` | Restarted; save answered (retried with the same `request_id`); exactly one session; `data_changed` `all`/`reload` seen on `/ws` |
| SYS-11 | Export and import | 3 sessions (one with 2 papers and images, one manual, one photo), a device image; export JSON with images; new installation; import the ZIP; import it again | Records identical (uuids, `saved_at`, shots, notes, retired flags); image bytes identical; second import: everything `existing` |
| SYS-12 | CSV export | Export the sessions of SYS-11 as CSV | Readable by a standard CSV parser; values as in the database |
| SYS-13 | Statistics | Data of SYS-11; statistics grouped by shooter, period month | Values equal a reference calculation from the saved sessions |
| SYS-14 | Image folder change while saving | Change the image folder with moving while a camera session is saved | Both succeed; all images in the new folder and served by `/images/` |
| SYS-15 | Admin password | Set a password; `DELETE` a session without login; log in; delete; `targetscope --reset-admin-password` | 401; then deleted; afterwards everything open again |
| SYS-16 | Settings take effect | `live_feed_fps` 5; `camera_fps` 15; `web_bind_address` to another port | MJPEG at about 5 fps within 2 s; camera reopened at 15 fps; the old address works until `web_server` is restarted, then the new one |
| SYS-17 | Program fails repeatedly | `kill` `analyzer` 11 times within 10 minutes | `/api/system` shows `failed`; restart button brings it back to `running` |
| SYS-18 | Camera gone and back | Stop the loopback feed for 10 s during a running session | Calibration status `no camera frames`; afterwards calibrated again; later shots counted, none from the pause |
| SYS-19 | Lens calibration | Loopback feed with checkerboard views of known distortion; calibrate with 12 views; then a session with shots near the paper edge | RMS below 0.5 px; shot positions within 1 mm |
| SYS-20 | Expired photo | Recovered photo session whose staged photo was deleted; save | Saved without that image; warning in the response |
| SYS-21 | Clean shutdown | `SIGTERM` to the launcher while idle | All stopped within 15 s; exit code 0; AOF written; no shared memory left; next start works |
| SYS-22 | Two browsers | Two WebSocket clients; one controls a session | Both see the same `session_changed` events |
| SYS-23 | Retired records | Retire `DEV-REF` | Not accepted in a new setup (400); still in the history; statistics with `include_retired` false leave its sessions out |
| SYS-24 | Network camera | `camera_source` `rtsp` with a `mediamtx` stream of `tt_ref_5shots` | Same result as SYS-04 |

## Long run (performance)
What is tested: stability over a range day. *(limits set here)*

| ID | Case | Steps | Expected outcome |
|---|---|---|---|
| SYS-PRF-01 | 8 hours live | Camera at 30 fps for 8 h (video in a loop with shots and paper swaps); a session saved every 30 min | No crash, no restart; memory of every program grows by less than 50 MB; Redis memory stays bounded (stream caps); every saved session correct |
| SYS-PRF-02 | Many sessions | 10 000 saved sessions; open the history and statistics through the API | History page within 1 s; statistics within 10 s |
