# TargetScope

TargetScope captures, stores, manages and analyzes shooting target data on a Linux PC. Sessions are recorded live with a camera, by clicking in the web UI, or from photos; they are scored, stored as records and evaluated in statistics.

Every feature runs as a separate program. The programs talk only through Redis Streams; camera video goes through iceoryx2 shared memory. Most programs are written in Rust, the camera and image processing in C++.

The full design is in [doc/software_description.md](doc/software_description.md).

## Repository layout
| Path | Content |
|---|---|
| `Cargo.toml` | Rust workspace |
| `CMakeLists.txt` | C++ projects |
| `schemas/` | language-neutral contract: JSON Schemas, examples, `frame_header.h` |
| `crates/targetscope_common/` | Rust shared library |
| `crates/targetscope_testkit/` | Rust test tools |
| `cpp/targetscope_common/` | C++ shared library (`targetscope_common_cpp`) |
| `cpp/targetscope_testkit/` | C++ test tools |
| `programs/<name>/` | one folder per program |
| `tests/system/` | system tests (all programs together) |
| `testdata/` | reference data, photos, videos, golden files |
| `config/launcher.toml` | default launcher configuration |
| `doc/` | design and test documentation |

## Programs
| Program | Language | Design |
|---|---|---|
| `launcher` | Rust | [doc/programs/launcher.md](doc/programs/launcher.md) |
| `database_handler` | Rust | [doc/programs/database_handler.md](doc/programs/database_handler.md) |
| `camera_reader` | C++ | [doc/programs/camera_reader.md](doc/programs/camera_reader.md) |
| `image_processor` | C++ | [doc/programs/image_processor.md](doc/programs/image_processor.md) |
| `session_manager` | Rust | [doc/programs/session_manager.md](doc/programs/session_manager.md) |
| `analyzer` | Rust | [doc/programs/analyzer.md](doc/programs/analyzer.md) |
| `statistics` | Rust | [doc/programs/statistics.md](doc/programs/statistics.md) |
| `exporter` | Rust | [doc/programs/exporter.md](doc/programs/exporter.md) |
| `importer` | Rust | [doc/programs/importer.md](doc/programs/importer.md) |
| `web_server` | Rust | [doc/programs/web_server.md](doc/programs/web_server.md) |

## Building
Requirements and how to install them: [doc/development.md](doc/development.md). Building, installing and running a release: [doc/releases.md](doc/releases.md).

```sh
cargo build --workspace            # Rust programs and libraries
cmake -S . -B build                # C++ library and programs
cmake --build build
```

## Running
TargetScope is started by hand through the launcher, installed as `targetscope`:
```sh
targetscope                        # uses ~/.config/targetscope/launcher.toml
targetscope --config <file>
targetscope --reset-admin-password
```
The web UI is then served by `web_server` in the local network.

## Tests
| Command | Runs |
|---|---|
| `cargo test --workspace` | Rust unit, contract and integration tests |
| `ctest --test-dir build -LE "performance\|accuracy"` | C++ unit, contract and integration tests |
| `node --test programs/web_server/static/tests/` | frontend unit tests |
| `cargo test -p system_tests -- --include-ignored` | system tests |

All test cases are documented in [doc/testing/overview.md](doc/testing/overview.md).

## Branches
Each part is developed on its own branch and merged into `main`:

| Branch | Part |
|---|---|
| `workspace` | root build files (`Cargo.toml`, `CMakeLists.txt`, `.gitignore`) |
| `documentation` | `README.md`, `doc/` |
| `schemas` | `schemas/` |
| `common_rust` | `crates/targetscope_common/` |
| `common_cpp` | `cpp/targetscope_common/` |
| `testkit` | `crates/targetscope_testkit/`, `cpp/targetscope_testkit/`, `testdata/` |
| `<program>` | `programs/<program>/` (`launcher` also has `config/`) |
| `system_tests` | `tests/system/` |

A branch builds once the branches it depends on (`workspace`, `schemas`, the shared libraries and `testkit`) are merged.
