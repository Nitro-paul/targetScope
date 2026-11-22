// doc/programs/camera_reader.md#sources
#include "rtsp_source.hpp"

#include <cstdlib>
#include <stdexcept>

namespace camera_reader {

RtspSource::RtspSource(std::string url) : url_(std::move(url)) {}

SourceInfo RtspSource::open() {
    setenv("OPENCV_FFMPEG_CAPTURE_OPTIONS", "rtsp_transport;tcp", 1);
    if (!capture_.open(url_, cv::CAP_FFMPEG)) throw std::runtime_error("cannot open " + url_);
    SourceInfo info;
    info.width_px = static_cast<int>(capture_.get(cv::CAP_PROP_FRAME_WIDTH));
    info.height_px = static_cast<int>(capture_.get(cv::CAP_PROP_FRAME_HEIGHT));
    info.fps = capture_.get(cv::CAP_PROP_FPS);
    return info;
}

void RtspSource::read(cv::Mat& bgr) {
    if (!capture_.read(bgr) || bgr.empty()) throw std::runtime_error("read failed");
}

void RtspSource::close() { capture_.release(); }

}
