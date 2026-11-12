// Finding the paper (doc/programs/image_processor.md#finding-the-paper): Otsu, contours,
// convex quadrilaterals >= 10 % of the image, cornerSubPix, aspect ratio check, ring center check.
#pragma once

#include <optional>

#include <opencv2/core.hpp>

#include "targetscope/model.hpp"

namespace image_processor {

// target_type: optional; enables the aspect ratio and ring center checks (and corner rotation).
std::optional<targetscope::model::Corners> find_paper(
    const cv::Mat& bgr, const targetscope::model::TargetType* target_type = nullptr);

// Supervision: search again near the known corners.
std::optional<targetscope::model::Corners> find_paper_near(
    const cv::Mat& bgr, const targetscope::model::Corners& known, const targetscope::model::TargetType& t);

}  // namespace image_processor
