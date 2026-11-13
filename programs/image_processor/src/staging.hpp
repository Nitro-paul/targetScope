// Writing images to the staging folder: temporary name ".<name>.part", then rename
// (doc/interfaces.md#staging-folder).
#pragma once

#include <filesystem>
#include <string>

#include <opencv2/core.hpp>

namespace image_processor {

// prefix: raw, target, photo, paper; returns the staged file name "<prefix>_<uuid>.jpg".
std::string write_staged_jpeg(const std::filesystem::path& staging_dir, const std::string& prefix,
                              const cv::Mat& bgr, int quality);

// Loads an image from the staging folder; checks the plain file name. Throws io_error.
cv::Mat read_staged_image(const std::filesystem::path& staging_dir, const std::string& staged_file);

}  // namespace image_processor
