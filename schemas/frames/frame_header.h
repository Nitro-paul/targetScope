/* doc/frame_transport.md#frame-format */
#ifndef TARGETSCOPE_FRAME_HEADER_H
#define TARGETSCOPE_FRAME_HEADER_H

#include <stdint.h>

#define TARGETSCOPE_FRAME_HEADER_TYPE_NAME "TargetScopeFrameHeader"

enum TargetScopePixelFormat { TARGETSCOPE_PIXEL_BGR8 = 1 };

typedef struct {
    uint64_t seq;
    uint64_t timestamp_ns;
    uint64_t source_seq;
    uint32_t width;
    uint32_t height;
    uint32_t stride;
    uint32_t format;
    double px_per_mm;
} TargetScopeFrameHeader;

#endif
