// doc/shared_libraries.md#bootstrap-configuration
#include "targetscope/config.hpp"

#include <cstdlib>

namespace targetscope {

std::filesystem::path expand_home(const std::string& path) {
    const char* home_env = std::getenv("HOME");
    const std::filesystem::path home = home_env ? home_env : "";
    if (path == "~") return home;
    if (path.rfind("~/", 0) == 0) return home / path.substr(2);
    return path;
}

BootstrapConfig BootstrapConfig::from_env() {
    return from_vars([](const std::string& name) -> std::optional<std::string> {
        const char* v = std::getenv(name.c_str());
        if (!v || !*v) return std::nullopt;
        return std::string(v);
    });
}

BootstrapConfig BootstrapConfig::from_vars(const std::function<std::optional<std::string>(const std::string&)>& var) {
    BootstrapConfig c;
    c.data_dir = expand_home(var("TARGETSCOPE_DATA_DIR").value_or("~/.local/share/targetscope"));
    auto db = var("TARGETSCOPE_DB_PATH");
    c.db_path = db ? expand_home(*db) : c.data_dir / "targetscope.db";
    c.redis_url = var("TARGETSCOPE_REDIS_URL").value_or("redis://127.0.0.1:6379/");
    c.log_dir = expand_home(var("TARGETSCOPE_LOG_DIR").value_or("~/.local/state/targetscope/logs"));
    c.log_level = var("TARGETSCOPE_LOG").value_or("info");
    return c;
}

}  // namespace targetscope
