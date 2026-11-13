// doc/programs/camera_reader.md#architecture
#pragma once

#include <memory>
#include <opencv2/videoio.hpp>
#include <string>

#include "frame_source.hpp"

namespace camera_reader {

struct V4l2Settings {
    std::string bus_info;
    int width_px{};
    int height_px{};
    double fps{};
};

class V4l2Source final : public FrameSource {
   public:
    explicit V4l2Source(V4l2Settings settings);
    SourceInfo open() override;
    void read(cv::Mat& bgr) override;
    void close() override;
    bool has_controls() const override { return true; }
    const std::string& device() const { return device_; }

   private:
    V4l2Settings settings_;
    std::string device_;
    cv::VideoCapture capture_;
};

}
