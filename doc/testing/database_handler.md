# Tests: database_handler

Tests of the program `database_handler` and of the database schema.

Design: [database_handler.md](../programs/database_handler.md), [database.md](../database.md), [interfaces.md](../interfaces.md#database_handler). Reference data and tools: [overview.md](overview.md).

## Setup
- **Integration tests** run `database_handler` as a process with `RedisTestServer`, `TempDataDir` and the `TARGETSCOPE_*` variables. Requests are sent with `TestBus`; events are checked with `EventProbe`.
- **Schema tests** (group `DB-SQL`) run SQL directly on a migrated database file, without the program.
- **Fixtures:** `TT-REF`, `DEV-REF`, `SH-REF`, `S-REF` ([overview](overview.md#reference-data)); `FILTER-SET` (below); staged images from `testdata/images/`.
- **`CS-REF`**, the standard `create_session` payload: shooter `SH-REF`, device `DEV-REF`, target type `TT-REF`, `distance_m` 10, `single_handed` true, `supported` false, shots `S-REF` A–E all on `paper_no` 1, `shot_count` 5, `paper_count` 1, `score` 38, `grouping_score` 45, `duration_s` 600, `notes` `"test"`, `images` `[{ "paper_no": 1, "staged_file": "upload_paper.jpg" }]` (copied from `testdata/images/paper.jpg`). The database stores scores as given; it doesn't calculate them.

## Startup and schema creation (integration)
What is tested: first start, migrations, default settings, ready signal ([database_handler.md](../programs/database_handler.md#startup)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-INI-01 | First start | Start with an empty data folder | Database file and parent folder created; `PRAGMA user_version` = number of migrations; 8 tables (`settings`, `shooters`, `devices`, `target_types`, `sessions`, `shots`, `session_images`, `processed_requests`), 6 indexes and 6 triggers as in [database.md](../database.md#schema-sql); image folder with `devices/` and `sessions/`; staging folder |
| DB-INI-02 | Connection settings | Open the database with the handler's `Database::open` in the test process; query the pragmas on its connection | `foreign_keys` 1, `journal_mode` `wal`, `busy_timeout` 5000 |
| DB-INI-03 | Default settings | `read_settings` with an empty filter after the first start | All 20 known settings with their default value and default `active`; `image_dir` = `<TARGETSCOPE_DATA_DIR>/images`, `staging_dir` = `<TARGETSCOPE_DATA_DIR>/staging`; `export_dir` and `camera_rtsp_url` inactive |
| DB-INI-04 | User changes kept | Set `camera_fps` to 15; restart | `camera_fps` is still 15 |
| DB-INI-05 | No target types | `read_target_types` after the first start | `[]` |
| DB-INI-06 | Migration of an older database | Database at `user_version` 1 with data; `schema::migrate` in the test process with the real migrations plus a test migration 2 | Only migration 2 runs; `user_version` 2; data kept |
| DB-INI-07 | Failing migration | As DB-INI-06, but test migration 2 creates a table and then contains an SQL error; then start the program on a database file it can't open (not a database) | Migration 2 is rolled back (no new table); `user_version` stays 1; `schema::migrate` returns the error; the program exits with an error code |
| DB-INI-08 | Orphan files | A file in `sessions/` without a record; start | A warning with the file's path is logged; the file is still there |
| DB-INI-09 | Ready only when started | Slow migration (failpoint `db.migrate` waits 3 s); send `ping` during it | No `ping_response` until startup is complete; then answered |
| DB-INI-10 | Reload event after start | Start; read `database_handler:events` | One `data_changed` with `table: "all"`, `action: "reload"` |

## Schema rules (unit, SQL)
What is tested: constraints and triggers that protect records, also against programming errors in the handler ([database.md](../database.md#records)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-SQL-01 | Sessions are records | `UPDATE sessions SET notes = 'x'` on a saved session | Aborted: `record: sessions cannot be changed` |
| DB-SQL-02 | Shots are records | `UPDATE shots SET radius_mm = 1` | Aborted: `record: shots cannot be changed` |
| DB-SQL-03 | Session images are records | `UPDATE session_images SET image_path = 'x'` | Aborted: `record: session images cannot be changed` |
| DB-SQL-04 | Deleting a session | `DELETE FROM sessions` for one session | Its `shots` and `session_images` rows are gone |
| DB-SQL-05 | Locked target type | Target type used by a session; `UPDATE` each geometry column, each score column and `uuid` (one statement each) | Each aborted: `in_use: target type is used by a session` |
| DB-SQL-06 | Locked target type, free columns | Same target type; `UPDATE` `name`, then `retired` | Both succeed |
| DB-SQL-07 | Unused target type | `UPDATE` a geometry column of an unused target type | Succeeds |
| DB-SQL-08 | Locked device | Device used by a session; `UPDATE` `bullet_diameter_mm`, then `uuid` | Both aborted: `in_use: bullet diameter is used by a session` |
| DB-SQL-09 | Locked device, free columns | Same device; `UPDATE` `name`, `category`, `description`, `caliber`, `image_path`, `retired` | All succeed |
| DB-SQL-10 | Shot count never decreases | `UPDATE devices SET shot_count = shot_count - 1`; then `+ 1` | First aborted (`record: shot_count cannot decrease`); second succeeds |
| DB-SQL-11 | Shooter names ignore case | Insert `Alex`, then `ALEX` | Second insert fails (unique) |
| DB-SQL-12 | Shooter names are trimmed | Insert `' Alex'`, then `''` | Both fail (check) |
| DB-SQL-13 | Target type names ignore case | Insert target types `TT-REF` and `tt-ref` | Second fails (unique) |
| DB-SQL-14 | Shot ranges | Insert shots with `angle_turns` 1.0, −0.1; `radius_mm` −0.1; `shot_no` 0; `paper_no` 0; the same `shot_no` twice in a session | Each fails |
| DB-SQL-15 | Session ranges | Insert sessions with `shot_count` 0, `paper_count` 0, `distance_m` −1, `duration_s` −1, `notes` of 10 001 characters, `single_handed` 2 | Each fails |
| DB-SQL-16 | Target type rules | Insert target types breaking each rule of [LIB-TGT-07](shared_libraries.md#ring-generation-and-target-type-validation-unit) | Each fails |
| DB-SQL-17 | Device ranges | Insert devices with `bullet_diameter_mm` 0 and `shot_count` −1 | Each fails |
| DB-SQL-18 | Foreign keys | Insert a session with an unknown `device_id`; delete a device, a target type and a shooter that a session uses | Each fails |

## Sessions: create, read, delete (integration)
What is tested: `create_session`, reading single sessions and `delete_session` ([interfaces.md](../interfaces.md#sessions)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-SES-01 | Create with one paper and image | `create_session` with `CS-REF` | Response `{ id, uuid, images: [{ paper_no: 1, image_path: "sessions/<uuid>_1.jpg" }] }`; `uuid` is a UUID v4; the row has every value of the payload and `saved_at` within 2 s of now; 5 `shots` rows with `shot_no` 1–5 in payload order and `paper_no` 1; one `session_images` row; the image is gone from the staging folder and in the image folder with the same bytes; `DEV-REF.shot_count` + 5 |
| DB-SES-02 | Several papers, some without image | `CS-REF` with `paper_count` 3, shots on papers 1, 2 and 3, images for papers 1 and 3 | `session_images` rows for papers 1 and 3 only; files `<uuid>_1.jpg` and `<uuid>_3.jpg` |
| DB-SES-03 | No images | `CS-REF` with `images: []` | Ok; `images: []` |
| DB-SES-04 | PNG image | Staged `device.png` as the paper image | Stored as `sessions/<uuid>_1.png` |
| DB-SES-05 | Invalid payloads | `CS-REF` with one change at a time: no shots; `shot_count` 4; `paper_count` 0; a shot with `paper_no` 0 or 2; an image with `paper_no` 2; two images for paper 1; `notes` of 10 001 characters; unknown `device_id`, `target_type_id` or `shooter_id`; a shot with `angle_turns` 1.0 or `radius_mm` −1; `distance_m` −1; `duration_s` −1 | Each: `validation_failed`; no rows; staged files untouched; `shot_count` unchanged; no event |
| DB-SES-06 | Longest notes | `notes` of exactly 10 000 characters | Ok |
| DB-SES-07 | Staged image missing | `staged_file` doesn't exist | `not_found` |
| DB-SES-08 | Staged name with a path | `staged_file` `../paper.jpg`, then `sub/paper.jpg` | `validation_failed` |
| DB-SES-09 | Staged file isn't an image | `not_an_image.jpg` (text) | `validation_failed` |
| DB-SES-10 | Failure before commit | `db.before_commit` returns an error | `database_error`; no rows; the image is back in the staging folder under its name with the same bytes; `shot_count` unchanged; no `processed_requests` row; no event |
| DB-SES-11 | Read one session | `read_sessions` with `session_ids` [id], `include_shots` true | The session with all fields; shots ordered by `shot_no`, each with `paper_no`; `images` ordered by `paper_no`; `total` 1 |
| DB-SES-12 | Without shots | Same with `include_shots` false | `shots: []`; all other fields present |
| DB-SES-13 | Delete | `delete_session` | Response `{ id }`; session, shots and image rows gone; image files deleted; `shot_count` of the device unchanged; event (see DB-EVT) |
| DB-SES-14 | Delete unknown | `delete_session` with an unknown id | `not_found` |
| DB-SES-15 | Delete with a missing image file | Delete the image file by hand; `delete_session` | Ok; warning logged |
| DB-SES-16 | Failure before commit when deleting | `db.before_commit` returns an error | `database_error`; nothing deleted, files still there |

## Session filter (integration)
What is tested: `read_sessions` with every field of the [SessionFilter](../interfaces.md#sessionfilter), ordering and paging. Setup: fixture `FILTER-SET` (`testdata/sessions/filter_set.json`).

`FILTER-SET`: shooters `Alex`, `Bea`, `Chris` (retired); devices `D1` (air pistol, `.177`, = `DEV-REF`), `D2` (air rifle, `.177`), `D3` (pistol, `.22 LR`), `D4` (air pistol, `.177`, retired); target types `TT-REF`, `TT-2` (copy of `TT-REF`), `TT-3` (copy, retired).

| Session | Shooter | Device | Target type | `distance_m` | `single_handed` | `supported` | `shot_count` | `score` | `saved_at` (UTC) |
|---|---|---|---|---|---|---|---|---|---|
| F1 | Alex | D1 | TT-REF | 10 | true | false | 5 | 40 | 2026-01-10 10:00 |
| F2 | Alex | D2 | TT-REF | 10 | false | true | 10 | 95 | 2026-02-10 10:00 |
| F3 | Bea | D1 | TT-REF | 10 | true | false | 20 | 180 | 2026-03-10 10:00 |
| F4 | Bea | D3 | TT-2 | 25 | false | false | 5 | 30 | 2026-04-10 10:00 |
| F5 | — | D3 | TT-2 | 25 | true | false | 60 | 500 | 2026-05-10 10:00 |
| F6 | Chris | D1 | TT-REF | 10 | true | false | 5 | 45 | 2026-06-10 10:00 |
| F7 | Alex | D4 | TT-REF | 10 | false | false | 5 | 41 | 2026-07-10 10:00 |
| F8 | Alex | D1 | TT-3 | 10 | false | false | 5 | 39 | 2026-08-10 10:00 |

| ID | Case | Filter | Expected sessions |
|---|---|---|---|
| DB-FLT-01 | No restriction | `{}` | F1–F8; `total` 8 |
| DB-FLT-02 | Shooters | `shooter_ids: [Alex]` | F1, F2, F7, F8 |
| DB-FLT-03 | Empty list | `shooter_ids: []` | none |
| DB-FLT-04 | Devices | `device_ids: [D3]` | F4, F5 |
| DB-FLT-05 | Device categories | `device_categories: ["air pistol"]` | F1, F3, F6, F7, F8 |
| DB-FLT-06 | Calibers | `calibers: [".22 LR"]` | F4, F5 |
| DB-FLT-07 | Fields are combined with AND | `device_categories: ["air pistol"], device_ids: [D2]` | none |
| DB-FLT-08 | Target types | `target_type_ids: [TT-2]` | F4, F5 |
| DB-FLT-09 | Distance from | `distance_m: { min: 20 }` | F4, F5 |
| DB-FLT-10 | Distance bounds are inclusive | `distance_m: { min: 10, max: 10 }` | F1, F2, F3, F6, F7, F8 |
| DB-FLT-11 | Time range, inclusive | `saved_at: { from: <F3 saved_at>, to: <F5 saved_at> }` | F3, F4, F5 |
| DB-FLT-12 | Single-handed | `single_handed: true` | F1, F3, F5, F6 |
| DB-FLT-13 | Supported | `supported: true` | F2 |
| DB-FLT-14 | Shot count | `shot_count: { min: 10, max: 20 }` | F2, F3 |
| DB-FLT-15 | Without retired | `include_retired: false` | F1–F5 (F6: retired shooter, F7: retired device, F8: retired target type) |
| DB-FLT-16 | Session ids | `session_ids: [F2, F4]` | F2, F4 |
| DB-FLT-17 | Unknown id | `session_ids: [999999]` | none; no error |
| DB-FLT-18 | Two fields | `shooter_ids: [Alex], supported: true` | F2 |
| DB-FLT-19 | SQL in a value | `device_categories: ["x' OR '1'='1"]` | none (value bound as a parameter) |

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-FLT-20 | Default order | No `order_by` | F8, F7, F6, F5, F4, F3, F2, F1 (`saved_at desc`) |
| DB-FLT-21 | Order by score | `order_by: { field: "score", direction: "asc" }` | F4, F8, F1, F7, F6, F2, F3, F5 |
| DB-FLT-22 | Order by shot count | `shot_count desc` | F5, F3, F2 first |
| DB-FLT-23 | Order by a field that isn't allowed | `order_by: { field: "notes" }` | `validation_failed` |
| DB-FLT-24 | Paging | `order_by id asc`, `limit` 3, `offset` 3 | F4, F5, F6; `total` 8 |
| DB-FLT-25 | Page after the end | `offset` 10 | `[]`; `total` 8 |

## Shooters (integration)
What is tested: `save_shooter`, `read_shooters`, `delete_shooter`.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-SHO-01 | Create | `save_shooter` with name `"  Alex  "` | Id; stored name `Alex`; new `uuid`; `retired` false; event created |
| DB-SHO-02 | Empty name | `""`, then `"   "` | `validation_failed` |
| DB-SHO-03 | Name exists, other case | `alex` while `Alex` exists | `validation_failed` |
| DB-SHO-04 | Rename | `save_shooter` with id and name `Alexandra` | Name changed; sessions keep their `shooter_id`; event updated |
| DB-SHO-05 | Retire | `retired: true` | Stored; still returned by `read_shooters`; sessions unchanged |
| DB-SHO-06 | Unknown id | `save_shooter` with an unknown id | `not_found` |
| DB-SHO-07 | Delete unused | `delete_shooter` | Ok; event deleted |
| DB-SHO-08 | Delete used | `delete_shooter` for a shooter with a session | `in_use` |
| DB-SHO-09 | Filter | `read_shooters` with `name like "Al%"` | `Alex` only |
| DB-SHO-10 | `uuid` in the payload is ignored | `save_shooter` (new) with a `uuid` | The stored `uuid` is a new one |

## Devices and device images (integration)
What is tested: `save_device`, `read_devices`, `delete_device`, `set_device_image`, `delete_device_image`.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-DEV-01 | Create | `save_device` with `DEV-REF` | Id; new `uuid`; `shot_count` 0; `image_path` null; `locked` false |
| DB-DEV-02 | Invalid | Empty name; `bullet_diameter_mm` 0; `bullet_diameter_mm` missing | `validation_failed` |
| DB-DEV-03 | Update an unlocked device | Change name, category, description, caliber, `retired`, `bullet_diameter_mm` | All stored |
| DB-DEV-04 | Read-only fields are ignored | Payload with `shot_count` 999, `image_path` `x`, `locked` true, another `uuid` | None of them applied |
| DB-DEV-05 | Locked device, bullet diameter | Device used by a session; change `bullet_diameter_mm` | `in_use`; nothing changed |
| DB-DEV-06 | Locked device, other fields | Same device; change name and description, `bullet_diameter_mm` unchanged | Ok (the locked column isn't written, so the trigger doesn't fire) |
| DB-DEV-07 | `locked` flag | `read_devices` before and after a session uses the device | `false`, then `true` |
| DB-DEV-08 | Unknown id | `save_device` with an unknown id | `not_found` |
| DB-DEV-09 | Set an image | `set_device_image` with a staged JPEG | `image_path` `devices/<id>_<8 hex>.jpg`; file moved; event updated |
| DB-DEV-10 | Replace an image twice quickly | Two `set_device_image` within one second | Two different file names; the device has the second; the first file is deleted after commit |
| DB-DEV-11 | Image for an unknown device | `set_device_image` with an unknown `device_id` | `not_found`; staged file untouched |
| DB-DEV-12 | Remove the image | `delete_device_image` | `image_path` null; file deleted |
| DB-DEV-13 | Remove when there is none | `delete_device_image` on a device without image | Ok; `image_path` stays null |
| DB-DEV-14 | Delete unused device | `delete_device` | Row and image file deleted; event deleted |
| DB-DEV-15 | Delete used device | `delete_device` for a device with a session | `in_use` |

## Target types (integration)
What is tested: `save_target_type`, `read_target_types`, `delete_target_type`.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-TGT-01 | Create and read | `save_target_type(TT-REF)`; `read_target_types` | Id, new `uuid`; `rings` as in [LIB-TGT-01](shared_libraries.md#ring-generation-and-target-type-validation-unit); `locked` false; `retired` false |
| DB-TGT-02 | `rings` in the payload are ignored | `TT-REF` with 3 made-up rings | Read returns the 12 generated rings |
| DB-TGT-03 | Invalid target types | Each input of [LIB-TGT-07](shared_libraries.md#ring-generation-and-target-type-validation-unit) | `validation_failed` |
| DB-TGT-04 | Name exists, other case | `tt-ref` while `TT-REF` exists | `validation_failed` |
| DB-TGT-05 | Locked target type | Used by a session; change any anchor, center or paper value | `in_use`; nothing changed |
| DB-TGT-06 | Locked target type, free fields | Same; change `name` and `retired` | Ok |
| DB-TGT-07 | Unlocked target type | Change `outer_radius_mm` | Ok; `rings` change accordingly |
| DB-TGT-08 | Delete | Delete an unused type; then a used one | Ok; `in_use` |
| DB-TGT-09 | Unknown id | `save_target_type` with an unknown id | `not_found` |

## Import requests (integration)
What is tested: `import_shooter`, `import_target_type`, `import_device`, `import_session` ([interfaces.md](../interfaces.md#import)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-IMP-01 | New shooter | `import_shooter` with a new uuid | `created` true; the given `uuid` is stored |
| DB-IMP-02 | Shooter uuid exists | Same uuid, other name | `created` false; existing id; nothing changed |
| DB-IMP-03 | Shooter name exists | New uuid, name `ALEX` while `Alex` exists | `created` false; id of `Alex`; no new row |
| DB-IMP-04 | New target type | `import_target_type` with a new uuid | `created` true; `uuid` kept |
| DB-IMP-05 | Target type uuid exists | Same uuid, other geometry | `created` false; existing record unchanged |
| DB-IMP-06 | Target type name exists | New uuid, name `TT-REF`; then another new uuid, same name | Created as `TT-REF (imported)`, then `TT-REF (imported 2)` |
| DB-IMP-07 | New device with image | `import_device` with `shot_count` 123 and a staged image | `created` true; `shot_count` 123; image moved to `devices/<id>_<8 hex>.jpg` |
| DB-IMP-08 | Device uuid exists | Same uuid, other `shot_count`, with a staged image | `created` false; `shot_count`, image and fields unchanged; the staged image is left in the staging folder |
| DB-IMP-09 | New session | `import_session` with `uuid`, `saved_at` 2025-05-01, two images | `created` true; `uuid` and `saved_at` as given; images at `sessions/<uuid>_<n>`; `shot_count` of the device unchanged |
| DB-IMP-10 | Session uuid exists | Same uuid again, with staged images | `created` false; nothing changed; staged images left in the staging folder |
| DB-IMP-11 | Invalid session | The invalid payloads of DB-SES-05 as `import_session` | `validation_failed` |
| DB-IMP-12 | Invalid uuid | `uuid` `not-a-uuid` in any import request | `validation_failed` |
| DB-IMP-13 | Missing `saved_at` | `import_session` without `saved_at` | `validation_failed` |

## Settings (integration)
What is tested: `read_settings`, `save_setting`, `delete_setting`, `reset_settings` ([database.md](../database.md#known-settings)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-SET-01 | Fields of a setting | `read_settings` | Every known setting has `optional`, `default_value`, `default_active` and `value_rule` |
| DB-SET-02 | Valid values | Save one valid value of each kind: folder (`export_dir`), integer (`camera_fps` 1 and 120), choice (`camera_source` `rtsp`), URL (`camera_rtsp_url` `rtsp://192.168.1.5/stream`), address (`web_bind_address` `192.168.1.10:80`), JSON (`camera_controls` `{}`), text (`camera_device`) | Ok; event updated for each |
| DB-SET-03 | Invalid values | `camera_fps` `0`, `121`, `abc`, `30.5`; `camera_source` `usb`; `camera_rtsp_url` `http://x`; `web_bind_address` `8080`, `0.0.0.0:70000`, `localhost:8080`; `camera_controls` `[]`, `{`; `live_feed_quality` `101`; `admin_password_hash` `secret` | Each: `validation_failed`; value unchanged |
| DB-SET-04 | Unsetting a required setting | `camera_fps` with `active` false | `validation_failed` |
| DB-SET-05 | Unsetting an optional setting | `export_dir` with `active` false | Ok; `read_settings` returns `active` false and the value kept |
| DB-SET-06 | `image_dir` through `save_setting` | `save_setting` for `image_dir` | `validation_failed` (message: use `change_image_dir`) |
| DB-SET-07 | Staging folder on another file system | `staging_dir` on the second mount of `TempDataDir` | `validation_failed` |
| DB-SET-08 | Folder can't be written | `export_dir` below a read-only folder | `validation_failed` |
| DB-SET-09 | Folder doesn't exist yet | `export_dir` that doesn't exist but can be created | Ok; the folder exists afterwards |
| DB-SET-10 | Delete a known setting | `delete_setting("camera_fps")` | `validation_failed` |
| DB-SET-11 | Own setting | `save_setting` for `my_setting`; `delete_setting`; delete again | Ok; ok; `not_found` |
| DB-SET-12 | Reset by name | Change `camera_fps` and `export_dir`; `reset_settings` with both names | Default value and default `active` restored; one `data_changed` per setting |
| DB-SET-13 | Reset all | Change several settings, set `admin_password_hash` and `lens_calibrations`, add `my_setting`; `reset_settings` with `names: null` | All known settings back to default, except `admin_password_hash` and `lens_calibrations`; `my_setting` unchanged |
| DB-SET-14 | Reset an unknown name | `reset_settings` with `["nonsense"]` | `validation_failed`; nothing reset |
| DB-SET-15 | Reset the password by name | `reset_settings` with `["admin_password_hash"]` | Inactive, value empty |

## Image folder change (integration)
What is tested: `change_image_dir` and `reset_settings` of `image_dir` ([database_handler.md](../programs/database_handler.md#image-folder-change)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-DIR-01 | Without moving | 10 images; `change_image_dir` with `move_images` false | `images: null`; setting changed; the old files stay; new images go to the new folder |
| DB-DIR-02 | With moving | 50 images; `move_images` true | `moved` 50, `failed` []; all files in the new folder with the same relative paths; none left in the old folder; `image_path` values unchanged |
| DB-DIR-03 | Saving during the copy | `db.image_dir_copy` slows the copy to 10 s; meanwhile `create_session` with an image | `create_session` answered within 5 s; after the change its image is in the new folder, not in the old |
| DB-DIR-04 | Deleting during the copy | Same; meanwhile `delete_session` of a session whose image was already copied | After the change the image is in neither folder |
| DB-DIR-05 | File that can't be read | One image with mode `000` | Its `image_path` in `failed`; it stays in the old folder; the others are moved |
| DB-DIR-06 | Second change while one runs | Start a slowed change; send another `change_image_dir` | `invalid_state` |
| DB-DIR-07 | Other file system than the staging folder | New folder on the second mount | `validation_failed`; nothing changed |
| DB-DIR-08 | Folder can't be written | New folder below a read-only folder | `validation_failed` |
| DB-DIR-09 | Crash during the copy | Kill the handler during the copy; check; restart | After the kill: setting still the old folder, every image readable there. After the restart the unacknowledged `change_image_dir` runs again ([request processing](../programs/database_handler.md#request-processing)) and completes: setting the new folder, every image complete in it, none left in the old folder; the partial copies of the first run do no harm |
| DB-DIR-10 | Reset with moving | `reset_settings` with `["image_dir"]`, `move_images` true | Like DB-DIR-02, to `<data_dir>/images` |

## Change events (integration)
What is tested: `data_changed` after every write, and only then ([interfaces.md](../interfaces.md#data_changed)).

| ID | Case | Steps / input | Expected events |
|---|---|---|---|
| DB-EVT-01 | Session created | `create_session` | `sessions/<id>/created` and `devices/<device id>/updated` |
| DB-EVT-02 | Session deleted | `delete_session` | `sessions/<id>/deleted` |
| DB-EVT-03 | Shooters | create, update, delete | `created`, `updated`, `deleted` with table `shooters` |
| DB-EVT-04 | Devices | create, update, `set_device_image`, `delete_device_image`, delete | `created`, `updated`, `updated`, `updated`, `deleted` |
| DB-EVT-05 | Target types | create, update, delete | `created`, `updated`, `deleted` |
| DB-EVT-06 | Settings | Change a known setting; create and delete an own setting | `updated`; `created`, `deleted` (key = name) |
| DB-EVT-07 | Reset | `reset_settings` with 3 names | 3 × `settings/<name>/updated` |
| DB-EVT-08 | Image folder change | `change_image_dir` | `settings/image_dir/updated` |
| DB-EVT-09 | Imports | Each import request once new, once existing | `created` events for new records only; none for existing ones; `import_session` doesn't send `devices/updated` |
| DB-EVT-10 | No event without a write | Every read request; every failed write | No event |

## Duplicate protection (integration)
What is tested: every write is done once, even if the request comes twice or the handler crashes ([interfaces.md](../interfaces.md#duplicate-protection-idempotency)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-IDM-01 | Same request twice | Every write request type (parametrized), sent twice with the same `request_id` | Second response identical to the first; effect only once (rows, files, `shot_count`); one `processed_requests` row with type, `processed_at` and the response |
| DB-IDM-02 | Failed requests aren't recorded | `create_session` fails at `db.before_commit`; the same request is sent again with the same `request_id` | Second attempt succeeds; one session |
| DB-IDM-03 | Crash after commit | `db.after_commit` aborts the process during `create_session`; restart | The pending request is processed again: the stored response is sent once; exactly one session; after reconnecting, `data_changed` `all`/`reload` is sent |
| DB-IDM-04 | Duplicate after the image moved | DB-IDM-01 for `create_session` with an image (the staged file is gone at the second request) | Second response identical, no error |

## Maintenance (integration)
What is tested: hourly clean-up ([database_handler.md](../programs/database_handler.md#startup)). The handler's DbWorker and maintenance loop run in the test process (library functions `spawn_db_worker`, `run_periodically`), so `FakeClock` can drive them.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-MNT-01 | Purge processed requests | Rows 29 and 31 days old; start | The 31-day-old row deleted, the 29-day-old kept |
| DB-MNT-02 | Retention setting | `processed_requests_retention_days` 1; advance the clock 1 h | Rows older than 1 day deleted at the next run |
| DB-MNT-03 | Staging folder | Files (incl. `.part`) 23 h and 25 h old | Files older than 24 h deleted; the others kept; folders untouched |
| DB-MNT-04 | Runs every hour | Advance the clock by 1 h several times | One run per hour |

## Error handling (integration)
What is tested: behavior in failure situations ([database_handler.md](../programs/database_handler.md#error-handling)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-ERR-01 | Redis lost | Stop Redis while 3 requests are queued; start it again | The handler reconnects, processes the queued requests, and sends `data_changed` `all`/`reload` |
| DB-ERR-02 | Disk full | Image folder on a 1 MB `tmpfs`; `create_session` with a 2 MB image | `io_error`; request acknowledged; no rows; staged file intact |
| DB-ERR-03 | Database locked | Another process holds an exclusive lock for 6 s; send `create_session` | `database_error` after about 5 s (`busy_timeout`); acknowledged; works again after the lock is released |
| DB-ERR-04 | Many writes at once | 50 `create_session` at the same time | All succeed; 50 sessions; `shot_count` = sum of all shots |
| DB-ERR-05 | Shutdown during a request | `SIGTERM` while a request is processed | The request is finished and answered, then the program exits |

## Performance (performance)
What is tested: response times with a large database. *(goals set here)*

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| DB-PRF-01 | History page | 10 000 sessions with 60 shots each; `read_sessions` with `limit` 50, `include_shots` false | Answer within 200 ms |
| DB-PRF-02 | Full read | Same database; all sessions with shots, in pages of 500 | Each page within 1 s |
| DB-PRF-03 | Save | `create_session` with 60 shots and 2 images | Answer within 200 ms |
