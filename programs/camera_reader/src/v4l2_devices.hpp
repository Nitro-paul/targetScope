// camera_list_devices: /dev/video* with VIDIOC_QUERYCAP (V4L2_CAP_VIDEO_CAPTURE only),
// formats with VIDIOC_ENUM_FMT, VIDIOC_ENUM_FRAMESIZES, VIDIOC_ENUM_FRAMEINTERVALS.
#pragma once

#include <optional>
#include <string>
#include <vector>

namespace camera_reader {

struct DeviceFormat {
    int width_px{};
    int height_px{};
    std::vector<double> fps;
};

struct CaptureDevice {
    std::string path;      // /dev/videoN
    std::string name;
    std::string bus_info;  // camera_key of USB cameras
    std::vector<DeviceFormat> formats;
};

std::vector<CaptureDevice> list_capture_devices();
// Current /dev/videoN of the camera with this bus_info; empty bus_info = lowest N.
std::optional<std::string> resolve_device(const std::string& bus_info);

}  // namespace camera_reader
