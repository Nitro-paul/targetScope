// doc/testing/overview.md#test-kit
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
    std::vector<std::pair<std::string, std::string>> env(const std::string& redis_url) const;
};

}
