# Tests: camera_reader

Tests of the program `camera_reader` (C++).

Design: [camera_reader.md](../programs/camera_reader.md), [interfaces.md](../interfaces.md#camera_reader), [frame_transport.md](../frame_transport.md). Reference data and tools: [overview.md](overview.md).

## Setup
- **Unit tests** (GoogleTest) use `FakeSource`: generated frames with a known pattern, scripted read failures and stalls.
- **Integration tests** run `camera_reader` as a process with `RedisTestServer`, a `FakeProgram` as `database_handler` (settings) and a Rust frame subscriber. Camera sources: `v4l2loopback` devices fed by `ffmpeg`, the `vivid` driver for controls, `mediamtx` for RTSP.
- Frames are checked with a test subscriber on `targetscope/frames/raw`.
- Cases that need a camera use the first camera of the PC when it has one (the tests never change a control value) and are skipped otherwise; the cases that need `v4l2loopback`, `vivid` or `mediamtx` are skipped where these aren't installed.

## Source selection (integration)
What is tested: which camera is opened, depending on the settings.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| CR-SRC-01 | No camera | `camera_source` `none` | State `disabled`; `camera_status` `disabled`; `camera_state_changed` `disabled`; no frames |
| CR-SRC-02 | First camera | `camera_source` `v4l2`, `camera_device` empty, two loopback devices | The capture device with the lowest `/dev/videoN` is opened |
| CR-SRC-03 | Camera by `bus_info` | `camera_device` = `bus_info` of the second loopback device | The second device is opened; `camera_status.device` is its path |
| CR-SRC-04 | Camera not present | `camera_device` = an unknown `bus_info` | State `disconnected`, message names the camera; retried every 2 s |
| CR-SRC-05 | Network camera without URL | `camera_source` `rtsp`, `camera_rtsp_url` inactive | State `disconnected`, message names `camera_rtsp_url` |
| CR-SRC-06 | Network camera | `camera_source` `rtsp`, URL of a `mediamtx` stream (1280 × 720) | `connected`; frames 1280 × 720; `camera_key` = the URL |

## Capture loop and frames (unit and integration)
What is tested: published frames and their headers ([camera_reader.md](../programs/camera_reader.md#capture-loop)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| CR-CAP-01 | Header fields | `FakeSource` 640 × 480 at 30 fps, 100 frames | `seq` 0, 1, 2, … without gaps; `source_seq` = `seq`; `timestamp_ns` increasing; `width` 640, `height` 480, `stride` 1920, `format` 1, `px_per_mm` 0 |
| CR-CAP-02 | Pixels unchanged | Same; compare payloads with the generated pattern | Byte-equal |
| CR-CAP-03 | YUYV conversion | Loopback device with YUYV frames of a known image | BGR pixels within ±3 of the reference |
| CR-CAP-04 | MJPEG conversion | Loopback device with MJPEG frames of a known image | BGR pixels within ±10 of the reference |
| CR-CAP-05 | Resolution not available | Request 1920 × 1080 from a device that offers 1280 × 720 only | Frames 1280 × 720; `camera_status` shows 1280 × 720; a warning is logged |
| CR-CAP-06 | Nobody watching | No subscriber | Frames are published; `frames_published` increases; no error |
| CR-CAP-07 | Frame counter | `camera_status` twice, 1 s apart, at 30 fps | `frames_published` grew by about 30 |
| CR-CAP-08 | Publisher size | Inspect the publisher configuration after opening 1280 × 720 | `initial_max_slice_len` = 1280 × 720 × 3 |

## Reconnecting (unit and integration)
What is tested: losing and finding the camera again. Unit tests use `FakeClock`.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| CR-REC-01 | Read errors | `FakeSource` fails 10 reads in a row, then works again | `disconnected` after the 10th failure; source closed; reopened every 2 s; `connected` once it works |
| CR-REC-02 | Nine read errors | 9 failures, then frames again | Stays `connected` |
| CR-REC-03 | No frames for 2 s | `FakeSource` stalls | `disconnected` after 2 s |
| CR-REC-04 | Network stream stops | Stop `mediamtx`; start it again | `disconnected`; `connected` again within 4 s of the restart |
| CR-REC-05 | Every state change is sent | Run CR-REC-01 | One `camera_state_changed` per change, with `camera_key`, `width_px`, `height_px` |

## Setting changes (integration)
What is tested: which settings reopen the camera ([camera_reader.md](../programs/camera_reader.md#responsibilities)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| CR-SET-01 | Frame rate | `camera_fps` 30 → 15 | Camera reopened; frames at 15 fps; `camera_status.fps` 15 |
| CR-SET-02 | Source off | `camera_source` → `none` | `disabled`; frames stop |
| CR-SET-03 | Each reopen setting | Change `camera_source`, `camera_device`, `camera_rtsp_url`, `camera_width_px`, `camera_height_px`, `camera_fps` one after the other | Each change reopens the camera |
| CR-SET-04 | Stored control values | `camera_controls` changes (written by camera_reader itself) | No reopen: frame numbers continue without a gap of more than 100 ms |
| CR-SET-05 | Controls reset | `camera_controls` reset to `{}` | Every control of the current camera set to its `default` (auto modes first); no reopen |
| CR-SET-06 | Other settings | `live_feed_fps` changes | Nothing happens |

## Camera controls (integration, `vivid`)
What is tested: listing, setting, storing and reapplying controls ([camera_reader.md](../programs/camera_reader.md#camera-controls)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| CR-CTL-01 | List | `camera_list_controls` | Controls of kind `integer`, `boolean`, `menu`, `button` with `min`, `max`, `step`, `default`, `value`; menu entries with value and label; no control classes, no disabled controls |
| CR-CTL-02 | Inactive controls | Switch an auto mode on | The dependent control has `inactive` true |
| CR-CTL-03 | Set a value | `camera_set_control` with a valid value | Applied (read back from the device); response contains all controls; setting `camera_controls` contains `{ "<camera_key>": { "<id>": value } }` |
| CR-CTL-04 | One control changes another | Switch auto exposure off | Response shows exposure no longer `inactive` |
| CR-CTL-05 | Out of range | Value above `max` | `validation_failed`; nothing stored |
| CR-CTL-06 | Wrong step | `step` 10, value 15 | `validation_failed` |
| CR-CTL-07 | Unknown control | Unknown id | `validation_failed` |
| CR-CTL-08 | No camera | Camera `disconnected`; list and set | `unavailable` |
| CR-CTL-09 | Network camera | `camera_list_controls` with RTSP | `controls: []` |
| CR-CTL-10 | Reapplied after reopening | Store manual exposure mode and exposure 300; reopen the camera | Both applied; the auto mode first, so exposure isn't rejected as inactive |
| CR-CTL-11 | Stored value no longer valid | Stored value outside the device's current range | Warning logged; the device keeps its value; capture continues |
| CR-CTL-12 | Values per camera | Set a value on camera 1, switch to camera 2, set a value there | `camera_controls` has separate entries for both `camera_key`s |

## Devices and camera identity (integration)
What is tested: `camera_list_devices` and `camera_key` ([camera_reader.md](../programs/camera_reader.md#camera-identity)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| CR-DEV-01 | Device list | Two loopback devices with known formats | Both with `path`, `name`, `bus_info` and formats (`width_px`, `height_px`, `fps` list) |
| CR-DEV-02 | No cameras | No capture devices | `devices: []` |
| CR-DEV-03 | Identity of a USB camera | `camera_status` and events with a V4L2 camera | `camera_key` = its `bus_info`, with the actual resolution |
| CR-DEV-04 | Identity of a network camera | Same with RTSP | `camera_key` = the URL |

## Error handling (integration)
What is tested: independence from Redis, clean shutdown.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| CR-ERR-01 | Redis down | Stop Redis for 5 s | Frames keep being published; afterwards requests are answered and events sent again |
| CR-ERR-02 | Shutdown | `SIGTERM` | Camera closed, publisher removed, exit within 2 s |

## Hardware tests (manual)
What is tested: real cameras, before a release.

| ID | Case | Steps | Expected outcome |
|---|---|---|---|
| CR-HW-01 | USB webcam | Plug in a UVC webcam; open the camera settings menu | Listed with its name and formats; live raw feed visible |
| CR-HW-02 | Exposure | Switch auto exposure off and change exposure | Brightness changes visibly; value kept after reopening and after a restart of TargetScope |
| CR-HW-03 | Unplug and plug in | Unplug during a session; plug into the same port | `disconnected` within 2 s; back within 4 s; controls applied again |
| CR-HW-04 | Two cameras | Two webcams; select the second; restart the PC | The second camera is selected again (same USB ports) |
| CR-HW-05 | Network camera | A real RTSP camera in the LAN | Frames at the camera's resolution; reconnect after a network interruption |
| CR-HW-06 | Metadata devices | A webcam that creates two `/dev/video` nodes | Only the capture node is listed |
