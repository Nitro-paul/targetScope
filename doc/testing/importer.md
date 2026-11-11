# Tests: importer

Tests of the program `importer`.

Design: [importer.md](../programs/importer.md), [interfaces.md](../interfaces.md#importer). Reference data and tools: [overview.md](overview.md).

## Setup
- **Integration tests** run `importer` with a real `database_handler` on `RedisTestServer` and `TempDataDir`. Import files are copied from `testdata/imports/` into the staging folder before each case.
- `valid_full.json` and `valid_full.zip` are exports of `EXPORT-SET` ([exporter tests](exporter.md#setup)); the ZIP contains the images.
- After every case the test also checks that nothing was written outside the staging, image and database folders.

## Valid imports (integration)
What is tested: a complete import, repeated imports and conflicts ([importer.md](../programs/importer.md#import-flow-import_file)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IM-OK-01 | Into an empty database | `import_file` with `valid_full.json` | Counts: shooters 1 created, target types 1, devices 2, sessions 3; every record equal to the export source (uuids, `saved_at`, all fields, shots with `paper_no`, `notes`, `retired`) |
| IM-OK-02 | The same file again | `import_file` again | Every count `existing`, none `created`; database unchanged; no `data_changed` |
| IM-OK-03 | ZIP with images | `valid_full.zip` into an empty database | `images.restored` 4; session images at `sessions/<uuid>_<n>.jpg`, the device image at `devices/<id>_<8 hex>.jpg`, bytes identical |
| IM-OK-04 | Clean-up | After IM-OK-01 and IM-OK-03 | The uploaded file is deleted; no `import_*` files left in the staging folder |
| IM-OK-05 | Shot counts | After IM-OK-01 | Created devices have `shot_count` from the file; imported sessions added nothing |
| IM-OK-06 | Order | Record the requests `database_handler` receives | All `import_shooter`, then all `import_target_type`, then all `import_device`, then all `import_session` |
| IM-OK-07 | Existing records | Database already has shooter `alex`, target type `TT-REF` (other uuid) and the device with the file's uuid (different `shot_count`) | Shooter `alex` reused (`existing`); target type created as `TT-REF (imported)` with a warning; device unchanged, including `shot_count` and image |
| IM-OK-08 | Existing session with images in the ZIP | Import the ZIP twice | Second time: sessions `existing`; their images not moved; extracted files deleted |

## Invalid files (integration)
What is tested: validation before anything is written ([importer.md](../programs/importer.md#import-flow-import_file)). Every case expects `validation_failed` with a message naming the first problem, no record created, and the uploaded file deleted.

| ID | Case | Input |
|---|---|---|
| IM-VAL-01 | Not JSON | `not_json.json` |
| IM-VAL-02 | Wrong format name | `format` `other-export` |
| IM-VAL-03 | Unknown version | `format_version` 2 |
| IM-VAL-04 | Missing field | Session without `uuid` (message names the field) |
| IM-VAL-05 | Unknown reference | Session with a `device_uuid` that isn't in the file |
| IM-VAL-06 | Duplicate uuid | Two sessions with the same `uuid` |
| IM-VAL-07 | Session without shots | `shots` [] |
| IM-VAL-08 | Paper number too high | Shot with `paper_no` 3, `paper_count` 2 |
| IM-VAL-09 | Invalid target type | `black_radius_mm` > `outer_radius_mm` |
| IM-VAL-10 | Invalid shot | `angle_turns` 1.0 |
| IM-VAL-11 | JSON too large | 100 MB + 1 byte |

## ZIP files (integration)
What is tested: protection against harmful ZIP files and image checks ([importer.md](../programs/importer.md#import-flow-import_file)).

| ID | Case | Input | Expected outcome |
|---|---|---|---|
| IM-ZIP-01 | Path out of the folder | Entry `../evil.jpg` | `validation_failed`; nothing imported; no file written outside the staging folder |
| IM-ZIP-02 | Absolute path | Entry `/tmp/evil.jpg` | `validation_failed` |
| IM-ZIP-03 | Link | Symbolic link entry | `validation_failed` |
| IM-ZIP-04 | Unexpected entry | `notes.txt`; `images/sessions/x.txt` | `validation_failed` |
| IM-ZIP-05 | No `export.json` | ZIP with images only | `validation_failed` |
| IM-ZIP-06 | Not an image | `images/sessions/<uuid>_1.jpg` with text content | `validation_failed` |
| IM-ZIP-07 | ZIP bomb | 1 MB ZIP that unpacks to more than 4 GB | `validation_failed` before anything is unpacked |
| IM-ZIP-08 | ZIP too large | 2 GB + 1 byte | `validation_failed` |
| IM-ZIP-09 | Referenced image missing | `export.json` references an image the ZIP doesn't contain | Warning; the session imported without that image |

## Interrupted import and errors (integration)
What is tested: running again after an interruption, missing files.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IM-INT-01 | Interrupted | `database_handler` stops answering after 5 records (`importer.after_record` counts) | `timeout`; the 5 records stay |
| IM-INT-02 | Run again | Same file again after IM-INT-01 | The rest is imported; every record exists exactly once |
| IM-ERR-01 | File missing | `staged_file` doesn't exist | `not_found` |
| IM-ERR-02 | Name with a path | `staged_file` `../valid_full.json` | `validation_failed` |
