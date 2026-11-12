// Camera controls through a separate fd on the same device (doc/programs/camera_reader.md#camera-controls).
#pragma once

#include <cstdint>
#include <map>
#include <string>
#include <vector>

#include "targetscope/model.hpp"

namespace camera_reader {

class CameraControls {
public:
    explicit CameraControls(const std::string& device);  // open(device, O_RDWR)
    ~CameraControls();

    // VIDIOC_QUERY_EXT_CTRL with V4L2_CTRL_FLAG_NEXT_CTRL; skip classes and disabled controls;
    // menus with VIDIOC_QUERYMENU; values with VIDIOC_G_EXT_CTRLS; inactive from V4L2_CTRL_FLAG_INACTIVE.
    std::vector<targetscope::model::CameraControl> list() const;

    // Checks range and step, VIDIOC_S_EXT_CTRLS; returns all controls read again.
    std::vector<targetscope::model::CameraControl> set(std::int64_t id, std::int64_t value);

    // Stored values of this camera, auto modes first; rejected values only logged.
    void apply(const std::map<std::int64_t, std::int64_t>& stored);

    // Every control to its default, auto modes first (reset of camera_controls).
    void reset_to_defaults();

private:
    int fd_ = -1;
};

}  // namespace camera_reader
