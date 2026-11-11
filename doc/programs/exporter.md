# exporter

Program `exporter` (Rust): exports sessions as CSV, JSON or PNG target plot into the export folder; CSV and JSON optionally together with the images as one ZIP file. Requests: [interfaces.md](../interfaces.md#exporter).

## Responsibilities
- Reads the selected sessions with their shots, shooters, devices and target types from `database_handler`.
- Gets the score of each shot from `analyzer`.
- Writes the export files into the export folder (setting `export_dir`); with `include_images`, packs the data files and the images into one ZIP.
- Lists and deletes export files.

`web_server` reads the export folder to offer the files for download; only the exporter writes and deletes files there. The exporter reads the image folder (setting `image_dir`) read-only, to copy images into ZIP files.

## Technology
| Purpose | Crate |
|---|---|
| Redis, model | `targetscope_common` |
| CSV | `csv` |
| JSON | `serde_json` |
| PNG plot | `tiny-skia` (drawing), `ab_glyph` (text), embedded font (DejaVu Sans) |
| Time formatting | `time` |
| ZIP | `zip` (deflate for data files, stored without compression for JPEG/PNG) |

## Export flow (`export_sessions`)
1. `read_sessions` with the request's SessionFilter (`include_shots = true`), in pages of 500 ordered by `id`. No match: `not_found`.
2. `read_shooters`, `read_devices` and `read_target_types` for the ids used by the sessions.
3. For each session: `analyze_shots` to get the score of each shot (CSV, JSON, PNG) and the grouping shift (CSV, PNG). Because target type geometry and bullet diameter are locked once used, the result always matches the stored scores.
4. Write the files to a temporary name (`.<name>.part`) in the export folder, then rename (leftover `.part` files are deleted at startup), so the download never sees half-written files. With `include_images`, the data files and images are written directly into one ZIP (see below).
5. Respond with the file names.

File names start with the time of the export and the first 8 characters of the `request_id`: `<yyyy-mm-dd_HHMMSS>_<id8>_...`. A repeated request with the same `request_id` would produce a new time, so the exporter first looks for existing files with the same `<id8>` and returns them instead (duplicate protection).

## Formats

### CSV
Two files, semicolon separated, UTF-8 with header row, decimal point `.`:
- `<prefix>_sessions.csv`: one row per session: `session_id;session_uuid;saved_at;shooter;device;device_category;caliber;target_type;distance_m;single_handed;supported;shot_count;paper_count;duration_s;score;grouping_score;grouping_shift_x_mm;grouping_shift_y_mm;notes`
- `<prefix>_shots.csv`: one row per shot: `session_id;shot_no;paper_no;angle_turns;radius_mm;x_mm;y_mm;score`

Text fields (e.g. `notes`) are quoted as CSV requires, so semicolons and line breaks in notes are kept.

`saved_at` is written as ISO 8601 in local time (`2026-11-11T14:30:00+01:00`).

### JSON
One file `<prefix>.json`, complete and self-contained. It can be imported again by [importer](importer.md), also on another installation. The format is defined by `schemas/export/v1.schema.json`, which exporter and importer share.
```json
{
  "format": "targetscope-export",
  "format_version": 1,
  "exported_at": "2026-11-11T14:30:00+01:00",
  "shooters":     [ { ...Shooter without id } ],
  "target_types": [ { ...TargetType without id, rings, locked } ],
  "devices":      [ { ...Device without id, image_path, locked } ],
  "sessions":     [ { ...Session without id, shooter_id, device_id, target_type_id, images,
                      "shooter_uuid": "…" or null, "device_uuid": "…", "target_type_uuid": "…",
                      "shot_scores": [10, 9, ...] } ]
}
```
- Records reference each other only by `uuid`, because ids differ between installations.
- Only the shooters, target types and devices used by the exported sessions are included.
- `shot_scores` is informational; the importer ignores it.
- Without `include_images`, the JSON file contains no images and no `image_entry` fields.

### PNG target plot
One file per session: `<prefix>_session_<id>.png`, the target paper to scale (long side 1600 px) plus a header area:
- paper outline, black area and the generated rings of the target type, ring scores labeled,
- every shot as a circle with the bullet diameter, labelled with its number (`<paper_no>.<n>` for sessions with several papers),
- mean point of impact as a cross, and the grouping shift as an arrow from the target center,
- header text: saved_at, shooter, device, distance, target type, score, grouping score, shot count, paper count.

### ZIP with images
With `include_images` (only `csv` and `json`), the export is one file `<prefix>_csv.zip` or `<prefix>_json.zip`:
```
<prefix>_json.zip                          <prefix>_csv.zip
├── export.json                                  ├── sessions.csv
└── images/                                      ├── shots.csv
    ├── sessions/<session uuid>_<paper_no>.<ext> └── images/
    └── devices/<device uuid>.<ext>                  ├── sessions/<session uuid>_<paper_no>.<ext>
                                                     └── devices/<device uuid>.<ext>
```
- Images are named by the uuid of their record, so they stay unique across installations.
- The data files reference the images by their path inside the ZIP:
  - JSON: `images: [ { "paper_no", "image_entry" } ]` on sessions (e.g. `images/sessions/7c9e…_1.jpg`) and an optional `image_entry` on devices, defined in the export schema,
  - CSV: two extra columns in `sessions.csv`: `image_entries` (the session's target images, separated by `|`, in paper order) and `device_image_entry`; empty if there is none.
- Only images of the exported sessions and their devices are included. A missing image file (e.g. deleted by hand) is skipped and logged; the data still references nothing for it.
- A `<prefix>_json.zip` can be imported by [importer](importer.md), which restores the images. A `<prefix>_csv.zip` is for other software (spreadsheets) and can't be imported.

## list_exports and delete_export
`list_exports` lists the files in the export folder (without `.part` files), newest first. `delete_export` accepts only a plain file name (no path separators, no `..`) that exists in the folder.

## Error handling
| Situation | Behavior |
|---|---|
| No matching sessions | `not_found` |
| `export_dir` not set (`active = false`) | `setting_not_set` with `details.setting = "export_dir"`; the UI asks for the folder and retries |
| Export folder not writable / disk full | `io_error`; partial `.part` files are deleted |
| database_handler or analyzer not answering | `timeout`; nothing is written |

## Testing
All test cases with their expected outcomes: [testing/exporter.md](../testing/exporter.md).
