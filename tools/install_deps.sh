#!/usr/bin/env bash
# doc/development.md#install-without-root
set -euo pipefail

parts=("$@")
[ ${#parts[@]} -gt 0 ] || parts=(valkey cpp)
wants() { [[ " ${parts[*]} " == *" $1 "* ]]; }

PREFIX=${PREFIX:-$HOME/.local/opt/targetscope-deps}
SRC=${SRC:-$HOME/.local/src/targetscope-deps}
JOBS=$(nproc)
mkdir -p "$PREFIX" "$SRC"
cd "$SRC"

clone() {
    [ -d "$2" ] || git clone -q --depth 1 -b "$1" "$3" "$2"
}

cmake_install() {
    local dir=$1
    shift
    cmake -S "$dir" -B "$dir/build" -G Ninja -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX="$PREFIX" -DCMAKE_PREFIX_PATH="$PREFIX" -DCMAKE_POLICY_VERSION_MINIMUM=3.5 "$@"
    cmake --build "$dir/build"
    cmake --install "$dir/build"
}

if wants valkey && [ ! -x "$PREFIX/bin/redis-server" ]; then
    clone 9.1.3 valkey https://github.com/valkey-io/valkey.git
    make -C valkey -j"$JOBS" BUILD_TLS=no
    make -C valkey PREFIX="$PREFIX" install
fi

if wants cpp && [ ! -f "$PREFIX/lib/cmake/hiredis/hiredis-config.cmake" ]; then
    clone v1.3.1 hiredis https://github.com/redis/hiredis.git
    cmake_install hiredis -DDISABLE_TESTS=ON
fi

if wants cpp && [ ! -f "$PREFIX/share/cmake/redis++/redis++-config.cmake" ]; then
    clone 1.3.9 redis-plus-plus https://github.com/sewenew/redis-plus-plus.git
    cmake_install redis-plus-plus -DREDIS_PLUS_PLUS_BUILD_TEST=OFF -DREDIS_PLUS_PLUS_CXX_STANDARD=20 \
        -DCMAKE_CXX_FLAGS=-Wno-maybe-uninitialized
fi

if wants cpp && [ ! -f "$PREFIX/lib/cmake/iceoryx2-cxx/iceoryx2-cxxConfig.cmake" ]; then
    clone v0.10.0 iceoryx2 https://github.com/eclipse-iceoryx/iceoryx2.git
    cmake_install iceoryx2 -DBUILD_CXX=ON -DBUILD_EXAMPLES=OFF -DBUILD_TESTING=OFF
fi

echo "Installed into $PREFIX. The build scripts find it by themselves (tools/env.sh); for your own shell:"
echo "  . $(cd "$(dirname "$0")" && pwd)/env.sh"
