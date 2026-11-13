# Tests: shared libraries and message contract

Tests of `targetscope_common` (Rust), `targetscope_common_cpp` (C++) and the message contract in `schemas/`. The frame helpers are tested in [frame_transport.md](frame_transport.md).

Design: [shared_libraries.md](../shared_libraries.md), [interfaces.md](../interfaces.md#language-neutral-contract). Reference data and tools: [overview.md](overview.md).

Unless a group says otherwise, every case runs in Rust **and** in C++ with the same input and the same expected outcome.

## Contract: schemas and examples (contract)
What is tested: every message and shared type has a schema, the examples are right, and both languages read and write exactly what the schemas define.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LIB-CON-01 | Every schema has an example | List the schemas in `schemas/` and the files in `schemas/examples/` | Every schema (requests, responses, events, shared types, `export/v1`) has at least one valid example |
| LIB-CON-02 | Valid examples are valid | Validate every file in `schemas/examples/` (JSON Schema draft 2020-12) | All pass |
| LIB-CON-03 | Invalid examples are invalid | Validate the `message` of every file in `schemas/examples/invalid/`; each file names its expected error in `expected_error`, starting with the field (`"<field>: <reason>"`) | Every file fails, with an error about the named field (e.g. missing `request_id`, `angle_turns: 1.0`, `status: "maybe"`, `paper_no: 0`) |
| LIB-CON-04 | Round trip | Deserialize every valid example into its type, serialize again | Equal JSON value (field order ignored) |
| LIB-CON-05 | Produced messages are valid | Build every message type in code, with every enum variant and every optional field once set and once unset; serialize | Valid against its schema |
| LIB-CON-06 | Invalid input is rejected | Deserialize every invalid example | Error; no panic, no escaping exception, no half-filled object |
| LIB-CON-07 | Completeness, Rust | Compare the Rust message and shared types with the list of schemas | No Rust type without schema, no schema without Rust type |
| LIB-CON-08 | Completeness, C++ | Compare the C++ types with the schemas of everything `camera_reader` and `image_processor` receive, send or publish | All present |
| LIB-CON-09 | Rust and C++ agree | For every shared type used by both: Rust serializes the example, C++ deserializes and serializes it, Rust deserializes it | Equal to the original example |
| LIB-CON-10 | Missing and `null` optional fields | An example with an optional field missing, and the same example with the field `null` | Both deserialize to "not set" |
| LIB-CON-11 | Field names | Check every property name in all schemas | All `snake_case` (`^[a-z][a-z0-9_]*$`) |

## Envelopes (unit)
What is tested: the request, response and event envelopes ([interfaces.md](../interfaces.md#message-format)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LIB-ENV-01 | Request envelope | Serialize a `ping` request with `reply_to` | Top-level fields `type`, `version`, `request_id`, `reply_to`, `payload`; `request_id` is a UUID v4 (version nibble 4) |
| LIB-ENV-02 | Request without `reply_to` | Serialize a request without `reply_to` | `reply_to` missing or `null`; valid against the schema |
| LIB-ENV-03 | Response type name | Build the response to a `create_session` request | `type` = `create_session_response`; same `request_id` and `version` as the request |
| LIB-ENV-04 | Ok response | Response with a payload | `status: "ok"`, `error: null`, `payload` set |
| LIB-ENV-05 | Error response | Error `setting_not_set` with `details: {"setting": "export_dir"}` | `status: "error"`, `payload: null`, `error.code`, `error.message` and `error.details` set |
| LIB-ENV-06 | Event envelope | Build a `shot_detected` event | `type`, `version`, `event_id` (UUID v4), `source`, `timestamp_ms`, `payload` |
| LIB-ENV-07 | Unknown request type | Parse a request with `type: "nonsense"` into a program's request type | Error `invalid_message` |
| LIB-ENV-08 | Unknown version | Parse a known request type with `version: 99` | Error `invalid_message` |

## Geometry (unit)
What is tested: conversion between the polar Shot coordinates and Cartesian millimeters ([interfaces.md](../interfaces.md#shot)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LIB-GEO-01 | Polar to Cartesian, four directions | (`angle_turns`, `radius_mm`) = (0, 10), (0.25, 10), (0.5, 10), (0.75, 10) | (x, y) = (0, 10), (10, 0), (0, −10), (−10, 0) |
| LIB-GEO-02 | Polar to Cartesian, diagonal | (0.125, 10) | (7.0710678, 7.0710678) |
| LIB-GEO-03 | Cartesian to polar, center | (0, 0) | (0, 0) |
| LIB-GEO-04 | Cartesian to polar, four directions | (0, 10), (10, 0), (0, −10), (−10, 0) | (0, 10), (0.25, 10), (0.5, 10), (0.75, 10) |
| LIB-GEO-05 | Angle stays below 1 | (−1e-17, 10), just left of the top | `angle_turns` = 0, never 1.0 |
| LIB-GEO-06 | Round trip | 100 000 random positions, `angle_turns` in [0, 1), `radius_mm` in (0.001, 200] | Back to the same position within 1e-9 |
| LIB-GEO-07 | Rust and C++ agree | The same 1 000 random inputs in both languages | Equal within 1e-12 |

## Ring generation and target type validation (unit)
What is tested: `generate_rings` and `validate_target_type` ([interfaces.md](../interfaces.md#targettype)). The web UI's JavaScript runs the same cases ([WS-JS-01](web_server.md#frontend-unit-tests-unit)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LIB-TGT-01 | Rings of `TT-REF` | `generate_rings(TT-REF)` | 12 rings, inside out: 12: 5, 11: 10, 10: 15, 9: 20, 8: 25, 7: 30, 6: 40, 5: 50, 4: 60, 3: 70, 2: 80, 1: 90 |
| LIB-TGT-02 | Anchor radii are exact | Anchors white 2.5 (12), black 29.75 (7), outer 77.75 (1) | Rings 12, 7 and 1 have exactly 2.5, 29.75 and 77.75 (bit-equal, not recalculated); rings 11–8 are 5.45 apart, rings 6–2 are 8 apart |
| LIB-TGT-03 | Neighboring anchor scores | Anchors white 5 (10), black 10 (9), outer 50 (1) | 10: 5, 9: 10, 8: 15, 7: 20, 6: 25, 5: 30, 4: 35, 3: 40, 2: 45, 1: 50 |
| LIB-TGT-04 | Outermost score above 1 | `TT-REF` with `outer_score` 3 | 10 rings: 12: 5 … 7: 30, then 6: 45, 5: 60, 4: 75, 3: 90 |
| LIB-TGT-05 | Valid target type | `validate_target_type(TT-REF)` | Ok |
| LIB-TGT-06 | Outermost ring touching the paper edge | `TT-REF` with `center_x_mm` 90 | Ok (90 − 90 = 0) |
| LIB-TGT-07 | Invalid target types | `TT-REF` with one change at a time, see the table below | Error naming the broken rule |
| LIB-TGT-08 | All example targets, all languages | Every file in `testdata/target_types/` (with its expected rings and validity) in Rust, C++ and JavaScript | Identical rings and identical validation results |

Inputs for LIB-TGT-07:

| Change to `TT-REF` | Broken rule |
|---|---|
| `inner_white_score` 7 | `inner_white_score > black_score` |
| `black_score` 1 | `black_score > outer_score` |
| `outer_score` 0 | `outer_score >= 1` |
| `inner_white_radius_mm` 0 | `inner_white_radius_mm > 0` |
| `inner_white_radius_mm` 30 | `inner_white_radius_mm < black_radius_mm` |
| `black_radius_mm` 90 | `black_radius_mm < outer_radius_mm` |
| `center_x_mm` 89 | outermost ring on the paper (left edge) |
| `center_x_mm` 111 | outermost ring on the paper (right edge) |
| `center_y_mm` 89 | outermost ring on the paper (top edge) |
| `center_y_mm` 111 | outermost ring on the paper (bottom edge) |
| `paper_width_mm` 0 | paper size `> 0` |

## Redis bus (integration)
What is tested: sending requests, serving requests and reading events ([shared_libraries.md](../shared_libraries.md#redis-bus)). Setup: `RedisTestServer`; two test programs `A` and `B` built with the library.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LIB-BUS-01 | Request and response | `A` sends `ping` to `B` (serve loop running) | `ping_response` with `program` = `B`, `program_version`, `language`, `started_at` ≤ now |
| LIB-BUS-02 | Responses are matched | `A` sends 100 requests at once; `B` answers them in random order | Every caller gets the response with its own `request_id` |
| LIB-BUS-03 | Timeout | Test request type with a 200 ms timeout; `B` doesn't answer | `Timeout` error after 200 ms |
| LIB-BUS-04 | Late response is dropped | As LIB-BUS-03, `B` answers after 400 ms; then `A` sends another request | The late response is dropped and logged; the next request gets its own response |
| LIB-BUS-05 | Resend with the same id | `request_with_id(id)` twice | `B` receives both requests with the same `request_id` |
| LIB-BUS-06 | Remote error | `B`'s handler returns `not_found` with a message and `details` | `Remote` error with the same code, message and details |
| LIB-BUS-07 | Stream entry format | Inspect the entry `A` wrote | Exactly one field, `data`, holding the envelope as UTF-8 JSON |
| LIB-BUS-08 | Unknown request type | Write a request with `type: "nonsense"` and `reply_to` | `invalid_message` sent to `reply_to`; request acknowledged (`XPENDING` empty) |
| LIB-BUS-09 | Unparseable entry | Write `data = "garbage"` | No response; acknowledged; error logged |
| LIB-BUS-10 | Unknown version | Known type with `version: 99` | `invalid_message`; acknowledged |
| LIB-BUS-11 | No `reply_to` | Valid request without `reply_to` | Handler runs; no response written; acknowledged |
| LIB-BUS-12 | Pending requests first | `B` crashes at `bus.before_ack`; one more request is queued; `B` restarts | The pending request is handled first (again), then the new one |
| LIB-BUS-13 | Consumer group already exists | Start `B`, stop it, start it again | No error (`BUSYGROUP` ignored); pending entries kept |
| LIB-BUS-14 | Consumer name | `XINFO CONSUMERS` on `B:requests` | Consumer named `<hostname>-<pid>` |
| LIB-BUS-15 | Stream caps | Write 25 000 events, 25 000 requests and 5 000 responses | `XLEN` about 10 000 for events and requests, about 1 000 for responses (approximate trimming; never more than 10 % above) |
| LIB-BUS-16 | Published event | `B.publish(shot_detected)` | Entry in `B:events` with `event_id` (UUID v4), `source` = `B`, `timestamp_ms` within 1 s of now, `type`, `payload` |
| LIB-BUS-17 | Consumer group reader keeps its position | Reader in group mode reads 3 events and stops; 5 events are published; reader starts again | The 5 new events are delivered; none of the first 3 again |
| LIB-BUS-18 | Plain reader misses nothing at startup | Plain reader notes the newest id; 3 events are published while it "loads" (`settings.after_note_id` waits); then it reads | The 3 events are delivered |
| LIB-BUS-19 | Plain reader continuity | 1 000 events published in bursts while the plain reader runs | Each delivered exactly once, in order |
| LIB-BUS-20 | Redis restart | Stop the Redis test server for 3 s, then start it again on the same folder | The bus reconnects with backoff; requests during the outage fail with a Redis error or a timeout; afterwards requests and the serve loop work again |
| LIB-BUS-21 | `ping` is answered by the serve loop | Send `ping` to a program whose handler records calls | `ping_response`; the handler isn't called |
| LIB-BUS-22 | `serve_concurrent` answers at once | One request at a time to an idle `serve_concurrent` loop, 5 times, 1.5 s apart (its read loop waits in `XREADGROUP BLOCK`) | Every response within 200 ms |
| LIB-BUS-23 | Many pending requests at start | 250 requests delivered to a consumer that never acknowledged them; then `serve_concurrent` starts | All 250 handled once and answered; nothing pending afterwards |

## Idempotency store (integration, Rust)
What is tested: the Redis record of processed requests used by `session_manager` and `launcher`.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LIB-IDS-01 | Unknown id | `get(new UUID)` | None |
| LIB-IDS-02 | Recorded in a transaction | `MULTI`, `SET x 1`, `record(id, response)`, `EXEC`; then `get(id)` | The stored response; key `<program>:processed:<id>`; `TTL` between 86 390 and 86 400 s; `x` set |
| LIB-IDS-03 | Discarded transaction | `MULTI`, `record(id, response)`, `DISCARD` | `get(id)` returns None |

## Settings cache (integration)
What is tested: loading settings and keeping them current ([shared_libraries.md](../shared_libraries.md#settings-cache)). Setup: `FakeProgram` as `database_handler` with the known settings; `EventProbe` to publish `data_changed`.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LIB-SET-01 | Load | `SettingsCache::load` | `get` returns the value of every active setting |
| LIB-SET-02 | Unset optional setting | `export_dir` with `active = false`, value `~/x` | `get("export_dir")` returns None |
| LIB-SET-03 | Parsed values | `camera_fps` = `30`, then `abc` | `get_parsed::<u32>` returns 30, then None |
| LIB-SET-04 | Paths | `~/TargetScope/exports`; `/srv/exports` | `$HOME/TargetScope/exports`; `/srv/exports` unchanged |
| LIB-SET-05 | Change event | Publish `data_changed` (settings, `camera_fps`, updated) with a new value in the fake | The setting is read again; `get` returns the new value within 1 s |
| LIB-SET-06 | No gap at startup | A change is published after the newest id was noted, before the settings are read (`settings.after_note_id` waits) | After `load`, the changed value is visible |
| LIB-SET-07 | `watch` | `watch(["camera_fps"])`; change `camera_fps`, then `live_feed_fps` | Notified once, for `camera_fps` only |
| LIB-SET-08 | Reload everything | Publish `data_changed` with `table: "all"`, `action: "reload"` | All settings are read again |

## Bootstrap configuration (unit)
What is tested: reading the variables the launcher sets ([shared_libraries.md](../shared_libraries.md#bootstrap-configuration)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LIB-CFG-01 | All variables set | Set the five `TARGETSCOPE_*` variables | All fields have the given values |
| LIB-CFG-02 | Defaults | No variables set | `redis://127.0.0.1:6379/`, `~/.local/share/targetscope`, `<data_dir>/targetscope.db`, `~/.local/state/targetscope/logs`, `info` (`~` expanded) |
| LIB-CFG-03 | Rust and C++ agree | Same environment in both languages | Same values |

## Logging (integration)
What is tested: log outputs ([shared_libraries.md](../shared_libraries.md#logging)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LIB-LOG-01 | Two outputs | `init("test")`, log one line | The line on stderr and in `<log_dir>/test.log` |
| LIB-LOG-02 | Daily file | `FakeClock` moves past midnight; log again | New file for the new day; the old file is kept |
| LIB-LOG-03 | Request id in every line | Serve a request whose handler logs 3 lines | All 3 lines contain the request's `request_id` |
| LIB-LOG-04 | Level | `TARGETSCOPE_LOG=warn`; log an info and a warning line | Only the warning is written |

## Clock (unit)
What is tested: the shared `Clock` used by all time-dependent code ([shared_libraries.md](../shared_libraries.md#clock-and-failpoints)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| LIB-CLK-01 | Fake time | `FakeClock` at T; a task sleeps 10 s; advance by 10 s | `now()` = T + 10 s; the sleep ends after the advance, not before |
| LIB-CLK-02 | Real clock | `SystemClock::now()` twice, 100 ms apart | Second value 100 ms later (±20 ms) |
