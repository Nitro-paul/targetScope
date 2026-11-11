# Frame transport

Camera frames are too large for Redis (a 1920×1080 BGR frame is about 6 MB; at 30 fps that's about 180 MB/s). They are passed between programs with **iceoryx2**, a zero-copy publish/subscribe library over shared memory: the publisher writes a frame once into shared memory, and subscribers read it in place, without copying. iceoryx2 has Rust, C and C++ APIs that share the same shared-memory format, so publisher and subscriber can be written in different languages.

```
 camera_reader (C++) ══ targetscope/frames/raw ══> image_processor (C++) ══ targetscope/frames/live ══> web_server (Rust) ──MJPEG──> browser
                               ║                                                                               ^
                               ╚═══════════════════ raw feed (manual calibration, fallback) ═══════════════════╝
```

Redis is still used for everything else about the camera (status, controls, calibration, detection and results; see [interfaces.md](interfaces.md)).

## iceoryx2 services
| Service name | Publisher | Subscribers | Content |
|---|---|---|---|
| `targetscope/frames/raw` | `camera_reader` | `image_processor`; `web_server` for `/stream/raw.mjpg` and as fallback | Frames as read from the camera |
| `targetscope/frames/live` | `image_processor` (live mode) | `web_server` | Transformed target paper with the holes and shots drawn on it |

Each publish/subscribe service has a matching event service with the same name plus `/notify`. The publisher notifies it after each frame, so subscribers can block on a listener instead of polling.

Service configuration (identical in every language; the values are part of the contract):

| Setting | Value | Reason |
|---|---|---|
| payload type | slice of `u8` / `uint8_t` | pixel data |
| user header | `FrameHeader` (type name `TargetScopeFrameHeader`) | see below |
| `max_publishers` | 1 | |
| `max_subscribers` | 4 | |
| `history_size` | 0 | late subscribers don't get old frames |
| `subscriber_max_buffer_size` | 2 | at most 2 frames queued per subscriber |
| `enable_safe_overflow` | true | a slow subscriber loses the oldest frame; the publisher never blocks |
| publisher allocation | `initial_max_slice_len = width × height × 3`, strategy `PowerOfTwo` | a resolution change reallocates automatically |

## Frame format
Each sample is a byte slice with the pixel data and a `FrameHeader`. The header is defined once as a C header, `schemas/frames/frame_header.h`; this is the contract for every language:
```c
/* schemas/frames/frame_header.h */
#include <stdint.h>

#define TARGETSCOPE_FRAME_HEADER_TYPE_NAME "TargetScopeFrameHeader"

enum TargetScopePixelFormat { TARGETSCOPE_PIXEL_BGR8 = 1 };

typedef struct {
    uint64_t seq;            /* increases by 1 per frame of the publisher */
    uint64_t timestamp_ns;   /* capture time of the raw frame, CLOCK_MONOTONIC */
    uint64_t source_seq;     /* live frames: seq of the raw frame they are based on; raw: = seq */
    uint32_t width;
    uint32_t height;
    uint32_t stride;         /* bytes per row */
    uint32_t format;         /* TargetScopePixelFormat */
    double   px_per_mm;      /* live frames: scale of the transformed paper image; raw: 0 */
} TargetScopeFrameHeader;    /* 48 bytes, 8-byte aligned */
```
- C++ uses the struct directly and sets `static constexpr const char* IOX2_TYPE_NAME = TARGETSCOPE_FRAME_HEADER_TYPE_NAME;` in a wrapper type, so iceoryx2 identifies it by name across languages.
- Rust mirrors it as a `#[repr(C)]` struct with `#[derive(ZeroCopySend)]` and `#[type_name("TargetScopeFrameHeader")]`.
- Both sides have compile-time checks of size (48) and field offsets.
- Pixel format is always BGR8 (OpenCV's native format). `camera_reader` converts from the camera's format (MJPEG, YUYV, H.264).

### Live frames
In live mode, `image_processor` publishes the target paper, transformed so it is seen straight on and cropped exactly to the paper (`paper_width_mm` × `paper_height_mm` of the target type). The long side has `live_long_side_px` pixels (default 1200); `px_per_mm` is in the header. Drawn on it:
- holes that were there before the session, or made while counting was off: thin gray circle,
- counted holes (the session's shots): colored circle with the bullet diameter and its number, the position among the counted holes (holes uncounted with `live_uncount_hole` turn gray and the others are renumbered, so the numbers match the session's shot list); the newest shot is emphasized,
- candidates not yet confirmed: dashed circle,
- the rings of the target type, thin, to check the calibration.

Because the image shows the paper at a fixed scale, the web UI can map a click on the live image directly to paper millimeters.

## Subscriber rules
- A subscriber may hold at most 2 samples at the same time and must release them quickly. `image_processor` processes a frame directly in shared memory and releases it before taking the next one. If it is slower than the camera, frames are skipped (safe overflow); it always processes the newest frame.
- Subscribers run the receive loop in a dedicated thread and hand results to the rest of the program through a queue or channel.
- If no frame arrives for 2 seconds, the subscriber considers the source gone. It keeps waiting, because iceoryx2 reconnects automatically when the publisher restarts.

## Live feed to the browser (MJPEG)
`web_server` delivers the feeds as MJPEG over HTTP (`Content-Type: multipart/x-mixed-replace; boundary=frame`). The browser shows them with a plain `<img>` tag.
- `GET /stream/live.mjpg`: frames of `targetscope/frames/live`
- `GET /stream/raw.mjpg`: frames of `targetscope/frames/raw`, used to click the paper corners for manual calibration and to set up the camera

For each stream:
1. A dedicated thread subscribes to the service, but only while at least one browser is watching.
2. It encodes each frame once as JPEG (`turbojpeg`, quality `live_feed_quality`, default 80) and limits the rate to `live_feed_fps` (default 15). Raw frames are downscaled to a width of 1280 px first.
3. The JPEG is put into a `tokio::sync::watch` channel, which always holds only the newest frame.
4. Each HTTP connection waits on the channel and writes every new JPEG as one multipart part. A slow client skips frames instead of falling behind.

**Fallback:** if live mode is active but `image_processor` sends no live frames for 2 seconds (e.g. it's restarting), `/stream/live.mjpg` shows the raw frames until live frames arrive again. The UI marks the feed as "not transformed".

## Stale shared memory
If a program crashes, its iceoryx2 resources stay in shared memory. On startup, the launcher (and every iceoryx2 program) lists the iceoryx2 nodes and removes the resources of dead nodes.

## Why iceoryx2
| Option | Copies per frame and subscriber | Notes |
|---|---|---|
| **iceoryx2** (chosen) | 0 | Rust, C and C++ APIs; lock-free; many subscribers; built-in cleanup of crashed nodes |
| Unix domain sockets | 2 (into and out of the kernel) | ~180 MB/s of copies per subscriber at 1080p30 |
| Own memfd ring buffer | 0 | Same speed, but synchronization and crash cleanup must be written by hand in every language |
