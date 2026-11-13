// doc/programs/camera_reader.md#architecture
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
    std::string path;
    std::string name;
    std::string bus_info;
    std::vector<DeviceFormat> formats;
};

std::vector<CaptureDevice> list_capture_devices();
std::optional<std::string> resolve_device(const std::string& bus_info);

}
