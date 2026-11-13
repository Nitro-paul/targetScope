// Temporary data, image, staging and export folders.
#pragma once

#include <filesystem>
#include <string>
#include <utility>
#include <vector>

namespace targetscope::testkit {

class TempDataDir {
public:
    TempDataDir();
    ~TempDataDir();
    std::filesystem::path data_dir, image_dir, staging_dir, export_dir, log_dir;
    // TARGETSCOPE_* variables for a program under test
    std::vector<std::pair<std::string, std::string>> env(const std::string& redis_url) const;
};

}  // namespace targetscope::testkit
