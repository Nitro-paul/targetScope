// doc/programs/camera_reader.md#sources
#pragma once

#include <opencv2/core.hpp>

namespace camera_reader {

struct SourceInfo {
    int width_px{};
    int height_px{};
    double fps{};
};

class FrameSource {
   public:
    virtual ~FrameSource() = default;
    virtual SourceInfo open() = 0;
    virtual void read(cv::Mat& bgr) = 0;
    virtual void close() = 0;
    virtual bool has_controls() const = 0;
};

}
