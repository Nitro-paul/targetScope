# launcher

Program `launcher` (Rust): the single entry point of TargetScope. It starts Redis and all programs in the right order, restarts programs that crash, and stops everything cleanly. Requests and events: [interfaces.md](../interfaces.md#launcher).

## Responsibilities
- Makes sure only one launcher runs (lock file).
- Removes stale iceoryx2 shared memory of crashed programs.
- Starts `redis-server` with persistence and waits until it answers.
- Starts the programs in dependency order and waits until each one answers `ping`.
- Passes the bootstrap configuration to every program as environment variables.
- Collects stdout/stderr of all programs.
- Restarts crashed programs with exponential backoff.
- Stops everything in reverse order on Ctrl+C / SIGTERM.
- Reports program states (`launcher_status`, `program_state_changed`) and restarts programs on request (`restart_program`).

## Technology
| Purpose | Crate |
|---|---|
| Processes, signals | `tokio::process`, `tokio::signal`, `nix` (sending SIGTERM) |
| Configuration | `toml`, `serde` |
| Lock file | `fs4` |
| Redis, iceoryx2 cleanup | `targetscope_common` |

## Usage
TargetScope is always started by hand, by starting the launcher; there is no autostart. The launcher binary is installed as `targetscope`; it starts everything else.
```
targetscope                       # = launcher with ~/.config/targetscope/launcher.toml
targetscope --config <file>
targetscope --reset-admin-password
```
`--reset-admin-password` doesn't start anything: it connects to the Redis of the running TargetScope, sends `reset_settings` for `admin_password_hash` to `database_handler` (the web UI is open again), and exits with code 0. If TargetScope isn't running, it says so and exits with code 1.
The launcher doesn't care which language a program is written in: every program is a binary that reads the bootstrap environment variables and answers `ping`.
If the configuration file doesn't exist, the launcher writes the default one and uses it.

## Configuration (`launcher.toml`)
```toml
[paths]
data_dir = "~/.local/share/targetscope"
log_dir  = "~/.local/state/targetscope/logs"
bin_dir  = "/usr/local/lib/targetscope"     # where the program binaries are

[redis]
binary = "redis-server"
port   = 6379

[log]
level = "info"                               # tracing EnvFilter / spdlog level

[env]                                        # optional extra environment variables for every program

[[program]]
name = "database_handler"

[[program]]
name = "camera_reader"
depends_on = ["database_handler"]

[[program]]
name = "analyzer"

[[program]]
name = "image_processor"
depends_on = ["database_handler", "camera_reader"]

[[program]]
name = "session_manager"
depends_on = ["database_handler", "image_processor", "analyzer"]

[[program]]
name = "exporter"
depends_on = ["database_handler", "analyzer"]

[[program]]
name = "importer"
depends_on = ["database_handler"]

[[program]]
name = "statistics"
depends_on = ["database_handler"]

[[program]]
name = "web_server"
depends_on = ["database_handler", "session_manager"]

[restart]
initial_backoff_s = 1
max_backoff_s     = 60
stable_after_s    = 60      # backoff resets after running this long
max_restarts      = 10      # within window_s; then state "failed"
window_s          = 600
```
A program can be disabled with `enabled = false` (e.g. `camera_reader` on a PC without a camera). The binary is `<bin_dir>/<name>` unless `binary` is set.

### Bootstrap environment
The launcher derives the bootstrap variables from the configuration and sets them for every program, so each value is configured exactly once:

| Variable | Value |
|---|---|
| `TARGETSCOPE_REDIS_URL` | `redis://127.0.0.1:<redis.port>/` |
| `TARGETSCOPE_DATA_DIR` | `paths.data_dir` |
| `TARGETSCOPE_DB_PATH` | `<paths.data_dir>/targetscope.db` |
| `TARGETSCOPE_LOG_DIR` | `paths.log_dir` |
| `TARGETSCOPE_LOG` | `log.level` |

Variables in `[env]` are added; they can't override these five (an attempt is ignored with a warning in the log).

## Redis
Started as a child process:
```
redis-server --port <redis.port> --bind 127.0.0.1 --dir <data_dir>/redis
             --appendonly yes --appendfsync everysec --save ""
```
- AOF persistence is required: the current session of `session_manager` must survive a restart of Redis or the PC.
- Redis only listens on localhost; the browser never talks to Redis.
- If the port is already in use (e.g. by a system Redis), the launcher stops with an error message instead of using a foreign Redis. Another port can be set in `[redis] port`.

## Startup sequence
1. Take the lock `$XDG_RUNTIME_DIR/targetscope/launcher.lock`. If another launcher holds it: exit with a message.
2. Create the data and log folders; clean up stale iceoryx2 resources (`cleanup_dead_nodes`).
3. Start Redis; wait until `PING` answers (at most 10 s, otherwise exit).
4. Start the programs in dependency order (topological sort of `depends_on`; cycles and names of programs that aren't configured are configuration errors: the launcher exits with code 1 and a message; dependencies on disabled programs are ignored, so e.g. `image_processor` still starts without `camera_reader`). A program is started when all its dependencies are `running`. It is `running` once it answers `ping` (checked every 250 ms, at most 30 s; otherwise it's stopped and handled as a crash).
5. Serve `launcher:requests`.

## Supervision
| Event | Reaction |
|---|---|
| Program exits (any exit code) while not stopping | State `backoff`, restart after the current backoff (doubling from 1 s up to 60 s). Programs that depend on it keep running; they handle the missing program through timeouts. |
| Program ran longer than `stable_after_s` | Backoff resets to 1 s |
| More than `max_restarts` restarts within `window_s` | State `failed`, no more restarts; the web UI shows it; `restart_program` resets it (`not_found` for a name that isn't configured) |
| Redis exits | Restart Redis immediately (same backoff); programs reconnect on their own |

Every state change is sent as `program_state_changed` and logged.

## Logs
stdout and stderr of every program are read line by line, prefixed with the program name and printed to the launcher's own stdout. Each program additionally writes its own log file (see [shared_libraries.md](../shared_libraries.md#logging)); Redis logs to `<log_dir>/redis.log`.

## Shutdown
On SIGINT/SIGTERM (or Ctrl+C):
1. Stop restarting.
2. Send SIGTERM to the programs in reverse dependency order; wait for each up to 10 s, then SIGKILL.
3. Stop Redis with `SHUTDOWN` (writes the AOF), then exit.

## Internal design
```rust
struct Launcher {
    config: LauncherConfig,
    programs: HashMap<String, ProgramHandle>,
    bus: Option<Bus>,                  // available after Redis is up
}

struct ProgramHandle {
    spec: ProgramSpec,
    state: ProgramState,               // Starting | Running | Backoff | Failed | Stopped
    child: Option<tokio::process::Child>,
    restarts: VecDeque<Instant>,       // restart times within window_s
    backoff: Duration,
    since: SystemTime,
}
```
One supervisor task per program waits for its child to exit and reports to the main loop through a channel; the main loop owns all state.

## Testing
All test cases with their expected outcomes: [testing/launcher.md](../testing/launcher.md).
