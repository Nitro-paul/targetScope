// Coordinate systems (doc/programs/image_processor.md#coordinate-systems):
// raw image (px) -> paper (mm, origin top-left, y down) -> shot (polar around the ring center, y up).
#pragma once

#include <opencv2/core.hpp>

#include "targetscope/geometry.hpp"
#include "targetscope/model.hpp"

namespace image_processor {

// Homography mapping the corners to (0,0), (W,0), (W,H), (0,H), W, H = paper size × px_per_mm.
cv::Mat paper_homography(const targetscope::model::Corners& corners, double paper_width_mm,
                         double paper_height_mm, double px_per_mm);

// x = px - center_x_mm, y = center_y_mm - py, then polar with the shared geometry functions.
targetscope::geometry::Polar paper_to_shot(const cv::Point2d& paper_mm, const targetscope::model::TargetType& t);
cv::Point2d shot_to_paper(const targetscope::geometry::Polar& shot, const targetscope::model::TargetType& t);

}  // namespace image_processor
