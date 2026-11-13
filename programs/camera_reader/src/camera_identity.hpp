// doc/programs/camera_reader.md#camera-identity
#pragma once

#include <string>

namespace camera_reader {

std::string camera_key_v4l2(const std::string& bus_info);
std::string camera_key_rtsp(const std::string& url);

}
