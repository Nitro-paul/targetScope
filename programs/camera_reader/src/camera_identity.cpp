// doc/programs/camera_reader.md#camera-identity
#include "camera_identity.hpp"

namespace camera_reader {

std::string camera_key_v4l2(const std::string& bus_info) { return bus_info; }

std::string camera_key_rtsp(const std::string& url) { return url; }

}
