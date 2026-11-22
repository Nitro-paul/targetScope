// doc/programs/camera_reader.md#sources
#include "v4l2_source.hpp"

#include <spdlog/spdlog.h>

#include <opencv2/videoio.hpp>
#include <stdexcept>

#include "v4l2_devices.hpp"

namespace camera_reader {

V4l2Source::V4l2Source(V4l2Settings settings) : settings_(std::move(settings)) {}

SourceInfo V4l2Source::open() {
    auto resolved = resolve_camera(settings_.bus_info);
    if (!resolved)
        throw std::runtime_error(settings_.bus_info.empty() ? "no USB camera found"
                                                            : "USB camera " + settings_.bus_info + " not found");
    device_ = resolved->path;
    bus_info_ = resolved->bus_info;
    if (!capture_.open(device_, cv::CAP_V4L2)) throw std::runtime_error("cannot open " + device_);
    if (settings_.width_px >= 1280) capture_.set(cv::CAP_PROP_FOURCC, cv::VideoWriter::fourcc('M', 'J', 'P', 'G'));
    capture_.set(cv::CAP_PROP_FRAME_WIDTH, settings_.width_px);
    capture_.set(cv::CAP_PROP_FRAME_HEIGHT, settings_.height_px);
    capture_.set(cv::CAP_PROP_FPS, settings_.fps);
    capture_.set(cv::CAP_PROP_BUFFERSIZE, 1);
    SourceInfo info;
    info.width_px = static_cast<int>(capture_.get(cv::CAP_PROP_FRAME_WIDTH));
    info.height_px = static_cast<int>(capture_.get(cv::CAP_PROP_FRAME_HEIGHT));
    info.fps = capture_.get(cv::CAP_PROP_FPS);
    if (info.width_px != settings_.width_px || info.height_px != settings_.height_px)
        spdlog::warn("camera delivers {}x{} instead of the requested {}x{}", info.width_px, info.height_px,
                     settings_.width_px, settings_.height_px);
    return info;
}

void V4l2Source::read(cv::Mat& bgr) {
    if (!capture_.read(bgr) || bgr.empty()) throw std::runtime_error("read failed");
}

void V4l2Source::close() { capture_.release(); }

}
