#!/usr/bin/env bash
# doc/releases.md#installing
set -euo pipefail
bundle="$(cd "$(dirname "$0")" && pwd)"
prefix="/usr/local"
while [ $# -gt 0 ]; do
    case "$1" in
        --prefix) prefix="$2"; shift 2 ;;
        *) echo "usage: $0 [--prefix <folder>]" >&2; exit 2 ;;
    esac
done
prefix="${prefix/#\~/$HOME}"
mkdir -p "$prefix" && prefix="$(cd "$prefix" && pwd)"
version="$(cat "$bundle/VERSION")"
lib="$prefix/lib/targetscope"

if [ ! -w "$prefix" ] || { [ -e "$prefix/bin" ] && [ ! -w "$prefix/bin" ]; }; then
    echo "$prefix isn't writable; run with sudo or use --prefix ~/.local" >&2
    exit 1
fi
mkdir -p "$prefix/bin" "$prefix/lib"
rm -rf "$lib.new"
cp -r "$bundle/lib/targetscope" "$lib.new"
rm -rf "$lib"
mv "$lib.new" "$lib"
install -m 755 "$bundle/bin/targetscope" "$prefix/bin/targetscope"
echo "TargetScope $version installed: $prefix/bin/targetscope, $lib"

# doc/releases.md#installing
if [ -n "${SUDO_USER:-}" ]; then
    user="$SUDO_USER"
    home="$(getent passwd "$user" | cut -d: -f6)"
else
    user="$(id -un)"
    home="$HOME"
fi
config="$home/.config/targetscope/launcher.toml"
if [ ! -e "$config" ]; then
    if [ "$lib" != "/usr/local/lib/targetscope" ]; then
        mkdir -p "$(dirname "$config")"
        sed "s|^bin_dir *= *\"[^\"]*\"|bin_dir  = \"$lib\"|" "$bundle/share/targetscope/launcher.toml" > "$config"
        [ -z "${SUDO_USER:-}" ] || chown -R "$user": "$home/.config/targetscope"
        echo "configuration written: $config"
    fi
else
    bin_dir="$(sed -n 's/^bin_dir *= *"\([^"]*\)".*/\1/p' "$config")"
    if [ "${bin_dir/#\~/$home}" != "$lib" ]; then
        echo "warning: $config has bin_dir = \"$bin_dir\", not \"$lib\"; the launcher won't find this installation" >&2
    fi
fi

command -v redis-server >/dev/null || echo "warning: redis-server isn't in the PATH; install Redis or Valkey (doc/development.md)" >&2
case ":$PATH:" in
    *":$prefix/bin:"*) ;;
    *) echo "note: $prefix/bin isn't in the PATH" >&2 ;;
esac
