// Network camera through RTSP: cv::VideoCapture(url, cv::CAP_FFMPEG) with
// OPENCV_FFMPEG_CAPTURE_OPTIONS="rtsp_transport;tcp". No controls.
#pragma once

#include <string>

#include <opencv2/videoio.hpp>

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

}  // namespace camera_reader
