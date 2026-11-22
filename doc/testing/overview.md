# Testing

This folder documents the complete test suite of TargetScope: every test case, what it checks, how it's run and what the expected outcome is. There is one document per part, matching the design documents.

| Test document | Part | Design document |
|---|---|---|
| [shared_libraries.md](shared_libraries.md) | `targetscope_common` (Rust), `targetscope_common_cpp` (C++), message contract | [shared_libraries.md](../shared_libraries.md), [interfaces.md](../interfaces.md) |
| [frame_transport.md](frame_transport.md) | iceoryx2 frame services and frame header, across languages | [frame_transport.md](../frame_transport.md) |
| [database_handler.md](database_handler.md) | `database_handler`, database schema | [database_handler.md](../programs/database_handler.md), [database.md](../database.md) |
| [camera_reader.md](camera_reader.md) | `camera_reader` | [camera_reader.md](../programs/camera_reader.md) |
| [image_processor.md](image_processor.md) | `image_processor`, detection accuracy | [image_processor.md](../programs/image_processor.md) |
| [session_manager.md](session_manager.md) | `session_manager` | [session_manager.md](../programs/session_manager.md) |
| [analyzer.md](analyzer.md) | `analyzer` | [analyzer.md](../programs/analyzer.md) |
| [statistics.md](statistics.md) | `statistics` | [statistics.md](../programs/statistics.md) |
| [exporter.md](exporter.md) | `exporter` | [exporter.md](../programs/exporter.md) |
| [importer.md](importer.md) | `importer` | [importer.md](../programs/importer.md) |
| [web_server.md](web_server.md) | `web_server`, frontend, manual UI checklist | [web_server.md](../programs/web_server.md) |
| [launcher.md](launcher.md) | `launcher` | [launcher.md](../programs/launcher.md) |
| [system.md](system.md) | all programs together, started by the launcher | [software_description.md](../software_description.md) |

## Test levels
| Level | What is tested | Environment | When it runs |
|---|---|---|---|
| Unit | one function or module, without I/O | `cargo test`, GoogleTest, `node --test` | every commit |
| Contract | messages and the frame header against `schemas/`, in every language | contract test suites of the shared libraries | every commit |
| Integration | one program as a real process: its Redis streams, files and database; the other programs are replaced by fakes | [test kit](#test-kit), Redis test server | every commit |
| System | all programs started by the launcher, driven through the REST API and the WebSocket | full build, test camera sources | nightly and before a release |
| Performance | the time limits of the design | release build on the development PC | nightly and before a release |
| Hardware | real cameras, real targets, real light | manual, with checklists | before a release |
| Manual UI | the web UI on a desktop and a phone | manual checklist | before a release |

The level of each group of test cases is given in its heading.

## Conventions
- **Test case IDs:** `<part>-<area>-<nn>`, e.g. `DB-SES-03`. Parts: `LIB`, `FT`, `DB`, `CR`, `IP`, `SM`, `AN`, `ST`, `EX`, `IM`, `WS`, `LA`, `SYS`. An ID never changes and is never reused; a removed case leaves a gap.
- **Columns:** *Case* says what exactly is checked, *Steps / input* how it's done, *Expected outcome* what must be observed. Every group of cases starts with *What is tested*.
- **Independent tests:** every test starts from a fresh state: its own Redis test server (or a flushed one), new temporary folders, a new database.
- **Errors change nothing:** for every case that expects an error, the test also checks that nothing else happened: no new or changed database rows, no moved or deleted files, no events, no requests to other programs, unless the case says otherwise.
- **Tolerances**, unless a case says otherwise: geometry and scoring 1e-9; grouping shift ±0.05 mm (the fine grid step); image processing positions < 1 mm; times ±10 % or ±50 ms, whichever is larger.
- **Names and units** in steps and outcomes are those of [interfaces.md](../interfaces.md).
- **Goals set here:** where the design only says "fast" or "a few seconds", the test documents set a concrete limit and mark it *(goal set here)*.

## Test environment

### Test kit
`crates/targetscope_testkit` (Rust) and `cpp/targetscope_testkit` (C++, the parts the C++ programs need) contain the tools all tests use:

| Tool | Purpose |
|---|---|
| `RedisTestServer` | starts `redis-server` on a free port with its own temporary folder and AOF on; can be stopped and started again on the same folder (Redis restart tests). It checks that the server answering on the port is the process it started (`INFO server`, `process_id`); if another test's server took the port first, it tries another free port |
| `TestBus` | a `Bus` for the test itself: sends requests to the program under test, with a chosen `request_id` if needed |
| `FakeProgram` | serves `<program>:requests` with scripted answers per request type (response, error code, delay, no answer, or an answer computed from the request payload, e.g. a page of `read_sessions`) and records every request it gets |
| `ProgramUnderTest`, `program_binary` | runs a program as a process with the `TARGETSCOPE_*` variables of a `TempDataDir` and waits until it answers `ping`. `program_binary(name)` builds another program of the workspace once per test process (`cargo build -p <name>` with the same target folder and profile) and returns its path, for tests that need a real `database_handler` or `analyzer` |
| `ProgramProcess` (C++) | runs a program binary as a process with a given environment, sends `SIGTERM` and waits for the exit |
| `EventProbe` | reads a `<program>:events` stream and waits for expected events, with a timeout; can also check that no event came |
| `TempDataDir` | temporary data, image, staging and export folders; optionally a small `tmpfs` ("disk full") and a second mount ("other file system") |
| `FakeClock` | controllable time; all time-dependent code reads time through the shared `Clock` ([shared_libraries.md](../shared_libraries.md#clock-and-failpoints)) |
| `FrameFeeder` | publishes frames on `targetscope/frames/raw` or `targetscope/frames/live` from images, videos or the synthetic renderer, at a chosen frame rate and resolution |
| `Failpoints` | switches named failpoints on (below) |
| `Fixtures` | loads the reference data (below) into a database, the image folder or the staging folder |
| `TestProgram` (launcher tests) | a small binary that answers `ping` after a chosen delay, exits with a chosen code, ignores `SIGTERM`, or prints lines, as told by its arguments |

### Failpoints
Test builds contain named failpoints (Rust: `fail` crate; C++: `TS_FAILPOINT` macro). Release builds don't contain them. A failpoint can return an error, wait, or abort the process.

| Failpoint | Where |
|---|---|
| `bus.before_response` | shared bus: after the handler ran, before the response is sent |
| `bus.before_ack` | shared bus: after the response was sent, before `XACK` |
| `settings.after_note_id` | settings cache: after noting the newest event id, before reading the settings |
| `db.before_commit` | database_handler: inside a write transaction, before `COMMIT` |
| `db.after_commit` | database_handler: after `COMMIT`, before the response is sent |
| `db.image_dir_copy` | database_handler: between two files of the background copy of an image folder change |
| `db.migrate` | database_handler: before each schema migration (only to wait, e.g. `sleep(3000)`) |
| `sm.before_exec` | session_manager: before `EXEC` of a state change |
| `sm.after_create_session_sent` | session_manager: after `create_session` was sent, before its response is handled |
| `exporter.before_rename` | exporter: after a `.part` file is written, before the rename |
| `importer.after_record` | importer: after each imported record |

### Camera sources
| Source | Used for |
|---|---|
| `FakeSource` | camera_reader unit tests: generated frames, scripted read failures and stalls |
| `v4l2loopback` device fed by `ffmpeg` | V4L2 capture path with recorded and synthetic videos (integration and system tests) |
| `vivid` kernel test driver | V4L2 controls of every kind |
| `mediamtx` streaming a video file | RTSP path |
| real USB webcams, a network camera | hardware tests |

## Reference data
Test data is in `testdata/` at the root of the repository:
```
testdata/
├── target_types/   # target types as JSON, with their expected rings and validity (also checked against the target_type schema, LIB-CON-02)
├── sessions/       # session sets for database, exporter and statistics tests, with expected results
├── images/         # small files for staging tests: paper.jpg, device.png, not_an_image.jpg, large_20mb.jpg
├── photos/         # photos of shot targets, with ground truth
├── videos/         # recorded camera videos, with ground truth
├── synthetic/      # renderer for targets with known holes, perspective, light and lens distortion
├── lens/           # checkerboard views with known distortion
├── imports/        # valid and invalid import files (incl. ZIP slip, ZIP bomb)
└── exports/        # golden export files
```

### Reference target type `TT-REF`
Round numbers, so expected scores can be calculated by hand.

| Field | Value |
|---|---|
| `name` | `TT-REF` |
| `paper_width_mm`, `paper_height_mm` | 200, 200 |
| `center_x_mm`, `center_y_mm` | 100, 100 |
| inner white | radius 5, score 12 |
| black | radius 30, score 7 |
| outer | radius 90, score 1 |

Generated rings (score: radius in mm): 12: 5 · 11: 10 · 10: 15 · 9: 20 · 8: 25 · 7: 30 · 6: 40 · 5: 50 · 4: 60 · 3: 70 · 2: 80 · 1: 90.

### Other reference records
| Record | Values |
|---|---|
| `SH-REF` | shooter `Alex` |
| `DEV-REF` | device `Test pistol`, category `air pistol`, caliber `.177`, `bullet_diameter_mm` 4.0 (bullet radius b = 2.0) |
| `S-REF` | five shots on `TT-REF` with `DEV-REF`, table below |

| Shot | `angle_turns` | `radius_mm` | x, y in mm | Score (edge rule `d − b ≤ r`) |
|---|---|---|---|---|
| A | 0 | 0 | 0, 0 | 12 |
| B | 0.25 | 12 | 12, 0 | 11 (10 ≤ 10, on the line) |
| C | 0.5 | 22 | 0, −22 | 9 (20 ≤ 20, on the line) |
| D | 0.75 | 42 | −42, 0 | 6 (40 ≤ 40, on the line) |
| E | 0.125 | 95 | 67.175, 67.175 | 0 (93 > 90, miss) |

`S-REF` has the score 38 and the mean point of impact (7.435, 9.035).

### Ground truth for photos, videos and renders
Every photo, video and synthetic render has a `<name>.truth.json`:
```json
{ "target_type": "TT-REF", "bullet_diameter_mm": 4.0,
  "corners": { "top_left": {"x": 412.0, "y": 96.5}, "top_right": {"x": 1508.0, "y": 101.0},
               "bottom_right": {"x": 1496.5, "y": 1003.0}, "bottom_left": {"x": 420.0, "y": 998.0} },
  "holes":  [ { "x_mm": 112.0, "y_mm": 100.0, "before_start": true } ],
  "events": [ { "t_s": 3.2, "type": "shot", "x_mm": 104.0, "y_mm": 92.5 },
              { "t_s": 10.0, "type": "paper_swap" } ] }
```
Positions are in paper millimeters (origin top left). Photos have `holes`; videos have `holes` (present at the start) and `events`.

Accuracy measures for image processing:
- **matched:** a detected hole within one bullet radius of a true hole; each true hole is matched at most once,
- **hit rate** = matched / true holes,
- **false positives** = detected holes without a match,
- **position error** = distance between a matched detection and its true hole, in mm.

## Running the tests
| Command | Runs |
|---|---|
| `cargo test --workspace` | Rust unit, contract and integration tests |
| `ctest --test-dir build -LE "performance\|accuracy"` | C++ unit, contract and integration tests |
| `node --test programs/web_server/static/tests/` | frontend unit tests |
| `cargo test -p system_tests -- --include-ignored` | system tests (`tests/system/`); needs the built programs, `v4l2loopback`, `ffmpeg` and `mediamtx` |
| `cargo test --release --workspace --exclude system_tests -- --ignored perf_` and `ctest --test-dir build -L performance` | performance tests (release build) |
| `cargo test --release -p system_tests -- --ignored perf_` | system performance tests (long run); need what the system tests need |
| `ctest --test-dir build -L accuracy` | image processing accuracy report over `testdata/photos` and `testdata/videos` |

Integration tests need `redis-server` in `PATH`. They start their own Redis on a free port and never touch a running TargetScope.

### When tests run
No CI service is configured in the repository yet; until there is one, these runs are started by hand.

- **Every commit (CI):** unit, contract, integration and frontend unit tests.
- **Nightly (CI):** system tests, performance tests, accuracy report.
- **Before a release:** everything above, plus the hardware tests ([camera_reader](camera_reader.md#hardware-tests-manual), [image_processor](image_processor.md#hardware-tests-manual)), the accuracy acceptance ([image_processor](image_processor.md#accuracy-acceptance-test-set)) and the manual UI checklist ([web_server](web_server.md#manual-ui-checklist)).

A release needs all automated tests passing, every checklist item done, the accuracy goals met and the performance limits met.
