// USB camera through V4L2.
#pragma once

#include <memory>
#include <string>

#include <opencv2/videoio.hpp>

#include "frame_source.hpp"

namespace camera_reader {

struct V4l2Settings {
    std::string bus_info;  // setting camera_device; empty = capture device with the lowest N
    int width_px{};
    int height_px{};
    double fps{};
};

class V4l2Source final : public FrameSource {
public:
    explicit V4l2Source(V4l2Settings settings);
    // Resolves bus_info to the current /dev/videoN (VIDIOC_QUERYCAP of every capture device);
    // cv::VideoCapture(device, cv::CAP_V4L2); MJPG for high resolutions at full frame rate;
    // CAP_PROP_FRAME_WIDTH/HEIGHT/FPS and CAP_PROP_BUFFERSIZE = 1.
    SourceInfo open() override;
    void read(cv::Mat& bgr) override;
    void close() override;
    bool has_controls() const override { return true; }
    const std::string& device() const { return device_; }  // current /dev/videoN
private:
    V4l2Settings settings_;
    std::string device_;
    cv::VideoCapture capture_;
};

}  // namespace camera_reader
