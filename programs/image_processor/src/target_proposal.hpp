// Ring proposal for a new target type from a transformed photo
// (doc/programs/image_processor.md#target-type-from-a-photo).
#pragma once

#include <optional>

#include <opencv2/core.hpp>

namespace image_processor {

struct TargetProposal {
    std::optional<double> center_x_mm, center_y_mm;
    std::optional<double> inner_white_radius_mm;
    std::optional<double> black_radius_mm;
    std::optional<double> outer_radius_mm;  // outermost dark peak of the radial intensity profile
};

TargetProposal propose_rings(const cv::Mat& transformed_bgr, double px_per_mm);

}  // namespace image_processor
