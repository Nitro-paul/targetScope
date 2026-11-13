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
| Image processing, camera | OpenCV with `core`, `imgproc`, `imgcodecs`, `calib3d`, `videoio` (V4L2 and FFmpeg backends) | 4.x or 5.x |
| C++ libraries | nlohmann/json, spdlog, GoogleTest | json ≥ 3.11, spdlog ≥ 1.12 |
| MJPEG feeds | libjpeg-turbo (`libturbojpeg`, found with `pkg-config`) | ≥ 2.1 |
| Frontend tests | Node.js | ≥ 20 |
| System tests | FFmpeg, v4l2loopback, mediamtx | see [testing/overview.md](testing/overview.md#camera-sources) |

SQLite is built into `database_handler` (`rusqlite` with the `bundled` feature); no system package is needed.

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
With a prefix in the home folder, make the tools and libraries visible (e.g. in `~/.profile`):
```sh
export TARGETSCOPE_DEPS=$HOME/.local/opt/targetscope-deps
export PATH="$TARGETSCOPE_DEPS/bin:$PATH"                       # redis-server for tests and the launcher
export CMAKE_PREFIX_PATH="$TARGETSCOPE_DEPS${CMAKE_PREFIX_PATH:+:$CMAKE_PREFIX_PATH}"
export LD_LIBRARY_PATH="$TARGETSCOPE_DEPS/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
```
The top-level `CMakeLists.txt` also adds `$TARGETSCOPE_DEPS` to the CMake search path when the variable is set.

## Build and test
See [README.md](../README.md#building) and [testing/overview.md](testing/overview.md#running-the-tests). Integration tests start their own `redis-server` from `PATH` on a free port; they never touch a running TargetScope.
