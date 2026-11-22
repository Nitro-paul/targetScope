// doc/testing/overview.md#test-kit
#include "targetscope/testkit/temp_data_dir.hpp"

#include <unistd.h>

#include <cstdlib>

namespace targetscope::testkit {

std::filesystem::path make_temp_dir(const std::string& prefix) {
    const char* base = std::getenv("TMPDIR");
    std::filesystem::path pattern =
        (base ? std::filesystem::path(base) : std::filesystem::temp_directory_path()) / ("ts-" + prefix + "-XXXXXX");
    std::string s = pattern.string();
    if (!mkdtemp(s.data())) throw std::runtime_error("mkdtemp failed");
    return s;
}

TempDataDir::TempDataDir() {
    root = make_temp_dir("data");
    data_dir = root / "data";
    image_dir = data_dir / "images";
    staging_dir = data_dir / "staging";
    export_dir = data_dir / "exports";
    log_dir = root / "logs";
    for (auto* d : {&data_dir, &image_dir, &staging_dir, &export_dir, &log_dir})
        std::filesystem::create_directories(*d);
}

TempDataDir::~TempDataDir() {
    std::error_code ec;
    std::filesystem::remove_all(root, ec);
}

// doc/shared_libraries.md#bootstrap-configuration
std::vector<std::pair<std::string, std::string>> TempDataDir::env(const std::string& redis_url) const {
    return {{"TARGETSCOPE_REDIS_URL", redis_url},
            {"TARGETSCOPE_DATA_DIR", data_dir.string()},
            {"TARGETSCOPE_DB_PATH", (data_dir / "targetscope.db").string()},
            {"TARGETSCOPE_LOG_DIR", log_dir.string()},
            {"TARGETSCOPE_LOG", "info"}};
}

}
