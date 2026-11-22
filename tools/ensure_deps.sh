#!/usr/bin/env bash
# doc/development.md#checking-and-installing-the-dependencies
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
. "$root/tools/env.sh"

cpp=yes
run=no
cpp_libs_missing=no
for a in "$@"; do
    case "$a" in
        --no-cpp) cpp=no ;;
        --run) run=yes ;;
        *) echo "usage: $0 [--no-cpp] [--run]" >&2; exit 2 ;;
    esac
done

missing=()
need() { missing+=("$1|$2|$3|$4"); }
have() { command -v "$1" >/dev/null 2>&1; }

have cargo && have rustc || need "Rust toolchain (cargo, rustc; install with https://rustup.rs if the package is too old)" rust cargo cargo
have pkg-config || need "pkg-config" pkgconf pkg-config pkgconf-pkg-config
have pkg-config && ! pkg-config --exists libturbojpeg && need "libjpeg-turbo (libturbojpeg)" libjpeg-turbo libturbojpeg0-dev turbojpeg-devel
have cc || have gcc || need "C compiler" base-devel build-essential gcc

if [ "$cpp" = yes ]; then
    have git || need "git" git git git
    have make || need "make" base-devel build-essential make
    have g++ || have c++ || need "C++ compiler" base-devel build-essential gcc-c++
    have ninja || need "Ninja" ninja ninja-build ninja-build
    if have cmake; then
        v="$(cmake --version | sed -n '1s/[^0-9]*\([0-9]*\.[0-9]*\).*/\1/p')"
        [ "$(printf '%s\n3.24\n' "$v" | sort -V | head -n 1)" = 3.24 ] || need "CMake 3.24 or newer (found $v)" cmake cmake cmake
    else
        need "CMake" cmake cmake cmake
    fi
fi

if [ "$cpp" = yes ] && have cmake; then
    probe="$(mktemp -d)"
    trap 'rm -rf "$probe"' EXIT
    cat > "$probe/CMakeLists.txt" <<'P'
cmake_minimum_required(VERSION 3.24)
project(probe LANGUAGES CXX)
find_package(OpenCV QUIET COMPONENTS core imgproc imgcodecs videoio)
if(NOT OpenCV_FOUND OR OpenCV_VERSION VERSION_LESS 4 OR NOT OpenCV_VERSION VERSION_LESS 6)
    message(STATUS "MISSING opencv")
endif()
find_package(nlohmann_json 3.11 QUIET)
if(NOT nlohmann_json_FOUND)
    message(STATUS "MISSING nlohmann")
endif()
find_package(spdlog QUIET)
if(NOT spdlog_FOUND)
    message(STATUS "MISSING spdlog")
endif()
find_package(redis++ QUIET)
find_package(iceoryx2-cxx QUIET)
if(NOT redis++_FOUND OR NOT iceoryx2-cxx_FOUND)
    message(STATUS "MISSING cpplibs")
endif()
P
    out="$(cmake -S "$probe" -B "$probe/b" 2>&1 || true)"
    grep -q "MISSING opencv" <<<"$out" && need "OpenCV 4 or 5 with development files (core, imgproc, imgcodecs, videoio)" opencv libopencv-dev opencv-devel
    grep -q "MISSING nlohmann" <<<"$out" && need "nlohmann/json 3.11 or newer" nlohmann-json nlohmann-json3-dev json-devel
    grep -q "MISSING spdlog" <<<"$out" && need "spdlog" spdlog libspdlog-dev spdlog-devel
    ! grep -q "MISSING cpplibs" <<<"$out" || cpp_libs_missing=yes
fi

if [ ${#missing[@]} -gt 0 ]; then
    id="$(. /etc/os-release 2>/dev/null; echo "${ID:-} ${ID_LIKE:-}")"
    case "$id" in
        *arch*) col=2; cmd="sudo pacman -S --needed" ;;
        *debian*|*ubuntu*) col=3; cmd="sudo apt install" ;;
        *fedora*|*rhel*) col=4; cmd="sudo dnf install" ;;
        *) col=0; cmd="" ;;
    esac
    echo "TargetScope can't be built here yet. Missing:" >&2
    pkgs=()
    for m in "${missing[@]}"; do
        echo "  - ${m%%|*}" >&2
        [ "$col" = 0 ] || pkgs+=("$(cut -d'|' -f"$col" <<<"$m")")
    done
    if [ "$col" != 0 ]; then
        echo "Install them with:" >&2
        echo "  $cmd $(printf '%s\n' "${pkgs[@]}" | tr ' ' '\n' | sort -u | tr '\n' ' ')" >&2
    fi
    echo "(see doc/development.md#what-is-needed), then run this again." >&2
    exit 1
fi

wanted=()
[ "$cpp_libs_missing" = no ] || wanted+=(cpp)
[ "$run" = no ] || have redis-server || wanted+=(valkey)
if [ ${#wanted[@]} -gt 0 ]; then
    echo "Building the missing dependencies (${wanted[*]}) into $TARGETSCOPE_DEPS; this takes a few minutes the first time." >&2
    PREFIX="$TARGETSCOPE_DEPS" "$root/tools/install_deps.sh" "${wanted[@]}" >&2
    . "$root/tools/env.sh"
fi
