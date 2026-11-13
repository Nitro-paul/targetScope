#!/usr/bin/env bash
# doc/releases.md#building-a-release
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
out="$root/dist"
without=""
while [ $# -gt 0 ]; do
    case "$1" in
        --without) without="$2"; shift 2 ;;
        --out) out="$(mkdir -p "$2" && cd "$2" && pwd)"; shift 2 ;;
        *) echo "usage: $0 [--without <program>,<program>] [--out <folder>]" >&2; exit 2 ;;
    esac
done

rust_programs=(database_handler session_manager analyzer statistics exporter importer web_server)
cpp_programs=(camera_reader image_processor)
required=(launcher database_handler web_server)

left_out() { [[ ",$without," == *",$1,"* ]]; }

# doc/releases.md#version-numbers
version="$(tr -d '[:space:]' < "$root/VERSION")"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "VERSION '$version' isn't <release>.<majorfix>.<minorfix>" >&2; exit 1; }
cargo_version="$(sed -n '/^\[workspace.package\]/,/^\[/ s/^version = "\(.*\)"/\1/p' "$root/Cargo.toml")"
cmake_version="$(sed -n 's/^project(targetscope VERSION \([0-9.]*\).*/\1/p' "$root/CMakeLists.txt")"
if [ "$cargo_version" != "$version" ] || [ "$cmake_version" != "$version" ]; then
    echo "version mismatch: VERSION $version, Cargo.toml $cargo_version, CMakeLists.txt $cmake_version" >&2
    exit 1
fi
for p in ${without//,/ }; do
    for r in "${required[@]}"; do
        [ "$p" != "$r" ] || { echo "$p can't be left out" >&2; exit 1; }
    done
    [[ " ${rust_programs[*]} ${cpp_programs[*]} " == *" $p "* ]] || { echo "unknown program $p" >&2; exit 1; }
done

name="targetscope-$version-linux-$(uname -m)"
bundle="$out/$name"
rm -rf "$bundle" "$out/$name.tar.gz" "$out/$name.tar.gz.sha256"
mkdir -p "$bundle/bin" "$bundle/lib/targetscope" "$bundle/share/targetscope"

# doc/releases.md#building-a-release
packages=(-p launcher)
for p in "${rust_programs[@]}"; do
    left_out "$p" || packages+=(-p "$p")
done
target_dir="${CARGO_TARGET_DIR:-$root/target}"
(cd "$root" && cargo build --release "${packages[@]}" --bins)
cp "$target_dir/release/targetscope" "$bundle/bin/targetscope"
for p in "${rust_programs[@]}"; do
    left_out "$p" || cp "$target_dir/release/$p" "$bundle/lib/targetscope/$p"
done
cp -r "$root/programs/web_server/static" "$bundle/lib/targetscope/static"

cpp_wanted=()
for p in "${cpp_programs[@]}"; do
    left_out "$p" || cpp_wanted+=("$p")
done
if [ ${#cpp_wanted[@]} -gt 0 ]; then
    build="$root/build-release"
    cmake -S "$root" -B "$build" -DCMAKE_BUILD_TYPE=Release -DTARGETSCOPE_BUILD_TESTS=OFF -DTARGETSCOPE_FAILPOINTS=OFF
    cmake --build "$build" --target "${cpp_wanted[@]}"
    for p in "${cpp_wanted[@]}"; do
        install -m 755 "$build/programs/$p/$p" "$bundle/lib/targetscope/$p"
    done
fi

# doc/releases.md#bundle
echo "$version" > "$bundle/VERSION"
echo "$version" > "$bundle/lib/targetscope/VERSION"
cp "$root/doc/releases.md" "$bundle/README.md"
install -m 755 "$root/release/package/install.sh" "$root/release/package/uninstall.sh" "$root/release/package/run.sh" "$bundle/"
cp "$root/config/launcher.toml" "$bundle/share/targetscope/launcher.toml"
for p in ${without//,/ }; do
    sed -i "/^name = \"$p\"$/a enabled = false" "$bundle/share/targetscope/launcher.toml"
done

(cd "$out" && tar czf "$name.tar.gz" "$name" && sha256sum "$name.tar.gz" > "$name.tar.gz.sha256")
echo "$out/$name.tar.gz"
