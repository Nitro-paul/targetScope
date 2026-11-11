# image_processor

Program `image_processor` (**C++**): finds the target paper in camera images and photos, transforms it to a straight, cropped view of the paper, detects holes, and draws the live view. Requests and events: [interfaces.md](../interfaces.md#image_processor). Frames: [frame_transport.md](../frame_transport.md).

C++ was chosen because OpenCV is a C++ library: its full API, documentation and examples are available directly, without generated bindings.

## Responsibilities
- **Live mode:** transforms the camera feed to show only the target paper; detects all holes; shows holes that were there before (not counted) and highlights new shots; sends `shot_detected` while counting is on; publishes the live view on `targetscope/frames/live`.
- **Capture mode:** transforms single images to the target paper without any drawing: target image of a session (`capture_target_image`), photos of shot targets (`detect_shots_in_photo`), and photos for new target types (`prepare_target_photo`).
- Finds the paper automatically (`find_paper`, auto calibration), or uses the four corners the user clicked (`live_calibrate`).
- Keeps the live view in line with the session: holes that `session_manager` doesn't count (`live_uncount_hole`) are drawn gray.
- Handles target paper changes during a session (`live_new_paper`, or detected automatically: `paper_replaced`).
- Lens calibration per camera with a printed checkerboard, and lens correction of all camera frames.
- Proposes the parameters of a new target type from a photo.

It doesn't score shots (analyzer) and doesn't decide which shots belong to a session (session_manager).

## Technology
| Purpose | Library |
|---|---|
| Image processing | OpenCV 4 (imgproc, calib3d, imgcodecs) |
| Frames | iceoryx2 C++ bindings, `TargetScopeFrameHeader` from `schemas/frames/frame_header.h` |
| Redis, messages, settings, logging | `targetscope_common_cpp` ([shared_libraries.md](../shared_libraries.md#c-library-targetscope_common_cpp)) |
| Build | CMake, C++20 |
| Tests | GoogleTest |

## Coordinate systems
| System | Origin and axes | Unit |
|---|---|---|
| Raw image | top-left of the camera frame, x right, y down | px |
| Paper | top-left corner of the paper, x right, y down | mm |
| Transformed image | top-left corner of the paper, x right, y down; `paper_mm = px / px_per_mm` | px |
| Shot | ring center (`center_x_mm`, `center_y_mm` of the target type), polar: `angle_turns` clockwise from the top, `radius_mm` in mm | mm |

The transformation from raw image to paper is a homography computed from the four paper corners (`getPerspectiveTransform`), mapping them to `(0,0)`, `(W,0)`, `(W,H)`, `(0,H)` with `W, H` = paper size in mm × `px_per_mm`. A hole at paper position `(px, py)` becomes a shot with `x = px - center_x_mm`, `y = center_y_mm - py` (y flipped to point up), converted to polar with the shared geometry functions.

## Architecture
```
 Redis thread                     processing thread                       photo worker
 (Bus::serve, requests)           (live mode)                             (capture-mode jobs)
 +-----------------------+        +------------------------------------+  +--------------------+
 | live_*                | -cmd-> | wait for raw frame, keep newest     |  | find_paper          |
 | capture_target_image  | <-res- | live mode only:                     |  | prepare_target_photo|
 | capture_raw_image     |        |   find paper (until found / 30th)   |  | detect_shots_in_    |
 |                       |        |   transform -> hole detection       |  |   photo             |
 | find_paper, prepare_, | -job-> |   draw -> publish live frame        |  +--------------------+
 |   detect_shots_in_    |        +------------------------------------+
 | publish events        | <- events (shot_detected, calibration_changed)
 +-----------------------+
```
The processing thread always subscribes to `targetscope/frames/raw` and keeps the newest frame (needed by `capture_raw_image` also outside live mode); the live pipeline only runs in live mode. Commands reach the processing thread through a thread-safe queue and are answered with `std::promise`. Photo jobs run on a small worker pool (2 threads), because they don't need the live state.

```cpp
struct ProcessorState {
    cv::Mat newest_raw;                 // newest camera frame (lens-corrected if a calibration applies)
    std::optional<LiveState> live;      // set between live_start and live_stop
    CameraInfo camera;                  // camera_key and resolution, from camera_reader events / camera_status
    std::optional<LensCalibration> lens;            // applied calibration of the current camera
    std::optional<LensCalibrationSession> collecting; // between lens_calibration_start and _finish/_cancel
};

struct LiveState {
    TargetType target_type;
    double bullet_diameter_mm;
    std::optional<Calibration> calibration;
    cv::Mat reference;                  // transformed grayscale image incl. all known holes
    std::vector<Hole> holes;            // all confirmed holes
    std::vector<Candidate> candidates;  // not yet confirmed
    bool counting = false;
    int next_hole_id = 1;
    int live_paper = 1;                 // +1 at every paper change; sent with shot_detected
    cv::Mat last_good_raw;              // newest frame with the current paper fully visible, and
    cv::Mat last_good_homography;       //   its homography: used for the image of a paper that is replaced
};

struct Calibration {
    Method method;                      // Auto | Manual
    Corners corners;                    // paper corners in the raw image
    cv::Mat homography;                 // raw -> transformed live image
    double px_per_mm;
    CalState state;                     // Calibrated | Lost
};

struct Hole {
    int id;                             // unique within one live mode, sent as hole_id
    cv::Point2d paper_mm;
    bool counted;                       // true: session shot (highlighted), false: old/uncounted (gray)
    float confidence;
};                                      // shot number in the live view = position among counted holes of the current paper
```

## Finding the paper
Used by auto calibration, `find_paper` and `detect_shots_in_photo`:
1. Grayscale, Gaussian blur 5×5, Otsu threshold (the paper is brighter than the background).
2. Find external contours; approximate each with `approxPolyDP` (epsilon 2 % of the perimeter).
3. Keep convex quadrilaterals covering at least 10 % of the image. Take the largest.
4. Refine the corners with `cornerSubPix`; order them top-left, top-right, bottom-right, bottom-left.
5. Check the aspect ratio of the transformed quadrilateral against `paper_width_mm / paper_height_mm` (if the target type is known); more than 10 % off → not found.
6. Check the ring center: in the transformed image, find the black aiming area (largest dark, round blob, `fitEllipse`). If its center is more than 3 mm from (`center_x_mm`, `center_y_mm`), the paper is probably rotated by 90° or 180°; try the other three corner orders and keep the one that matches. If none matches: not found.

The camera is often at an angle to the target, so this step relies only on the paper outline and the black area, both of which stay clearly visible in perspective.

## Live mode

### Start and calibration
On `live_start`, the target type and bullet diameter are stored (a running live mode is replaced, and all its holes are forgotten; `live_paper = 1`). The paper is searched about once per second until it's found; without frames, the status message is `no camera frames`. After `live_calibrate`, the clicked corners are used instead. They're in raw camera pixels (the raw feed isn't lens-corrected), so with an applied lens calibration they're converted with `undistortPoints` first; `CalibrationStatus.corners` is always reported in raw camera pixels. On success: `calibration_changed` (`calibrated`), then the **initial hole scan**.

**Supervision:** every 30th frame, the paper is searched again near the known corners. If a corner moved more than 2 mm (paper scale), the calibration is updated (auto) or set to `lost` (manual), and `calibration_changed` is sent. After an update, the reference image is created again from the current frame; the known holes keep their paper positions, so nothing is counted twice. While `lost`, no holes are detected.

### Initial hole scan
All holes that are already in the target are found with the single-image method (see [photo detection](#holes-in-a-single-image)) on the first transformed frame. They're stored as not counted and drawn gray. This frame becomes the reference.

### New holes
For each new frame:
1. Transform and convert to grayscale.
2. **Occlusion check:** if more than 5 % of the paper's pixels differ strongly from the reference (person, hand, shadow), skip the frame.
3. `absdiff` with the reference, threshold, morphological opening to remove noise.
4. Find blobs (connected components). Keep blobs with a diameter between 0.6× and 2× the bullet diameter, and a roundness above 0.6.
5. Match blobs to candidates (within half a bullet diameter). A candidate seen in `detection_confirm_frames` consecutive frames (default 5) becomes a hole:
   - while counting is on: counted, highlighted, and sent as `shot_detected` with its `hole_id` (confidence from size and roundness),
   - while counting is off: not counted, drawn gray.
   The region of the hole is copied from the current frame into the reference, so it isn't detected again.
6. Candidates not seen for 3 frames are dropped.

**Last good frame:** whenever a frame is calibrated and the occlusion check passes, it's kept (with its homography) as `last_good_raw`. It's only warped when needed, for the image of a replaced paper.

### Paper change
A new paper starts in one of two ways:
- **By request** (`live_new_paper`, the user pressed "new paper"): only if `from_live_paper` equals the current `live_paper` (otherwise the paper was already changed automatically, and nothing happens again). The last good frame of the old paper is warped at `capture_long_side_px` and saved as `target_<uuid>.jpg`.
- **Automatically:** after the calibration was `lost` (the paper was taken away) and the paper is found again, all known holes of the old paper (counted or not) are looked for at their paper positions. If at most 20 % of them are visible, it's a new paper: the same image is saved and `paper_replaced` is sent. Otherwise it's the same paper again and nothing changes. If the old paper had no holes at all, nothing changes either (a blank paper replaced by a blank paper makes no difference).

Then `live_paper` increases, all holes are forgotten, the initial hole scan runs on the new paper (holes found are shown gray, not counted), and the live view numbering starts again at 1. Counting keeps its state. Every `shot_detected` carries the `live_paper` of its hole, so session_manager assigns it to the right paper even if it arrives after the change.

**Uncounting:** `live_uncount_hole` sets `counted = false` for the hole; it's drawn gray, and the remaining counted holes are numbered again.

**Overlapping holes:** if a confirmed blob grows to more than 1.6× the area of a single hole, it's treated as two holes; the second center is placed at the far end of the blob from the first. Such holes get a confidence below 0.5, so the UI highlights them for checking in Review.

### Live frame
After each processed frame, the transformed color image is drawn as described in [frame_transport.md](../frame_transport.md#live-frames) and published. Without calibration, nothing is published on `targetscope/frames/live` (the web server shows the raw feed instead).

## Capture mode

### capture_target_image
Transforms `last_good_raw` (the newest frame in which the paper was calibrated and nothing was in front of it) with its homography, at a resolution where the long side has `capture_long_side_px` pixels (default 2400). Nothing is drawn. Written as JPEG (quality 92) to the staging folder as `target_<uuid>.jpg`. This is the image stored with the session.

### capture_raw_image
Writes `newest_raw` (not transformed, but lens-corrected if a calibration applies) as JPEG to the staging folder (`raw_<uuid>.jpg`). Used to take a picture for a new target type.

### Holes in a single image
Used for photos and the initial hole scan; there is no reference image:
1. Inside the black area (`black_radius_mm` around the ring center), holes are lighter than their surroundings; outside, darker. Adaptive threshold separately for both areas, with the polarity of each.
2. Ring lines are thin and long: blobs are filtered as in step 4 of [new holes](#new-holes) (size and roundness), which removes line fragments.
3. Blobs rejected only because of their size are reported as warnings.

### detect_shots_in_photo
1. Load the photo from the staging folder.
2. Find the paper (or use the given corners). If not found: `processing_failed`; the UI then asks the user to click the corners.
3. Transform at `capture_long_side_px` and write it to the staging folder (`photo_<uuid>.jpg`); this is the image stored with the session.
4. Detect holes in the single image; return them as shots with confidence, plus warnings.

### Target type from a photo
1. `find_paper` returns the paper corners of the picture (taken with `capture_raw_image` or uploaded), or `null` if it finds none. The user corrects or places them in the web UI and enters the paper size.
2. `prepare_target_photo` transforms the photo with the corners and paper size, writes it as `paper_<uuid>.jpg` to the staging folder, and proposes:
   - **ring center and black radius**: center and mean radius of the fitted ellipse of the black area,
   - **inner white radius**: the light, round area at the center inside the black area,
   - **outer radius**: from the ring center, the radial intensity profile is averaged over all angles; ring lines show as dark peaks; the outermost peak inside the paper is the outermost ring.
   Values that can't be found are `null`.
3. The web UI shows the proposal as an overlay on the transformed photo; the user adjusts it (see [target type editor](web_server.md#target-type-editor)).

## Lens calibration
Cheap webcams bend straight lines (barrel distortion). The homography from the paper corners can't remove that, and near the paper edges the error can exceed the 1 mm goal. An optional lens calibration per camera corrects it.

**Collecting** (camera settings menu of the web UI, live mode not active):
1. `lens_calibration_start`. The user holds a printed checkerboard (9 × 6 inner corners; PDF in the web UI; the square size doesn't matter for the distortion) in front of the camera.
2. For each view, `lens_calibration_capture`: `findChessboardCornersSB` on the newest raw frame (not corrected); if found, the corners are kept. The UI asks for at least 10 views, at different positions, distances and angles, covering the image corners too.
3. `lens_calibration_finish`: `calibrateCamera` with all views → camera matrix and distortion coefficients (k1, k2, p1, p2, k3) and the RMS reprojection error. Stored in the setting `lens_calibrations` under the camera's `camera_key`, together with the resolution (`save_setting` through `database_handler`). The UI shows the RMS error; below 0.5 px is good, above 1 px suggests repeating.

**Camera changes:** if the camera or its resolution changes while views are being collected, the views are discarded (`lens_calibration_status.collecting_views` drops to 0) and the UI asks to start again.

**Applying:** the current camera's `camera_key` and resolution come from `camera_status` at startup and from `camera_state_changed` events. If a calibration for that `camera_key` and resolution exists, `initUndistortRectifyMap` builds a correction map once:
- every raw frame is corrected before anything else (`remap`), so finding the paper, hole detection, captures and the live view all work on corrected frames,
- in live mode, the correction and the homography are combined into one remap table, so each frame is warped only once.

A calibration for another resolution isn't applied (`lens_calibration_status.applied = false`); the UI suggests calibrating again. Photos (from other cameras, e.g. a phone) are never lens-corrected.

## Error handling
| Situation | Behavior |
|---|---|
| No raw frames for 2 s | Calibration state `not_calibrated` with message `no camera frames`; `capture_*` and `live_set_counting(true)` return `unavailable`; live frames stop; the web server shows its fallback |
| Paper lost during live mode | `calibration_changed` (`lost`); detection pauses until it's found again; then the reference is renewed and holes made in the meantime are added as not counted |
| Photo not readable | `io_error` |
| Paper not found in a photo | `find_paper`: `corners = null`; `detect_shots_in_photo`: `processing_failed` |

## Testing
- Unit tests: homography and coordinate conversions, corner ordering, ring generation, combined lens + homography map against applying both one after the other.
- Paper change: video with a paper swap → `paper_replaced` once, holes of the new paper not counted; taking the same paper off and back on → no paper change.
- **Test set** of photos and short videos with known hole positions (marked by hand once). Goal: hit rate > 98 %, position error < 1 mm. It is built up in this order:
  1. target types: 10 m air pistol, 10 m air rifle, 25/50 m pistol, own targets;
  2. conditions: indoor range with artificial light; camera at an angle to the target (up to about 45°).
- Live detection is tested by feeding recorded videos through a test frame publisher, including occlusion (a hand in front of the target) and a pause with counting off.
