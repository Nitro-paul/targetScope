# Tests: exporter

Tests of the program `exporter`.

Design: [exporter.md](../programs/exporter.md), [interfaces.md](../interfaces.md#exporter). Reference data and tools: [overview.md](overview.md).

## Setup
- **Integration tests** run `exporter` with a real `database_handler` and a real `analyzer` on `RedisTestServer`, with `TempDataDir` and `FakeClock`.
- **Fixture `EXPORT-SET`** (`testdata/sessions/export_set.json`):
  - E1: `SH-REF`, `DEV-REF` (with a device image), `TT-REF`, one paper with image, shots `S-REF`, `notes` `line1;` + line break + `"quoted"`, `saved_at` 2026-11-11 13:30:00 UTC;
  - E2: no shooter, `DEV-REF`, `TT-REF`, two papers, both with images, 4 shots (2 per paper);
  - E3: `SH-REF`, device `D2` (no image), `TT-REF`, manual, no images;
  - an unused device `D9` and an unused target type `TT-9`.
- The clock stands at 2026-11-11 14:30:00 in `Europe/Berlin` (+01:00); `request_id` `7c9e6679-7425-40de-944b-e07fc1f90ae7`. File prefix: `2026-11-11_143000_7c9e6679`.
- Golden files are in `testdata/exports/`.

## CSV (integration)
What is tested: the CSV format ([exporter.md](../programs/exporter.md#csv)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| EX-CSV-01 | Files | `export_sessions`, all of `EXPORT-SET`, `csv` | Response `files`: `<prefix>_sessions.csv`, `<prefix>_shots.csv`; both in the export folder |
| EX-CSV-02 | Session columns | First line of `_sessions.csv` | Exactly `session_id;session_uuid;saved_at;shooter;device;device_category;caliber;target_type;distance_m;single_handed;supported;shot_count;paper_count;duration_s;score;grouping_score;grouping_shift_x_mm;grouping_shift_y_mm;notes` |
| EX-CSV-03 | Session values | Compare with the golden file | Equal; `grouping_shift` from the analyzer |
| EX-CSV-04 | Time format | `saved_at` of E1 | `2026-11-11T14:30:00+01:00` |
| EX-CSV-05 | Decimal point | Run with `LC_ALL=de_DE.UTF-8` | Decimal point `.`, not `,` |
| EX-CSV-06 | Text with separators | `notes` of E1, read back with a CSV parser | Quoted; semicolon, quote and line break kept exactly |
| EX-CSV-07 | Shot rows | `_shots.csv` | One row per shot: `session_id;shot_no;paper_no;angle_turns;radius_mm;x_mm;y_mm;score`; for E1: scores 12, 11, 9, 6, 0; x, y as in [S-REF](overview.md#other-reference-records) |
| EX-CSV-08 | No shooter | Row of E2 | Empty `shooter` field |

## JSON (integration)
What is tested: the export format that can be imported again ([exporter.md](../programs/exporter.md#json)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| EX-JSN-01 | Valid file | `json`; validate `<prefix>.json` against `schemas/export/v1.schema.json` | Valid; `format` `targetscope-export`, `format_version` 1, `exported_at` `2026-11-11T14:30:00+01:00` |
| EX-JSN-02 | Only used records | Contents | Shooters, devices and target types of E1–E3 only; not `D9`, not `TT-9` |
| EX-JSN-03 | References by uuid | Sessions | `shooter_uuid` (null for E2), `device_uuid`, `target_type_uuid`; no ids; target types without `rings`/`locked`; devices without `image_path`/`locked` |
| EX-JSN-04 | Without images | Sessions and devices | No `images` entries and no `image_entry` fields |
| EX-JSN-05 | Shot scores | `shot_scores` of E1 | [12, 11, 9, 6, 0] |
| EX-JSN-06 | Golden file | Compare | Equal (apart from `exported_at`, fixed by the clock) |

## PNG target plot (integration)
What is tested: the plot per session ([exporter.md](../programs/exporter.md#png-target-plot)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| EX-PNG-01 | One file per session | `png` | `<prefix>_session_<id>.png` for E1, E2, E3 |
| EX-PNG-02 | Size | Image size for `TT-REF` (200 × 200 mm) | Paper area 1600 × 1600 px plus the header area above it |
| EX-PNG-03 | Content | Pixel probes | Ring lines at the 12 ring radii; shot circles with the bullet diameter at the shot positions; a cross at the mean point; an arrow from the center along the grouping shift |
| EX-PNG-04 | Labels with several papers | Plot of E2 | Shots labelled `1.1`, `1.2`, `2.1`, `2.2` |
| EX-PNG-05 | Compared with the golden image | Perceptual difference | At most 1 % of pixels differ |

## ZIP with images (integration)
What is tested: exports with `include_images` ([exporter.md](../programs/exporter.md#zip-with-images)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| EX-ZIP-01 | JSON with images | `json`, `include_images` true | One file `<prefix>_json.zip` with exactly: `export.json`, `images/sessions/<E1 uuid>_1.jpg`, `images/sessions/<E2 uuid>_1.jpg`, `images/sessions/<E2 uuid>_2.jpg`, `images/devices/<DEV-REF uuid>.jpg` |
| EX-ZIP-02 | CSV with images | `csv`, `include_images` true | `<prefix>_csv.zip` with `sessions.csv`, `shots.csv` and the same images; extra columns `image_entries` (E2: `images/sessions/<uuid>_1.jpg\|images/sessions/<uuid>_2.jpg`) and `device_image_entry` |
| EX-ZIP-03 | Entries match the files | Compare each referenced entry with the image folder | Every `image_entry` exists in the ZIP; bytes identical |
| EX-ZIP-04 | Compression | Inspect the ZIP entries | Data files deflated; images stored without compression |
| EX-ZIP-05 | Image missing on disk | Delete E1's image file by hand; export | Entry skipped; warning logged; E1 references no image |
| EX-ZIP-06 | PNG with images | `png`, `include_images` true | `validation_failed`; nothing written |

## Flow, files and errors (integration)
What is tested: reading, writing, duplicate protection, export list ([exporter.md](../programs/exporter.md#export-flow-export_sessions)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| EX-FLW-01 | Filter and paging | 1 200 sessions match | `read_sessions` with the request's SessionFilter, pages of 500 by `id`; all 1 200 exported |
| EX-FLW-02 | No match | Filter matches nothing | `not_found`; no files |
| EX-FLW-03 | Half-written files are hidden | `exporter.before_rename` waits; `list_exports` meanwhile | Only `.part` file on disk; `list_exports` doesn't show it; after the rename it does |
| EX-FLW-04 | Leftovers at startup | A `.part` file in the export folder; start | Deleted |
| EX-FLW-05 | Same request twice | `export_sessions` twice with the same `request_id`, the clock 1 min later | Same file names; no new files |
| EX-FLW-06 | Export folder not set | `export_dir` inactive; `export_sessions`, then `list_exports` | Both `setting_not_set` with `details.setting` `export_dir` |
| EX-FLW-07 | Disk full | Export folder on a small `tmpfs` | `io_error`; no `.part` file left |
| EX-FLW-08 | Analyzer silent | `analyze_shots` doesn't answer | `timeout`; nothing written |
| EX-LST-01 | Export list | 3 exports at different times | Newest first, with `size_bytes` and `created_ms`; no `.part` files |
| EX-LST-02 | Delete an export | `delete_export` with an existing name; with `../x`; with a missing name | Deleted; `validation_failed`; `not_found` |
