// doc/programs/image_processor.md#coordinate-systems
#pragma once

#include <opencv2/core.hpp>

#include "targetscope/geometry.hpp"
#include "targetscope/model.hpp"

namespace image_processor {

cv::Mat paper_homography(const targetscope::model::Corners& corners, double paper_width_mm, double paper_height_mm,
                         double px_per_mm);

targetscope::geometry::Polar paper_to_shot(const cv::Point2d& paper_mm, const targetscope::model::TargetType& t);
cv::Point2d shot_to_paper(const targetscope::geometry::Polar& shot, const targetscope::model::TargetType& t);

}
