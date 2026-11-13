# Tests: launcher

Tests of the program `launcher`.

Design: [launcher.md](../programs/launcher.md), [interfaces.md](../interfaces.md#launcher). Reference data and tools: [overview.md](overview.md).

## Setup
- **Integration tests** run the launcher with a temporary `HOME`, a test configuration and `TestProgram` binaries in place of the real programs. `TestProgram` answers `ping` after a chosen delay, exits with a chosen code, ignores `SIGTERM`, or prints lines, as told by its arguments. Redis is a real `redis-server` started by the launcher on a free port.
- Timing cases use `FakeClock` where the launcher waits (backoff, windows); process start and stop use real time.

## Configuration (integration)
What is tested: reading and checking `launcher.toml` ([launcher.md](../programs/launcher.md#configuration-launchertoml)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LA-CFG-01 | No configuration file | Start without `~/.config/targetscope/launcher.toml` | The default file is written (content as in the design) and used |
| LA-CFG-02 | Own file | `targetscope --config <file>` | That file is used |
| LA-CFG-03 | Invalid file | TOML syntax error | Exit with code 1 and a message with the line |
| LA-CFG-04 | Dependency cycle | `a` depends on `b`, `b` on `a` | Exit with code 1, message names the cycle |
| LA-CFG-05 | Unknown dependency | `depends_on = ["nonexistent"]` | Exit with code 1, message names the program |
| LA-CFG-06 | Disabled program | `enabled = false` for `camera_reader` | Not started; `image_processor` (depends on it) starts anyway; `launcher_status` shows it `stopped` |
| LA-CFG-07 | Own binary | `binary = "/opt/x"` for one program | That binary is started |

## Bootstrap environment (integration)
What is tested: the variables every program gets ([launcher.md](../programs/launcher.md#bootstrap-environment)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LA-ENV-01 | Derived values | `[redis] port = 6390`, `data_dir = /tmp/t/data`, `log_dir = /tmp/t/log`, `level = "debug"`; `TestProgram` prints its environment | `TARGETSCOPE_REDIS_URL` `redis://127.0.0.1:6390/`, `TARGETSCOPE_DATA_DIR` `/tmp/t/data`, `TARGETSCOPE_DB_PATH` `/tmp/t/data/targetscope.db`, `TARGETSCOPE_LOG_DIR` `/tmp/t/log`, `TARGETSCOPE_LOG` `debug` |
| LA-ENV-02 | Extra variables | `[env] FOO = "bar"` | Every program has `FOO=bar` |
| LA-ENV-03 | No overriding | `[env] TARGETSCOPE_DB_PATH = "/x"` | The derived value is kept; a warning is logged |

## Lock (integration)
What is tested: only one launcher at a time.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LA-LCK-01 | Second launcher | Start a second launcher while one runs | It exits with code 1 and a message; the first is unaffected |
| LA-LCK-02 | After a hard kill | `kill -9` the launcher; start a new one | Starts (the lock was released with the process) |

## Redis (integration)
What is tested: starting, supervising and stopping Redis ([launcher.md](../programs/launcher.md#redis)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LA-RDS-01 | Options | `CONFIG GET` on the started Redis | `port` as configured, `bind` 127.0.0.1, `dir` `<data_dir>/redis`, `appendonly` yes, `appendfsync` everysec, `save` empty |
| LA-RDS-02 | Port in use | Another process listens on the port | Exit with code 1 and a message; no program started |
| LA-RDS-03 | Redis doesn't answer | A fake `redis-server` that never answers `PING` | Exit after 10 s with a message |
| LA-RDS-04 | Redis crashes | `kill -9` Redis | Restarted with backoff; `PING` answers again; programs reconnect |
| LA-RDS-05 | Clean stop | Stop the launcher | `SHUTDOWN` sent to Redis last; the AOF file contains the last writes |
| LA-RDS-06 | Redis from the deps folder | `[redis] binary = "redis-server"`, not in the `PATH`; a `redis-server` in `~/.local/opt/targetscope-deps/bin`, then also in `$TARGETSCOPE_DEPS/bin`, then also in the `PATH` (lookup function `find_binary_in`) | Found in the deps folder of `HOME`; then `$TARGETSCOPE_DEPS/bin` wins; then the `PATH` wins |
| LA-RDS-07 | Redis not found | Neither in the `PATH` nor in a deps folder; a full path that doesn't exist | Not found; the launcher would exit with code 1 and the message naming the searched places |

## Startup order (integration)
What is tested: starting in dependency order and waiting for `ping` ([launcher.md](../programs/launcher.md#startup-sequence)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LA-STA-01 | Order | Default program list; record start times | Every program starts only after all its dependencies answered `ping` |
| LA-STA-02 | Slow start | `database_handler` answers `ping` after 2 s | `starting` for 2 s, then `running`; its dependents start after that |
| LA-STA-03 | Never ready | A program never answers `ping` | Stopped after 30 s and handled like a crash (backoff) |
| LA-STA-04 | Stale shared memory | Leave iceoryx2 resources of a killed process; start | Removed before any program starts |
| LA-STA-05 | Own requests | `launcher_status` after startup | Answered; every configured program listed |

## Supervision (integration)
What is tested: restarts, backoff and failure ([launcher.md](../programs/launcher.md#supervision)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LA-SUP-01 | Backoff | A program exits right after starting, again and again | Restarted after 1, 2, 4, 8, 16, 32, 60, 60 s |
| LA-SUP-02 | Backoff reset | A program runs 61 s, then exits | Restarted after 1 s |
| LA-SUP-03 | Giving up | 11 exits within 600 s | State `failed`; no more restarts |
| LA-SUP-04 | Restart a failed program | `restart_program` for it | Started again; restart count reset |
| LA-SUP-05 | Restart a running program | `restart_program` for a running program | `SIGTERM`, then started again |
| LA-SUP-06 | Restart an unknown program | `restart_program` `{ name: "nonexistent" }` | `not_found` |
| LA-SUP-07 | Restart twice | The same `restart_program` twice with one `request_id` | One restart |
| LA-SUP-08 | State events | Every state change | `program_state_changed` with `name`, `state`, `pid`, `restarts`, `since_ms` |
| LA-SUP-09 | Status | `launcher_status` | Every program with state, `pid`, `restarts`, `since_ms` |
| LA-SUP-10 | Dependents keep running | `database_handler` crashes | Its dependents keep running (not restarted) |

## Shutdown (integration)
What is tested: stopping everything ([launcher.md](../programs/launcher.md#shutdown)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LA-STP-01 | Order | `SIGTERM` to the launcher | Programs get `SIGTERM` in reverse dependency order (`web_server` first, `database_handler` last); Redis last |
| LA-STP-02 | Program ignores `SIGTERM` | A program that ignores it | `SIGKILL` after 10 s |
| LA-STP-03 | No restarts while stopping | Programs exit during the shutdown | Not restarted |
| LA-STP-04 | Exit | After the shutdown | Exit code 0; lock released |

## Logs (integration)
What is tested: collected output ([launcher.md](../programs/launcher.md#logs)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LA-LOG-01 | Program output | `TestProgram` named `analyzer` prints a line on stdout and one on stderr | Both appear on the launcher's stdout, prefixed with `[analyzer]` |
| LA-LOG-02 | Redis log | After startup | `<log_dir>/redis.log` exists |

## Command line (integration)
What is tested: `--reset-admin-password` ([launcher.md](../programs/launcher.md#usage)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LA-CLI-01 | System running | Admin password set; `targetscope --reset-admin-password` | `reset_settings` for `admin_password_hash` sent; password inactive; exit code 0; nothing started |
| LA-CLI-02 | System not running | Same without a running TargetScope | Message "TargetScope isn't running"; exit code 1 |
