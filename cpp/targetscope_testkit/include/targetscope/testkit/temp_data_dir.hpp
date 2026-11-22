// doc/testing/overview.md#test-kit
#pragma once

#include <filesystem>
#include <string>
#include <utility>
#include <vector>

namespace targetscope::testkit {

std::filesystem::path make_temp_dir(const std::string& prefix);

class TempDataDir {
   public:
    TempDataDir();
    ~TempDataDir();
    TempDataDir(const TempDataDir&) = delete;
    TempDataDir& operator=(const TempDataDir&) = delete;
    std::filesystem::path root, data_dir, image_dir, staging_dir, export_dir, log_dir;
    std::vector<std::pair<std::string, std::string>> env(const std::string& redis_url) const;
};

}
