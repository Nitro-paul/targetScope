// doc/programs/camera_reader.md#camera-controls
#pragma once

#include <cstdint>
#include <map>
#include <string>
#include <vector>

#include "targetscope/model.hpp"

namespace camera_reader {

class CameraControls {
   public:
    explicit CameraControls(const std::string& device);
    ~CameraControls();

    std::vector<targetscope::model::CameraControl> list() const;

    std::vector<targetscope::model::CameraControl> set(std::int64_t id, std::int64_t value);

    void apply(const std::map<std::int64_t, std::int64_t>& stored);

    void reset_to_defaults();

   private:
    int fd_ = -1;
};

}
