// Drawing of the live frame (doc/frame_transport.md#live-frames): holes gray, counted shots
// highlighted with their number, low-confidence holes marked.
#pragma once

#include <opencv2/core.hpp>

#include "state.hpp"

namespace image_processor {

void draw_live_frame(cv::Mat& transformed_bgr, const LiveState& live, double px_per_mm);

}  // namespace image_processor
