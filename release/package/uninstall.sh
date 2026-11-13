#!/usr/bin/env bash
# doc/releases.md#uninstalling
set -euo pipefail
prefix="/usr/local"
while [ $# -gt 0 ]; do
    case "$1" in
        --prefix) prefix="$2"; shift 2 ;;
        *) echo "usage: $0 [--prefix <folder>]" >&2; exit 2 ;;
    esac
done
prefix="${prefix/#\~/$HOME}"
lib="$prefix/lib/targetscope"
if [ ! -f "$lib/VERSION" ]; then
    echo "no TargetScope installation in $prefix ($lib/VERSION is missing)" >&2
    exit 1
fi
version="$(cat "$lib/VERSION")"
rm -rf "$lib"
rm -f "$prefix/bin/targetscope"
echo "TargetScope $version removed from $prefix; data and configuration are kept"
