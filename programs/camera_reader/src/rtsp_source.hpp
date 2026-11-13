// doc/programs/camera_reader.md#architecture
#pragma once

#include <opencv2/videoio.hpp>
#include <string>

#include "frame_source.hpp"

namespace camera_reader {

class RtspSource final : public FrameSource {
   public:
    explicit RtspSource(std::string url);
    SourceInfo open() override;
    void read(cv::Mat& bgr) override;
    void close() override;
    bool has_controls() const override { return false; }

   private:
    std::string url_;
    cv::VideoCapture capture_;
};

}
