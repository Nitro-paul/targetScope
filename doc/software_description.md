# TargetScope

TargetScope is an application for Linux PCs that captures, stores, manages and analyzes shooting target data.

## Design principle
Every feature runs as a **separate program**. The programs communicate only through Redis Streams (see [interfaces.md](interfaces.md)); only the camera video is passed through shared memory (see [frame_transport.md](frame_transport.md)). This keeps the parts independent, so that each can be written in the language best suited to its task:
- **Rust** by default,
- **C++** for the camera and image processing (`camera_reader`, `image_processor`), because OpenCV and the Linux camera interface (V4L2) are C/C++ APIs.

All interfaces are language-neutral: messages are JSON defined by JSON Schema files, and the video frame header is a C struct. Any program can be rewritten in another language without changing the others.

## Key features
- **Recording shooting sessions**
  - live with a camera: the camera image is transformed to show only the target paper; all holes are detected; holes that were already there are shown but not counted; new shots are highlighted live,
  - manually by clicking on the target in the web UI,
  - from uploaded photos of the shot target.
  - The target paper can be changed during a session (e.g. one card per shot); every paper's image is stored.
- **Live scoring:** score and grouping score are shown while shooting.
- **Storing shooting sessions as records:** once saved, a session can't be changed (it can be deleted). A session needs at least one shot and can carry notes. A transformed image of every target paper is stored with it.
- **Analyzing shooting sessions:** ring score per shot (edge rule with the bullet diameter), session score, grouping score.
- **Statistics over many sessions** on their own page of the web UI: summaries, comparisons (by shooter, device, distance, …), trends over time and shot distribution, with many filters.
- **Shooters** managed in their own list, so names are consistent across all sessions. Shooters, devices and target types that are no longer used can be retired (hidden in the setup, kept in the history).
- **Matching a weapon (device) to a session,** including a description, a device photo and a lifetime shot count that never decreases.
- **Target types**, always created in the web UI from a photo of the target: paper size, ring center and rings are proposed by image recognition and adjusted with an overlay.
- **Camera settings** in their own web UI menu, with all controls the camera offers and an optional lens calibration with a printed checkerboard.
- **Settings** with defaults that can be restored; optional settings (e.g. the export folder) are asked for when they're first needed.
- **Exporting** sessions as CSV, JSON or PNG target plot; CSV and JSON optionally as a ZIP together with the session and device images. **Importing** JSON exports, including the images from a ZIP.
- Locally hosted **web UI** (English) for managing, viewing, analyzing, recording and the live feed, usable from phones and tablets in the local network. Destructive actions can optionally be protected with an admin password.

## Technology
| Area | Choice |
|---|---|
| Languages | Rust (Cargo workspace) for most programs; C++20 (CMake) for `camera_reader` and `image_processor` |
| Messaging between programs | Redis Streams, JSON messages defined by JSON Schema; Redis persisted with AOF |
| Video between programs | iceoryx2 zero-copy shared memory (Rust and C++ APIs) |
| Storage | SQLite via rusqlite, image files in a folder (see [database.md](database.md)) |
| Image processing | OpenCV 4 or 5 (C++) |
| Camera | V4L2 (USB) or RTSP (network camera), configurable |
| Web | axum, plain HTML/JS/CSS frontend, MJPEG live feed, WebSocket events |
| Process supervision | Own program `launcher`, started by hand |
| Platform | Linux PC |

## Architecture
Each program has exactly one responsibility. Programs don't share memory or access each other's data, except:
- the iceoryx2 video frames,
- files handed over through the [staging folder](interfaces.md#staging-folder) (uploads, captured images, import files), always referenced by name in a Redis message,
- read-only access to the image, export and staging folders by `web_server`, so it can serve those files,
- read-only access to the image folder by `exporter`, so it can put images into ZIP exports.

```
                               +----------+
                               | launcher |  started by hand; starts and supervises Redis and all programs
                               +----------+

  camera ──> camera_reader ══raw frames══> image_processor ══live view══> web_server <──> browser
               (C++)     ║                     (C++)                        │       (HTTP, WebSocket,
                         ╚══════════════ raw frames ═══════════════════════>│        MJPEG)
                 │                               │                          │
  ─────────┬─────┴────────┬───────────┬──────────┴┬──────────┬──────────────┼──
           │              │           │           │          │              │
    session_manager   analyzer   statistics   exporter   importer   database_handler ── SQLite
                                                  │                         │
                                            export folder             image folder
```

═══ iceoryx2 shared memory (video frames), ─── Redis Streams (all messages)

### Programs
| Program | Language | Responsibility | Documentation |
|---|---|---|---|
| `launcher` | Rust | Starts Redis and all programs in order, restarts crashed programs, stops everything cleanly | [programs/launcher.md](programs/launcher.md) |
| `database_handler` | Rust | Only program with access to the database and write access to the image folder; protects records | [programs/database_handler.md](programs/database_handler.md) |
| `camera_reader` | C++ | Reads frames from a USB or network camera and publishes them; camera controls | [programs/camera_reader.md](programs/camera_reader.md) |
| `image_processor` | C++ | Live mode (transformed paper view, hole detection, shot highlighting) and capture mode (transformed paper images for storing, photos, new target types) | [programs/image_processor.md](programs/image_processor.md) |
| `session_manager` | Rust | State machine of the current session: setup, recording, live scoring, review, saving | [programs/session_manager.md](programs/session_manager.md) |
| `analyzer` | Rust | Calculates ring scores, session score and grouping score | [programs/analyzer.md](programs/analyzer.md) |
| `statistics` | Rust | Statistics over many saved sessions for the statistics page | [programs/statistics.md](programs/statistics.md) |
| `exporter` | Rust | Exports sessions as CSV, JSON or PNG target plot, optionally as ZIP with images | [programs/exporter.md](programs/exporter.md) |
| `importer` | Rust | Imports JSON exports (also as ZIP with images), also from other installations | [programs/importer.md](programs/importer.md) |
| `web_server` | Rust | Web UI, REST API, live events over WebSocket, live feeds as MJPEG | [programs/web_server.md](programs/web_server.md) |

Shared code is in one library per language: `targetscope_common` (Rust) and `targetscope_common_cpp` (C++), both tested against the same schemas ([shared_libraries.md](shared_libraries.md)).

### Typical flow: live session
1. The user sets up a session in the web UI with input mode "camera" (shooter, device, distance, target type). `web_server` forwards each action to `session_manager`.
2. With device and target type chosen, `session_manager` starts live mode in `image_processor`. It finds the target paper and shows the transformed paper view; holes already in the target are shown gray. If the paper isn't found, the user clicks its four corners.
3. The user starts the session; counting is switched on.
4. For every new hole, `image_processor` highlights it in the live view and sends `shot_detected`. `session_manager` adds the shot, has it scored by `analyzer`, and the UI shows the shot, the score and the grouping score live.
5. When the user puts up a new paper (or the camera notices the swap), the old paper's image is kept and the shots continue on the new paper.
6. The user stops the session: the image of the last paper is taken and live mode ends. In Review, the user can correct the shots.
7. The user saves. `session_manager` sends `create_session` with all shots and paper images to `database_handler`. From now on the session is a record.

## Code organization
```
targetscope/
├── Cargo.toml, Cargo.lock  # Rust workspace
├── CMakeLists.txt          # C++ projects
├── rustfmt.toml, .clang-format, .editorconfig   # code style (development.md#code-style)
├── schemas/                # language-neutral contract: JSON Schemas, examples, frame_header.h
├── crates/
│   ├── targetscope_common/         # Rust shared library
│   └── targetscope_testkit/        # test tools (Redis test server, fakes, fake clock, frame feeder)
├── cpp/
│   ├── targetscope_common/         # C++ shared library (targetscope_common_cpp)
│   └── targetscope_testkit/        # test tools for the C++ programs
├── programs/
│   ├── launcher/           # Rust
│   ├── database_handler/   # Rust
│   ├── camera_reader/      # C++
│   ├── image_processor/    # C++
│   ├── session_manager/    # Rust
│   ├── analyzer/           # Rust
│   ├── statistics/         # Rust
│   ├── exporter/           # Rust
│   ├── importer/           # Rust
│   └── web_server/         # Rust, includes static/ with the frontend
├── tests/
│   └── system/             # system tests (all programs together)
├── testdata/               # reference data, photos, videos, golden files
├── config/
│   └── launcher.toml
├── tools/                  # ensure_deps.sh, install_deps.sh, env.sh (check and install the dependencies), check_docs.py (links and comments)
├── release/                # release tools: build.sh, bump.sh, install/run scripts of the bundle (branch releases)
├── VERSION                 # version number <release>.<majorfix>.<minorfix> (see releases.md)
└── doc/
```

## File locations
| Path | Content |
|---|---|
| `~/.config/targetscope/launcher.toml` | Launcher and bootstrap configuration |
| `~/.local/share/targetscope/` | Data folder (`data_dir` in `launcher.toml`) |
| `~/.local/share/targetscope/targetscope.db` | SQLite database |
| `~/.local/share/targetscope/redis/` | Redis persistence (AOF) |
| `~/.local/share/targetscope/images/` | Image folder (setting `image_dir`, default `<data_dir>/images`) |
| `~/.local/share/targetscope/staging/` | [Staging folder](interfaces.md#staging-folder) for files on their way between programs (setting `staging_dir`, default `<data_dir>/staging`) |
| `~/TargetScope/exports/` | Export folder (setting `export_dir`) |
| `~/.local/state/targetscope/logs/` | Log files, one per program |

The staging and image folders must be on the same file system, so that moving a file is an atomic rename.

## Future additions
- Backups: not part of this version; the structure is prepared for them (see [database.md](database.md#future-backups)).

## Documentation
- [database.md](database.md): tables, records, relations, SQL schema, known settings
- [interfaces.md](interfaces.md): language-neutral contract, Redis conventions, all messages of all programs
- [frame_transport.md](frame_transport.md): video frames over iceoryx2 and the MJPEG live feeds
- [shared_libraries.md](shared_libraries.md): the Rust and C++ libraries that implement the contract
- [programs/](programs/): one design document per program, same names as the program folders in the code
- [testing/](testing/overview.md): the complete test suite: test levels, tools, reference data and every test case per part
- [development.md](development.md): installing the build, runtime and test dependencies
- [releases.md](releases.md): version numbers, building a release, installing, running and uninstalling it
