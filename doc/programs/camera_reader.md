# camera_reader

Program `camera_reader` (**C++**): reads frames from the camera, publishes them on the iceoryx2 service `targetscope/frames/raw` ([frame_transport.md](../frame_transport.md)), and gives access to the camera's controls. Requests and events: [interfaces.md](../interfaces.md#camera_reader).

C++ was chosen because both camera access paths are C/C++ APIs: OpenCV for reading frames and the V4L2 kernel interface (`ioctl`) for camera controls.

## Responsibilities
- Opens the configured camera source: USB camera (V4L2) or network camera (RTSP).
- Reads frames continuously, converts them to BGR, and publishes every frame with a `TargetScopeFrameHeader`.
- Lists the available USB cameras and their formats for the camera settings menu.
- Lists the controls of the current camera (exposure, gain, focus, white balance, zoom, … as the camera reports them), sets them, stores their values and applies them again after every reconnect.
- Reconnects automatically when the camera is unplugged or the stream breaks.
- Reports its state (`camera_status`, `camera_state_changed`).
- Reopens the camera when one of the settings `camera_source`, `camera_device`, `camera_rtsp_url`, `camera_width_px`, `camera_height_px` or `camera_fps` changes. A change of `camera_controls` doesn't reopen the camera (camera_reader writes that setting itself; only a reset to `{}` is acted on, see [Camera controls](#camera-controls)).

It doesn't analyze images; that is the image_processor's job.

## Technology
| Purpose | Library |
|---|---|
| Reading frames | OpenCV 4 or 5 `cv::VideoCapture` (backend V4L2 for USB, FFmpeg for RTSP) |
| Camera controls, device and format lists | V4L2 `ioctl`s (`linux/videodev2.h`) |
| Frame publishing | iceoryx2 C++ bindings |
| Redis, messages, settings, logging | `targetscope_common_cpp` |
| Build, tests | CMake, C++20, GoogleTest |

## Configuration (settings)
`camera_source` (`v4l2`, `rtsp`, `none`), `camera_device`, `camera_rtsp_url`, `camera_width_px`, `camera_height_px`, `camera_fps`, `camera_controls`. See [database.md](../database.md#known-settings). They're changed in the camera settings menu of the web UI.

## Architecture
```
 Redis thread                              capture thread
 +---------------------------------+      +------------------------------------------+
 | Bus::serve                      |      | loop:                                     |
 |  camera_status                  | <--- |   open source (retry every 2 s)           |
 |  camera_list_devices  (ioctl)   |      |   apply stored controls                   |
 |  camera_list_controls (ioctl)   |      |   read frame -> copy into loaned slice    |
 |  camera_set_control   (ioctl)   |      |   -> publish + notify                     |
 | settings watch -> reopen ------ | ---> |                                           |
 | publish camera_state_changed    |      +------------------------------------------+
 +---------------------------------+
```
Controls are set through a separate file descriptor on the same device (`open(device, O_RDWR)`); V4L2 allows this while OpenCV is capturing.

## Sources
```cpp
class FrameSource {
public:
    virtual ~FrameSource() = default;
    virtual SourceInfo open() = 0;                 // actual width, height, fps
    virtual void read(cv::Mat& bgr) = 0;           // blocks until the next frame
    virtual void close() = 0;
    virtual bool has_controls() const = 0;
};

class V4l2Source : public FrameSource {
    // open(): resolve the setting camera_device (bus_info) to the current /dev/videoN by checking
    // VIDIOC_QUERYCAP of every capture device; empty setting = the capture device with the lowest N.
    // cv::VideoCapture(device, cv::CAP_V4L2); requests MJPG for high resolutions at full frame rate;
    // sets CAP_PROP_FRAME_WIDTH/HEIGHT/FPS and CAP_PROP_BUFFERSIZE = 1 for low latency.
};

class RtspSource : public FrameSource {
    // cv::VideoCapture(url, cv::CAP_FFMPEG) with OPENCV_FFMPEG_CAPTURE_OPTIONS="rtsp_transport;tcp".
    // Resolution and frame rate come from the camera; no controls.
};
```

## Camera identity
Every camera has a `camera_key` under which per-camera data is stored (`camera_controls` here, `lens_calibrations` in image_processor): the V4L2 `bus_info` for USB cameras (stays the same when the device number `/dev/videoN` changes, as long as the camera is plugged into the same port), the RTSP URL for network cameras. For a USB camera it is the `bus_info` of the camera that was opened, also when the setting `camera_device` is empty (first camera found). It's part of `camera_status` and of every `camera_state_changed` event, together with the actual resolution.

## Camera controls
- **List** (`camera_list_controls`): enumerate with `VIDIOC_QUERY_EXT_CTRL` and the flag `V4L2_CTRL_FLAG_NEXT_CTRL`; skip control classes and disabled controls; for menus, read the entries with `VIDIOC_QUERYMENU`; read current values with `VIDIOC_G_EXT_CTRLS`. `inactive` comes from `V4L2_CTRL_FLAG_INACTIVE`.
- **Set** (`camera_set_control`): check range and step, `VIDIOC_S_EXT_CTRLS`, read all controls again (one control can change others), then store the value in the setting `camera_controls` under the camera's `camera_key` (`save_setting` through `database_handler`).
- **Apply after open:** all stored values of this camera are set in the order auto modes first (e.g. auto exposure), then the rest, so dependent controls aren't rejected as inactive.
- **Reset:** when `camera_controls` is reset (`data_changed` with the value `{}`), every control of the current camera is set to its `default` (auto modes first).
- **Devices** (`camera_list_devices`): `/dev/video*` with `VIDIOC_QUERYCAP` (only devices with `V4L2_CAP_VIDEO_CAPTURE`), formats with `VIDIOC_ENUM_FMT`, `VIDIOC_ENUM_FRAMESIZES` and `VIDIOC_ENUM_FRAMEINTERVALS`.

## Capture loop
1. Create the source from the settings. With `camera_source = none`: state `disabled`, wait for a settings change.
2. `open()`. On failure: state `disconnected`, retry every 2 seconds.
3. On success: apply stored controls; state `connected`; create the frame publisher with `initial_max_slice_len = width × height × 3`.
4. For each frame: `read()`, set `seq`, `source_seq = seq`, `timestamp_ns` (CLOCK_MONOTONIC at the time of `read`), `width`, `height`, `stride`, `format = BGR8`, `px_per_mm = 0`, copy the pixels into a loaned slice, publish and notify.
5. If `read()` fails 10 times in a row or no frame arrives for 2 seconds: `close()`, state `disconnected`, back to step 2. A watchdog thread reports `disconnected` as soon as the 2 seconds have passed, even while `read()` still blocks.
6. If one of the reopen settings above changes: `close()` and back to step 1.

Every state change is sent as `camera_state_changed` and logged.

## Error handling
| Situation | Behavior |
|---|---|
| Camera not present / RTSP unreachable | State `disconnected`, retry every 2 s, no crash; the message names the camera (`bus_info` or URL) |
| `camera_source` `rtsp` without an active `camera_rtsp_url` | State `disconnected`, message names `camera_rtsp_url`; opened as soon as the setting is set |
| Camera delivers a different resolution than requested | Use the real resolution, log a warning; the header carries it |
| A stored control value is rejected (e.g. different firmware) | Log a warning, keep the camera's value |
| No subscriber | Frames are still published |
| Redis down | Capture continues; requests, status and events resume after reconnect |

## Testing
All test cases with their expected outcomes: [testing/camera_reader.md](../testing/camera_reader.md).
