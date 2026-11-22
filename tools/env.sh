# doc/development.md#environment
TARGETSCOPE_DEPS="${TARGETSCOPE_DEPS:-$HOME/.local/opt/targetscope-deps}"
export TARGETSCOPE_DEPS
if [ -d "$TARGETSCOPE_DEPS" ]; then
    case ":$PATH:" in *":$TARGETSCOPE_DEPS/bin:"*) ;; *) export PATH="$TARGETSCOPE_DEPS/bin:$PATH" ;; esac
    case ":${CMAKE_PREFIX_PATH:-}:" in *":$TARGETSCOPE_DEPS:"*) ;; *) export CMAKE_PREFIX_PATH="$TARGETSCOPE_DEPS${CMAKE_PREFIX_PATH:+:$CMAKE_PREFIX_PATH}" ;; esac
    case ":${LD_LIBRARY_PATH:-}:" in *":$TARGETSCOPE_DEPS/lib:"*) ;; *) export LD_LIBRARY_PATH="$TARGETSCOPE_DEPS/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ;; esac
    case ":${PKG_CONFIG_PATH:-}:" in *":$TARGETSCOPE_DEPS/lib/pkgconfig:"*) ;; *) export PKG_CONFIG_PATH="$TARGETSCOPE_DEPS/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}" ;; esac
fi
