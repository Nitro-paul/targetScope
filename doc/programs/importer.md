# importer

Program `importer` (Rust): imports JSON export files and ZIP files of JSON exports with images ([exporter](exporter.md#json)) into the database, also from another TargetScope installation. Request: [interfaces.md](../interfaces.md#importer).

## Responsibilities
- Reads an uploaded export file (`.json`, or `.zip` with `export.json` and images) from the staging folder.
- Restores session target images and device images from a ZIP, by passing them to `database_handler` with the records.
- Validates it completely against `schemas/export/v1.schema.json` and the consistency rules below, before anything is written.
- Imports shooters, target types, devices and sessions through `database_handler`, keeping their `uuid`, so importing the same file twice changes nothing.
- Reports what was created, what existed already, and warnings.
- Deletes the uploaded file afterwards.

Imported sessions are records like any other: they are inserted unchanged (original `saved_at`, shots, scores) and never merged with or written over existing sessions.

## Technology
| Purpose | Crate |
|---|---|
| Redis, model | `targetscope_common` |
| JSON | `serde_json` |
| Schema validation | `jsonschema` |
| ZIP | `zip` |

## Import flow (`import_file`)
1. Check `staged_file` (plain name, exists in the staging folder; `.json` at most 100 MB, `.zip` at most 2 GB).
   - `.json`: parse it.
   - `.zip`: check every entry name first: only `export.json` and files matching `images/sessions/<uuid>_<paper_no>.<jpg|png>` or `images/devices/<uuid>.<jpg|png>` are allowed; no absolute paths, no `..`, no links (protection against "zip slip"). Uncompressed size at most 4 GB. Then parse `export.json`. On any violation: `validation_failed`, nothing is imported.
2. Validate against the export schema. `format` must be `targetscope-export`; `format_version` must be supported (currently 1). On failure: `validation_failed` with the first problem; nothing is imported.
3. Consistency checks:
   - every `shooter_uuid`, `device_uuid` and `target_type_uuid` of a session exists in the file,
   - uuids are unique within the file,
   - every target type passes the TargetType validation, every session the Session validation (at least one shot, `shot_count` = number of shots, shot ranges, `paper_no` within `1 ..= paper_count`),
   - ZIP: every `image_entry` in `export.json` exists in the ZIP and is a JPEG or PNG (by magic bytes). An image that is missing in the ZIP is only a warning; the record is imported without it.
4. **Shooters:** `import_shooter` for each (an existing shooter with the same name is reused). Keep a map `uuid → local id`.
5. **Target types:** `import_target_type` for each. Keep a map `uuid → local id`.
6. **Devices:** `import_device` for each. Keep a map `uuid → local id`.
7. **Sessions:** for each, replace `shooter_uuid`, `device_uuid` and `target_type_uuid` by the local ids, then `import_session`. Existing uuids are skipped.
8. Delete the uploaded file and all extracted images that are left in the staging folder; respond with the counts and warnings.

**Images (ZIP only):** before each `import_device` and `import_session` with image entries, the images are extracted from the ZIP into the staging folder (`import_<uuid>.<ext>`) and their names are passed as `staged_file` (devices) or in `images` (sessions). `database_handler` moves it into the image folder only if it creates the record; for records that already exist the image is ignored, and the importer deletes the extracted file. Images are extracted one at a time, so the staging folder never holds the whole ZIP content.

Each record is its own request; there is no transaction around the whole file. If the import is interrupted (crash, timeout), running it again with the same file is safe: records that are already imported are found by their `uuid` and skipped.

## Conflicts
| Case | Result |
|---|---|
| Session uuid exists | Skipped (counted as `existing`) |
| Shooter, device or target type uuid exists | The existing record is used unchanged (counted as `existing`) |
| Shooter uuid is new, but the name exists | The existing shooter is used (same person); counted as `existing` |
| Target type uuid is new, but the name exists | Created with the name suffix ` (imported)` (or ` (imported 2)`, …); warning |
| Existing target type with the same uuid but different geometry | Can't happen through TargetScope (geometry is locked once used, and the uuid never changes); if it does, the existing record is used and a warning is added |

Importing never changes `devices.shot_count` of existing devices. A device created by the import takes `shot_count` from the file.

## Error handling
| Situation | Behavior |
|---|---|
| File isn't valid JSON or doesn't match the schema | `validation_failed`, nothing imported |
| database_handler doesn't answer during the import | `timeout`; already imported records stay; a second import completes the rest |
| File not found | `not_found` |

## Testing
- Import of an export of a test database into an empty database: all records equal, including uuids and dates.
- Importing the same file twice: second run creates nothing.
- Interrupted import (fake database handler fails after n records), then a second run: everything imported once.
- Invalid files: wrong format, unknown version, missing references, duplicate uuids.
- ZIP: images are restored for created records and ignored for existing ones; entries with `..`, absolute paths, links or unexpected names reject the whole file; a missing image gives a warning; no extracted files are left in the staging folder.
