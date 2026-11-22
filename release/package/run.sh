#!/usr/bin/env bash
# doc/releases.md#running
set -euo pipefail
bundle="$(cd "$(dirname "$0")" && pwd)"
config="$bundle/etc/launcher.toml"
if [ ! -e "$config" ]; then
    mkdir -p "$bundle/etc"
    sed "s|^bin_dir *= *\"[^\"]*\"|bin_dir  = \"$bundle/lib/targetscope\"|" "$bundle/share/targetscope/launcher.toml" > "$config"
fi
deps="${TARGETSCOPE_DEPS:-$HOME/.local/opt/targetscope-deps}"
[ ! -d "$deps/bin" ] || export PATH="$deps/bin:$PATH"
exec "$bundle/bin/targetscope" --config "$config" "$@"
