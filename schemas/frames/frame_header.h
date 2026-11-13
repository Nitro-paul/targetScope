/* schemas/frames/frame_header.h
 *
 * Header of every iceoryx2 frame sample on targetscope/frames/raw and targetscope/frames/live.
 * This file is the contract for every language (doc/frame_transport.md#frame-format):
 * C++ uses the struct directly, Rust mirrors it as a #[repr(C)] struct.
 * Both sides check size (48) and field offsets at compile time.
 */
#ifndef TARGETSCOPE_FRAME_HEADER_H
#define TARGETSCOPE_FRAME_HEADER_H

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

#endif /* TARGETSCOPE_FRAME_HEADER_H */
