#!/usr/bin/env bash
# doc/releases.md#building-and-running-in-one-step
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
build_args=()
while [ $# -gt 0 ] && [ "$1" != "--" ]; do
    build_args+=("$1")
    shift
done
[ $# -eq 0 ] || shift
# doc/releases.md#building-and-running-in-one-step
[ ${#build_args[@]} -gt 0 ] || build_args=(--without camera_reader,image_processor)
tarball="$("$root/release/build.sh" "${build_args[@]}" | tee /dev/stderr | tail -n 1)"
bundle="${tarball%.tar.gz}"
[ -x "$bundle/run.sh" ] || { echo "build didn't produce $bundle" >&2; exit 1; }
echo "starting $bundle" >&2
exec "$bundle/run.sh" "$@"
