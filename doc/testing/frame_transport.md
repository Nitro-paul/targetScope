# Tests: frame transport

Tests of the iceoryx2 frame services and the frame header, in both languages. The MJPEG feeds of `web_server` are tested in [web_server.md](web_server.md#mjpeg-feeds-integration).

Design: [frame_transport.md](../frame_transport.md). Reference data and tools: [overview.md](overview.md).

## Frame header (contract)
What is tested: the C header `schemas/frames/frame_header.h` is the same struct in C++ and Rust.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| FT-HDR-01 | Size and offsets | Build the Rust and C++ libraries (compile-time checks) and run the runtime check | Size 48, alignment 8; offsets: `seq` 0, `timestamp_ns` 8, `source_seq` 16, `width` 24, `height` 28, `stride` 32, `format` 36, `px_per_mm` 40 |
| FT-HDR-02 | Type name | Rust subscriber opens a service created by a C++ publisher | Opens; both use the type name `TargetScopeFrameHeader` |
| FT-HDR-03 | Wrong header type | A publisher with a different header type creates `targetscope/frames/raw`; a `FrameHeader` subscriber opens it | Opening fails with a type mismatch error; nothing crashes |
| FT-HDR-04 | Pixel format value | Read `format` of a published frame | 1 (`TARGETSCOPE_PIXEL_BGR8`) |

## Across languages (integration)
What is tested: frames pass between C++ and Rust unchanged.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| FT-XL-01 | C++ to Rust | C++ test publisher sends 100 frames of 1920 × 1080 with a known pattern; Rust subscriber receives | Every header field and every payload byte identical |
| FT-XL-02 | Rust to C++ | The same in the other direction | Identical |

## Service configuration (integration)
What is tested: the service settings that are part of the contract ([frame_transport.md](../frame_transport.md#iceoryx2-services)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| FT-SVC-01 | Same configuration in both languages | Read the static service configuration from both libraries | `max_publishers` 1, `max_subscribers` 4, `history_size` 0, `subscriber_max_buffer_size` 2, safe overflow on |
| FT-SVC-02 | One publisher | Create a second publisher on the same service | Fails; the first keeps working |
| FT-SVC-03 | Four subscribers | Create a fifth subscriber | Fails; the first four keep receiving |
| FT-SVC-04 | No history | Publish 3 frames, then create a subscriber | It receives only frames published after it was created |
| FT-SVC-05 | Slow subscriber | Publisher at 100 fps; subscriber takes a frame every 200 ms | Every `publish` takes less than 1 ms (never blocks); the subscriber gets the newest frames (gaps in `seq`), never more than 2 queued |
| FT-SVC-06 | Borrow limit | Subscriber holds 2 samples and tries to take a third | The third receive fails without crashing; after releasing one sample, receiving works again |
| FT-SVC-07 | Resolution change | Publish 1280 × 720, then 1920 × 1080, then 3840 × 2160 frames | All delivered; header sizes match; payload length = `height × stride` |
| FT-SVC-08 | Notification | Subscriber waits (`wait(5 s)`); then a frame is published | `wait` returns the frame within 5 ms |
| FT-SVC-09 | Wait timeout | No publisher; `wait(2 s)` | Returns "no frame" after 2 s |
| FT-SVC-10 | Publisher restart | Kill the publisher; after 3 s start a new one | The subscriber reports "source gone" after 2 s; then receives the new publisher's frames without being recreated |
| FT-SVC-11 | Stale resources | `kill -9` a publisher process; run `cleanup_dead_nodes` | The dead node's resources are removed; a new publisher can create the service |
| FT-SVC-12 | Publisher allocation | Publisher created with `initial_max_slice_len = width × height × 3` for 1280 × 720, then publishes 1920 × 1080 | Works (strategy `PowerOfTwo` reallocates) |
