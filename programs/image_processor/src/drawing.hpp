// doc/frame_transport.md#live-frames
#pragma once

#include <opencv2/core.hpp>

#include "state.hpp"

namespace image_processor {

void draw_live_frame(cv::Mat& transformed_bgr, const LiveState& live, double px_per_mm);

}
