// doc/interfaces.md#staging-folder
#pragma once

#include <filesystem>
#include <opencv2/core.hpp>
#include <string>

namespace image_processor {

std::string write_staged_jpeg(const std::filesystem::path& staging_dir, const std::string& prefix, const cv::Mat& bgr,
                              int quality);

cv::Mat read_staged_image(const std::filesystem::path& staging_dir, const std::string& staged_file);

}
