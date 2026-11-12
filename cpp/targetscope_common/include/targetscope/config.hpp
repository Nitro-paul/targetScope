// Bootstrap configuration from the environment (doc/shared_libraries.md#bootstrap-configuration).
#pragma once

#include <filesystem>
#include <string>

namespace targetscope {

struct BootstrapConfig {
    std::string redis_url;            // TARGETSCOPE_REDIS_URL, default redis://127.0.0.1:6379/
    std::filesystem::path data_dir;   // TARGETSCOPE_DATA_DIR, default ~/.local/share/targetscope
    std::filesystem::path db_path;    // TARGETSCOPE_DB_PATH, default <data_dir>/targetscope.db
    std::filesystem::path log_dir;    // TARGETSCOPE_LOG_DIR, default ~/.local/state/targetscope/logs
    std::string log_level;            // TARGETSCOPE_LOG, default "info"

    static BootstrapConfig from_env();
};

}  // namespace targetscope
