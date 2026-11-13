#!/usr/bin/env bash
# doc/releases.md#raising-the-version
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
part="${1:-}"
IFS=. read -r release majorfix minorfix < "$root/VERSION"
case "$part" in
    release)  release=$((release + 1)); majorfix=0; minorfix=0 ;;
    majorfix) majorfix=$((majorfix + 1)); minorfix=0 ;;
    minorfix) minorfix=$((minorfix + 1)) ;;
    *) echo "usage: $0 release|majorfix|minorfix" >&2; exit 2 ;;
esac
version="$release.$majorfix.$minorfix"
echo "$version" > "$root/VERSION"
sed -i "/^\[workspace.package\]/,/^\[/ s/^version = \".*\"/version = \"$version\"/" "$root/Cargo.toml"
sed -i "s/^project(targetscope VERSION [0-9.]*/project(targetscope VERSION $version/" "$root/CMakeLists.txt"
echo "$version"
