# testdata

Reference data for all tests (doc/testing/overview.md#reference-data).

```
testdata/
├── target_types/   # target types as JSON, with their expected rings and validity (also schema examples)
├── sessions/       # session sets for database, exporter and statistics tests, with expected results
├── images/         # small files for staging tests: paper.jpg, device.png, not_an_image.jpg, large_20mb.jpg
├── photos/         # photos of shot targets, with ground truth
├── videos/         # recorded camera videos, with ground truth
├── synthetic/      # renderer for targets with known holes, perspective, light and lens distortion
├── lens/           # checkerboard views with known distortion
├── imports/        # valid and invalid import files (incl. ZIP slip, ZIP bomb)
└── exports/        # golden export files
```

Every photo, video and synthetic render has a `<name>.truth.json` (format in doc/testing/overview.md#ground-truth-for-photos-videos-and-renders). Positions are in paper millimeters, origin top left.

Large binary files (photos, videos) should be stored with Git LFS.
