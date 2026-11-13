// doc/programs/image_processor.md#finding-the-paper
#pragma once

#include <opencv2/core.hpp>
#include <optional>

#include "targetscope/model.hpp"

namespace image_processor {

std::optional<targetscope::model::Corners> find_paper(const cv::Mat& bgr,
                                                      const targetscope::model::TargetType* target_type = nullptr);

std::optional<targetscope::model::Corners> find_paper_near(const cv::Mat& bgr, const targetscope::model::Corners& known,
                                                           const targetscope::model::TargetType& t);

}
