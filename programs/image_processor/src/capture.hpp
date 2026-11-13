// doc/programs/image_processor.md#capture-mode
#pragma once

#include <filesystem>
#include <string>

#include "state.hpp"

namespace image_processor {

std::string capture_target_image(const LiveState& live, const std::filesystem::path& staging_dir,
                                 int capture_long_side_px);

std::pair<std::string, bool> capture_raw_image(const ProcessorState& state, const std::filesystem::path& staging_dir);

}
