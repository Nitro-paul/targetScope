# Tests: image_processor

Tests of the program `image_processor` (C++), including the detection accuracy.

Design: [image_processor.md](../programs/image_processor.md), [interfaces.md](../interfaces.md#image_processor), [frame_transport.md](../frame_transport.md#live-frames). Reference data and tools: [overview.md](overview.md).

## Setup
- **Unit tests** (GoogleTest) for coordinates, corner ordering and image functions.
- **Integration tests** run `image_processor` as a process with `RedisTestServer`, a `FakeProgram` as `database_handler` (settings) and as `camera_reader` (`camera_status`, `camera_state_changed`), `FrameFeeder` for raw frames, a test subscriber on `targetscope/frames/live`, and `EventProbe` on `image_processor:events`.
- **Synthetic renders** (`testdata/synthetic/`) draw `TT-REF` on a dark background with holes at known positions, with a chosen camera position (frontal, 30°, 45°), light, noise, hand occlusion and lens distortion. Renders and videos have ground truth ([overview](overview.md#ground-truth-for-photos-videos-and-renders)).
- Live tests use `TT-REF` and `DEV-REF` (bullet 4.0 mm) unless a case says otherwise.

## Coordinates (unit)
What is tested: homography and conversion from paper to shot coordinates ([image_processor.md](../programs/image_processor.md#coordinate-systems)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IP-GEO-01 | Homography from corners | Four corners of a known quadrilateral, paper 200 × 200 mm, `px_per_mm` 6 | The corners map to (0, 0), (1200, 0), (1200, 1200), (0, 1200) within 1e-6 px |
| IP-GEO-02 | Paper to shot | `TT-REF` holes at paper (110, 100), (100, 90), (100, 100), (90, 100) | (`angle_turns`, `radius_mm`) = (0.25, 10), (0, 10), (0, 0), (0.75, 10) |
| IP-GEO-03 | Corner order | The same four points in all 24 orders | Always ordered top-left, top-right, bottom-right, bottom-left |
| IP-GEO-04 | Combined map | Lens correction and homography as one map, compared with applying them one after the other | Pixel values within 1 gray level; point positions within 0.1 px |

## Finding the paper (integration)
What is tested: automatic paper detection ([image_processor.md](../programs/image_processor.md#finding-the-paper)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IP-PAP-01 | Frontal | Render 1920 × 1080, paper fills 60 % of the image | Corners within 1 px of the truth |
| IP-PAP-02 | At an angle | Renders at 30° and 45° | Corners within 2 px |
| IP-PAP-03 | Paper too small | Paper covers 8 % of the image | Not found |
| IP-PAP-04 | Two papers | Two papers of different size | The larger one |
| IP-PAP-05 | Wrong aspect ratio | Live mode with a target type of 200 × 140 mm, render of a 200 × 200 mm paper | Not found |
| IP-PAP-06 | Rotated paper | Renders rotated by 90°, 180°, 270° | Corners reordered so that the black area lies at (`center_x_mm`, `center_y_mm`) within 3 mm |
| IP-PAP-07 | No paper | Image without paper | `find_paper` answers `corners: null`; `detect_shots_in_photo` answers `processing_failed` |
| IP-PAP-08 | Photos | Every photo in `testdata/photos/` | Corners within 1 % of the image diagonal of the truth |

## Live mode: start and calibration (integration)
What is tested: `live_start`, automatic and manual calibration, supervision ([image_processor.md](../programs/image_processor.md#start-and-calibration)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IP-LIV-01 | Start without frames | `live_start`, no frames | `CalibrationStatus` `not_calibrated`, message `no camera frames`; nothing on `targetscope/frames/live` |
| IP-LIV-02 | Automatic calibration | Then frontal frames at 30 fps | `calibration_changed` `calibrated`, method `auto`, within 1.5 s; live frames published |
| IP-LIV-03 | Start again | `live_start` with another target type while active | Holes forgotten; `live_paper` 1; calibrated again for the new type |
| IP-LIV-04 | Manual calibration outside live mode | `live_calibrate` without live mode | `invalid_state` |
| IP-LIV-05 | Manual calibration | `live_calibrate` with the true corners | `calibrated`, method `manual`; `corners` in the status equal the given ones |
| IP-LIV-06 | Manual calibration with lens correction | Distorted render, lens calibration applied; corners clicked in the uncorrected raw image | Hole positions within 0.5 mm of the truth; status `corners` equal the given raw corners |
| IP-LIV-07 | Status outside live mode | `live_calibration_status` without live mode | `not_calibrated`, message `live mode not active` |
| IP-LIV-08 | Paper moves, automatic | Video: paper moves by 5 mm after 3 holes | `calibration_changed` (updated); the 3 holes keep their positions; no `shot_detected` |
| IP-LIV-09 | Paper moves, manual | Same, manually calibrated | State `lost`; no holes detected while `lost` |
| IP-LIV-10 | Small movement | Paper moves by 1.5 mm | No change |
| IP-LIV-11 | Paper taken off and put back | Same paper back after 5 s; meanwhile a hole was made | `lost`, then `calibrated`; the new hole is added as not counted |

## Hole detection (integration)
What is tested: holes present at the start, new holes, counting, filters ([image_processor.md](../programs/image_processor.md#new-holes)). Uses synthetic videos.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IP-HOL-01 | Holes already there | Paper with 5 holes at the start | 5 holes, not counted, gray in the live view; no `shot_detected` |
| IP-HOL-02 | Counting off | New hole while counting is off | Drawn gray; no event |
| IP-HOL-03 | Counting without calibration | `live_set_counting(true)` while not calibrated | `unavailable` |
| IP-HOL-04 | New shot | Counting on; a hole appears and stays | After 5 frames exactly one `shot_detected` with `hole_id`, `live_paper` 1, position within 1 mm of the truth, `confidence` > 0.5; highlighted with number 1 |
| IP-HOL-05 | Short artifact | A hole-like spot visible for 3 frames only | No event; candidate dropped 3 frames after it disappeared |
| IP-HOL-06 | Confirmation setting | `detection_confirm_frames` 10 | Event after 10 frames, not after 5 |
| IP-HOL-07 | Hand in front of the paper | A hand covers 20 % of the paper for 2 s | Frames skipped; no false holes; detection continues afterwards |
| IP-HOL-08 | Shadow | A shadow covers 6 % of the paper | Frames skipped; no false holes |
| IP-HOL-09 | Size filter | Spots of 0.5× and 2.5× the bullet diameter | Ignored |
| IP-HOL-10 | Shape filter | Long thin spot (like a ring line fragment) | Ignored |
| IP-HOL-11 | Overlapping holes | Second shot partly into the first (blob 1.7× the area of one hole) | Second hole reported; `confidence` < 0.5 |
| IP-HOL-12 | Holes in the black and in the white | One new hole inside the black area, one outside | Both detected |
| IP-HOL-13 | Uncount | Counted holes 1, 2, 3; `live_uncount_hole` for hole 2 | Hole 2 gray; the others numbered 1 and 2 |
| IP-HOL-14 | Uncount an unknown hole | `live_uncount_hole` with an unknown id | `{}`; nothing changes |
| IP-HOL-15 | Hole ids | 10 holes in one live mode | Unique, increasing `hole_id`s |

## Paper change (integration)
What is tested: `live_new_paper`, automatic detection and `paper_replaced` ([image_processor.md](../programs/image_processor.md#paper-change)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IP-PPR-01 | Requested change | 3 counted holes; `live_new_paper` with `from_live_paper` 1 | `changed` true, `live_paper` 2; `previous_paper_staged_file` is a `target_*.jpg` of the old paper (transformed, long side `capture_long_side_px`, no drawing, the 3 holes visible); holes forgotten |
| IP-PPR-02 | Request after an automatic change | Automatic change to paper 2, then `live_new_paper` with `from_live_paper` 1 | `changed` false, `live_paper` 2, file `null`; no second change |
| IP-PPR-03 | Change before calibration | `live_new_paper` while never calibrated | `changed` true; file `null` |
| IP-PPR-04 | Automatic swap | Video: paper with 4 holes taken off, a new paper without holes put up | `paper_replaced` with `live_paper` 2 and the image of the old paper |
| IP-PPR-05 | Same paper back | Video: paper taken off and put back | No `paper_replaced` |
| IP-PPR-06 | Old paper without holes | Video: blank paper swapped for another blank paper | No `paper_replaced` |
| IP-PPR-07 | 20 % limit | Old paper with 5 holes; new paper shows 1 of them at the same position, then (second video) 2 | 1 visible (20 %): new paper; 2 visible (40 %): same paper |
| IP-PPR-08 | Shots around the change | A hole confirmed just before the swap, one after | First event `live_paper` 1, second `live_paper` 2 |
| IP-PPR-09 | Counting kept | Counting on before the change | Still on after it |
| IP-PPR-10 | Holes on the new paper | New paper with 2 holes | Shown gray, not counted; numbering starts at 1 |

## Live frames (integration)
What is tested: what is published on `targetscope/frames/live` ([frame_transport.md](../frame_transport.md#live-frames)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IP-FRM-01 | Only when calibrated | Before and after calibration | Nothing before; frames after |
| IP-FRM-02 | Size and scale | `TT-REF`, `live_long_side_px` 1200 | 1200 × 1200; header `px_per_mm` 6 |
| IP-FRM-03 | Setting changes | `live_long_side_px` 800 | Next frames 800 × 800 |
| IP-FRM-04 | Drawing | Pixel probes at a gray hole, a counted hole, a candidate and on a ring | Gray circle; colored circle with its number; dashed circle; thin ring line |
| IP-FRM-05 | Header link | Compare live and raw headers | `source_seq` = `seq` of the raw frame; same `timestamp_ns` |
| IP-FRM-06 | Stop | `live_stop` | No more live frames; `live_calibration_status` `not_calibrated` (`live mode not active`) |
| IP-FRM-07 | Lens-corrected | Distorted render with an applied lens calibration | Straight paper edges in the live frame (deviation < 0.5 px) |

## Capture (integration)
What is tested: `capture_target_image` and `capture_raw_image` ([image_processor.md](../programs/image_processor.md#capture-mode)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IP-CAP-01 | Target image | Live mode calibrated; `capture_target_image` | `target_*.jpg` in the staging folder (written via `.part`); long side 2400; the paper only, no drawing |
| IP-CAP-02 | Hand in front of the paper | The newest frames show a hand | The image comes from the last good frame: no hand |
| IP-CAP-03 | No good frame | Never calibrated; and without live mode | `unavailable` |
| IP-CAP-04 | Size setting | `capture_long_side_px` 1000 | Long side 1000 |
| IP-CAP-05 | Camera picture | `capture_raw_image` without live mode | `raw_*.jpg` with the camera resolution; `lens_corrected` false |
| IP-CAP-06 | Camera picture with lens correction | Applied lens calibration | `lens_corrected` true; straight lines straight |
| IP-CAP-07 | No frames | `capture_raw_image` without frames | `unavailable` |

## Photos and target type proposals (integration)
What is tested: `find_paper`, `prepare_target_photo`, `detect_shots_in_photo` ([image_processor.md](../programs/image_processor.md#target-type-from-a-photo)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IP-PHO-01 | Paper corners | `find_paper` on a photo | Corners within 1 % of the diagonal; `width_px`, `height_px` of the photo |
| IP-PHO-02 | No paper | `find_paper` on a photo without paper | `corners: null`; no error |
| IP-PHO-03 | Proposal | `prepare_target_photo` on a render of `TT-REF`, paper 200 × 200 | `paper_*.jpg`; `px_per_mm`; center (100, 100), black radius 30, white radius 5, outer radius 90, each within 1 mm |
| IP-PHO-04 | Blank paper | `prepare_target_photo` on a white paper | Every proposed value `null` |
| IP-PHO-05 | Shots in a photo | `detect_shots_in_photo` on a render with 10 holes | 10 shots within 0.5 mm; `photo_*.jpg` with long side 2400 |
| IP-PHO-06 | Given corners | Same with `corners` where automatic detection fails (dark background missing) | The given corners are used; shots found |
| IP-PHO-07 | No paper, no corners | Photo without paper | `processing_failed` |
| IP-PHO-08 | Unreadable file | Text file named `.jpg` | `io_error` |
| IP-PHO-09 | Size warning | Spot of 3× the bullet diameter | Listed in `warnings` |
| IP-PHO-10 | No lens correction for photos | Lens calibration applied for the camera; detect in a photo | The photo isn't corrected (result identical to without calibration) |
| IP-PHO-11 | Photos during live mode | Two photo jobs while live mode runs at 30 fps | Live frames keep their rate (within 10 %) |

## Lens calibration (integration)
What is tested: collecting views, computing, storing and applying ([image_processor.md](../programs/image_processor.md#lens-calibration)). Uses checkerboard views from `testdata/lens/` with known distortion.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IP-LNS-01 | During live mode | `lens_calibration_start` while live mode is active | `invalid_state` |
| IP-LNS-02 | Start | `lens_calibration_start` | `views` 0; status `collecting_views` 0 |
| IP-LNS-03 | Capture | Frame with a checkerboard; frame without | `found` true and `views` 1; then `found` false and `views` 1 |
| IP-LNS-04 | Too few views | `lens_calibration_finish` after 9 views | `validation_failed`; views kept |
| IP-LNS-05 | Compute | 12 views; `lens_calibration_finish` | `rms_px` < 0.5; coefficients within 5 % of the truth; `lens_calibrations` has an entry for the `camera_key` with resolution, matrix, coefficients and RMS; status `calibrated` and `applied` true |
| IP-LNS-06 | Applied | Distorted test frame with straight lines | After correction, lines deviate less than 0.5 px from straight |
| IP-LNS-07 | Camera changes while collecting | Resolution changes after 5 views | Views discarded; `collecting_views` 0 |
| IP-LNS-08 | Cancel | `lens_calibration_cancel` after 5 views | Views discarded; nothing stored |
| IP-LNS-09 | Delete | `lens_calibration_delete` | Entry removed from `lens_calibrations`; status `calibrated` false |
| IP-LNS-10 | Other resolution | Calibration for 1920 × 1080; camera now runs at 1280 × 720 | `applied` false; frames not corrected |
| IP-LNS-11 | Other camera | `camera_state_changed` with another `camera_key` | That camera's calibration is applied, or none |
| IP-LNS-12 | Raw feed stays uncorrected | Calibration applied | Frames on `targetscope/frames/raw` unchanged |
| IP-LNS-13 | Accuracy near the edge | Distorted video, shots near the paper edge | With calibration: position error < 1 mm |

## Error handling (integration)
What is tested: missing frames, unreadable files, shutdown.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IP-ERR-01 | Frames stop | Live mode calibrated; frames stop for 3 s | Status `not_calibrated`, message `no camera frames`; `capture_raw_image` and `live_set_counting(true)` answer `unavailable`; `capture_target_image` still works (last good frame); live frames stop |
| IP-ERR-02 | Frames come back | Frames resume | Calibrated again; holes made meanwhile not counted |
| IP-ERR-03 | Shutdown | `SIGTERM` during live mode | Exits within 2 s; publisher removed |

## Performance (performance)
What is tested: processing speed on the development PC. *(goals set here)*

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IP-PRF-01 | Live pipeline | 1920 × 1080 at 30 fps, `TT-REF`, counting on | At least 15 live frames per second (default `live_feed_fps`); delay from raw frame to live frame below 100 ms |
| IP-PRF-02 | Photo | `detect_shots_in_photo` on a 12-megapixel photo | Below 10 s (timeout 60 s) |
| IP-PRF-03 | Lens calibration | `lens_calibration_finish` with 20 views | Below 30 s (timeout 60 s) |

## Accuracy acceptance (test set)
What is tested: detection quality on real photos and videos. Run with `ctest -L accuracy`; the report lists every measure per target type and condition.

The test set is built up in this order: target types 10 m air pistol, 10 m air rifle, 25/50 m pistol, own targets; conditions indoor range with artificial light and camera at an angle to the target (up to about 45°).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| IP-ACC-01 | Photos | `detect_shots_in_photo` on every photo of `testdata/photos/` | Hit rate > 98 % and position error < 1 mm for every target type and condition; false positives reported |
| IP-ACC-02 | Live videos | Every video of `testdata/videos/` through `FrameFeeder`, counting on from start to end | Each true shot reported exactly once (hit rate > 98 %); position error < 1 mm; no shot reported for holes present at the start |
| IP-ACC-03 | Occlusion videos | Videos with hands and people in front of the target | No false shots |
| IP-ACC-04 | Paper swap videos | Videos with paper changes | Every swap reported once; no false swaps |
| IP-ACC-05 | Target type proposals | `prepare_target_photo` on photos of every target type | Proposed center and radii within 1 mm of the real values |

## Hardware tests (manual)
What is tested: the whole camera chain at a real range, before a release.

| ID | Case | Steps | Expected outcome |
|---|---|---|---|
| IP-HW-01 | Live session at the range | USB camera at an angle of about 30°, 10 m air pistol, 20 shots | Every shot appears within 1 s; positions match the paper within 1 mm (checked on the stored target image) |
| IP-HW-02 | Light changes | Switch range lights on and off during a session | No false shots; detection continues |
| IP-HW-03 | Paper change at the range | Change the paper after every 5 shots, without pressing "new paper" | Each change recognized; images of all papers stored |
| IP-HW-04 | Lens calibration with a real camera | Calibrate a wide-angle webcam with the printed checkerboard | RMS below 0.5 px; shots near the paper edge within 1 mm |
| IP-HW-05 | Target type from a phone photo | Create the 10 m air pistol target type from a phone photo | Proposal within 1 mm after adjusting; scores of a test session match a manual count |
