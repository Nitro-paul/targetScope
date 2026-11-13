// doc/shared_libraries.md#bootstrap-configuration
#pragma once

#include <filesystem>
#include <functional>
#include <optional>
#include <string>

namespace targetscope {

std::filesystem::path expand_home(const std::string& path);

struct BootstrapConfig {
    std::string redis_url;
    std::filesystem::path data_dir;
    std::filesystem::path db_path;
    std::filesystem::path log_dir;
    std::string log_level;

    static BootstrapConfig from_env();
    static BootstrapConfig from_vars(const std::function<std::optional<std::string>(const std::string&)>& var);
};

}
