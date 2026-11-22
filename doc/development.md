# Development setup

How to install everything needed to build, run and test TargetScope on a Linux PC. The commands are for Arch Linux and Manjaro; other distributions have packages with similar names.

## What is needed
| Purpose | Dependency | Version |
|---|---|---|
| Rust programs | Rust toolchain (`cargo`, `rustc`) | stable, ≥ 1.89 |
| C++ programs | C++20 compiler, CMake, Ninja | GCC ≥ 13 or Clang ≥ 16, CMake ≥ 3.24 |
| Messaging | Redis-compatible server (`redis-server`): Redis or Valkey | Redis ≥ 7 or Valkey ≥ 8 |
| C++ Redis client | hiredis, redis-plus-plus | hiredis ≥ 1.2, redis-plus-plus ≥ 1.3.9 |
| Video between programs | iceoryx2 C and C++ bindings | exactly the version of the Rust crate (`0.10.0`, see `Cargo.toml`), because both sides share the shared-memory format |
| Image processing, camera | OpenCV with `core`, `imgproc`, `imgcodecs`, `videoio` (V4L2 and FFmpeg backends) and the camera calibration modules: `calib3d` in OpenCV 4, `calib`, `geometry` and `objdetect` (the checkerboard detector) in OpenCV 5 (the header `opencv2/calib3d.hpp` exists in both). CMake accepts 4.x and 5.x and stops with a message for other versions | 4.x or 5.x |
| C++ libraries | nlohmann/json, spdlog, GoogleTest | json ≥ 3.11, spdlog ≥ 1.12 |
| MJPEG feeds | libjpeg-turbo (`libturbojpeg`, found with `pkg-config`) | ≥ 2.1 |
| Frontend tests | Node.js | ≥ 20 |
| System tests | FFmpeg, v4l2loopback, mediamtx | see [testing/overview.md](testing/overview.md#camera-sources) |

SQLite is built into `database_handler` (`rusqlite` with the `bundled` feature); no system package is needed.

## Checking and installing the dependencies
The build scripts do this themselves, on any PC, so nothing has to be installed or exported by hand first:
```
tools/ensure_deps.sh [--no-cpp] [--run]
```
`release/build.sh` calls it (with `--no-cpp` when no C++ program is built) and `release/build_and_run.sh` calls it with `--run`. It sources `tools/env.sh` (below), then:
1. Checks the tools and libraries that only the system can provide: `cargo` and `rustc`, a C compiler, `pkg-config` with libjpeg-turbo; for the C++ programs also `git`, `make`, a C++ compiler, Ninja, CMake ≥ 3.24, OpenCV 4 or 5 (found by CMake), nlohmann/json and spdlog. If something is missing it stops and prints what is missing and the install command for Arch, Debian/Ubuntu or Fedora.
2. Builds into `TARGETSCOPE_DEPS` what needs no root and has no package: redis-plus-plus and iceoryx2 (with hiredis) when CMake doesn't find them, and with `--run` Valkey when there is no `redis-server`. This is `tools/install_deps.sh [valkey] [cpp]` and takes a few minutes the first time; it needs `git`, network access and `cargo`.

## Install with the package manager (needs root)
```sh
sudo pacman -S --needed base-devel git rust cmake ninja nodejs \
    valkey hiredis opencv nlohmann-json spdlog gtest libjpeg-turbo ffmpeg
yay -S redis-plus-plus mediamtx                  # AUR
sudo pacman -S --needed v4l2loopback-dkms        # system tests only
```
Valkey's package also provides `redis-server`. iceoryx2's C and C++ bindings have no package; build them as in [iceoryx2 from source](#iceoryx2-from-source) with `PREFIX=/usr/local` (then `cmake --install` needs `sudo`).

## Install without root
Everything that has no package, or when root isn't available, is built from source into one prefix in the home folder. `tools/install_deps.sh` (repository root) runs exactly these steps:
```sh
PREFIX=$HOME/.local/opt/targetscope-deps
SRC=$HOME/.local/src/targetscope-deps
mkdir -p "$PREFIX" "$SRC" && cd "$SRC"

# Redis-compatible server: Valkey (installs valkey-server and redis-server)
git clone --depth 1 -b 9.1.3 https://github.com/valkey-io/valkey.git
make -C valkey -j"$(nproc)" BUILD_TLS=no
make -C valkey PREFIX="$PREFIX" install

# hiredis and redis-plus-plus
git clone --depth 1 -b v1.3.1 https://github.com/redis/hiredis.git
cmake -S hiredis -B hiredis/build -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX="$PREFIX" -DDISABLE_TESTS=ON \
      -DCMAKE_POLICY_VERSION_MINIMUM=3.5
cmake --build hiredis/build && cmake --install hiredis/build
git clone --depth 1 -b 1.3.9 https://github.com/sewenew/redis-plus-plus.git
cmake -S redis-plus-plus -B redis-plus-plus/build -G Ninja -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX="$PREFIX" -DCMAKE_PREFIX_PATH="$PREFIX" \
      -DREDIS_PLUS_PLUS_BUILD_TEST=OFF -DREDIS_PLUS_PLUS_CXX_STANDARD=20 \
      -DCMAKE_POLICY_VERSION_MINIMUM=3.5 -DCMAKE_CXX_FLAGS=-Wno-maybe-uninitialized
cmake --build redis-plus-plus/build && cmake --install redis-plus-plus/build
```
redis-plus-plus 1.3.9 needs `CMAKE_POLICY_VERSION_MINIMUM` with CMake ≥ 4, and builds with `-Werror`; GCC ≥ 15 reports a false `maybe-uninitialized` warning in its headers, which the extra flag turns off.

### iceoryx2 from source
```sh
git clone --depth 1 -b v0.10.0 https://github.com/eclipse-iceoryx/iceoryx2.git
cmake -S iceoryx2 -B iceoryx2/build -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX="$PREFIX" \
      -DBUILD_CXX=ON -DBUILD_EXAMPLES=OFF -DBUILD_TESTING=OFF
cmake --build iceoryx2/build && cmake --install iceoryx2/build
```
The build calls `cargo` to compile the Rust core of iceoryx2.

## Environment
`tools/env.sh` is sourced by the build scripts and does this: `TARGETSCOPE_DEPS` defaults to `~/.local/opt/targetscope-deps`; when that folder exists, its `bin` is put in front of `PATH` (`redis-server`), and the folder is added to `CMAKE_PREFIX_PATH`, its `lib` to `LD_LIBRARY_PATH` and `lib/pkgconfig` to `PKG_CONFIG_PATH`. For your own shell (running tests by hand, e.g. in `~/.profile`):
```sh
. <repository>/tools/env.sh
```
The top-level `CMakeLists.txt` also finds that folder without the script (and uses it as run path of the C++ binaries): `$TARGETSCOPE_DEPS`, or `~/.local/opt/targetscope-deps` if it exists. `release/package/run.sh` does the same for the `PATH`. If redis-plus-plus or the iceoryx2 bindings aren't found anywhere, CMake stops with a message that says so.

## Build and test
See [README.md](../README.md#building) and [testing/overview.md](testing/overview.md#running-the-tests). Integration tests start their own `redis-server` from `PATH` on a free port; they never touch a running TargetScope.

## Code style
- **Rust:** `rustfmt.toml` (maximum line width 140, `use_small_heuristics = "Max"`). Format with `cargo fmt --all`; `cargo clippy --workspace --all-targets` reports no warnings. A warning that can't be fixed is silenced at the place it occurs with `#[allow(...)]`, preceded by the comment that names the documentation section of the design behind it.
- **C++:** `.clang-format` (Google style, 4 spaces, maximum line width 120, no namespace end comments). Format with `clang-format -i <files>`.
- **Other files:** `.editorconfig` (UTF-8, LF, final newline, no trailing spaces; 4 spaces, 2 for web and data files).
- **Comments:** a comment in code only names the documentation section that explains the code (`// doc/interfaces.md#shot`, several sections separated by `, `) or, in a test, the id of the test case (`// DB-SES-01`). Behavior that isn't documented yet is documented first. Design, reasons and usage belong in `doc/`, not in the code.
- **Unwritten tests:** a planned test case that isn't implemented is a test function marked `#[ignore = "not implemented"]` with `todo!("<id>")` (Rust) or `GTEST_SKIP() << "not implemented: <id>"` (C++), so it is reported as ignored or skipped, never as passed.
- **Build output** (`target/`, `build/`, `build-*/`, `dist/`) is never committed.

## Checks
```
cargo fmt --all --check
cargo clippy --workspace --all-targets
tools/check_docs.py              # Markdown links and anchors, doc references in code, the comment rule
```
`tools/check_docs.py` exits with code 1 and lists every problem it finds; it is run before every release.
