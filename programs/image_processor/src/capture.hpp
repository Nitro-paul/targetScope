// Capture mode from the camera (doc/programs/image_processor.md#capture-mode).
#pragma once

#include <filesystem>
#include <string>

#include "state.hpp"

namespace image_processor {

// Warps last_good_raw with its homography at capture_long_side_px; returns "target_<uuid>.jpg".
std::string capture_target_image(const LiveState& live, const std::filesystem::path& staging_dir,
                                 int capture_long_side_px);

// newest_raw, lens-corrected if a calibration applies; returns {"raw_<uuid>.jpg", lens_corrected}.
std::pair<std::string, bool> capture_raw_image(const ProcessorState& state, const std::filesystem::path& staging_dir);

}  // namespace image_processor
