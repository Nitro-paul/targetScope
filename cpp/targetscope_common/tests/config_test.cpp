// doc/testing/shared_libraries.md#bootstrap-configuration-unit
#include <gtest/gtest.h>

#include <cstdlib>
#include <map>

#include "targetscope/config.hpp"

using namespace targetscope;

// LIB-CFG-01
TEST(Config, LIB_CFG_01_AllVariablesSet) {
    std::map<std::string, std::string> env{{"TARGETSCOPE_REDIS_URL", "redis://127.0.0.1:7000/"}, {"TARGETSCOPE_DATA_DIR", "/srv/data"},
                                           {"TARGETSCOPE_DB_PATH", "/srv/db.sqlite"},              {"TARGETSCOPE_LOG_DIR", "/srv/logs"},
                                           {"TARGETSCOPE_LOG", "debug"}};
    auto c = BootstrapConfig::from_vars([&](const std::string& n) -> std::optional<std::string> {
        auto it = env.find(n);
        return it == env.end() ? std::nullopt : std::optional(it->second);
    });
    EXPECT_EQ(c.redis_url, "redis://127.0.0.1:7000/");
    EXPECT_EQ(c.data_dir, "/srv/data");
    EXPECT_EQ(c.db_path, "/srv/db.sqlite");
    EXPECT_EQ(c.log_dir, "/srv/logs");
    EXPECT_EQ(c.log_level, "debug");
}

// LIB-CFG-02
TEST(Config, LIB_CFG_02_Defaults) {
    std::filesystem::path home = std::getenv("HOME");
    auto c = BootstrapConfig::from_vars([](const std::string&) { return std::optional<std::string>{}; });
    EXPECT_EQ(c.redis_url, "redis://127.0.0.1:6379/");
    EXPECT_EQ(c.data_dir, home / ".local/share/targetscope");
    EXPECT_EQ(c.db_path, home / ".local/share/targetscope/targetscope.db");
    EXPECT_EQ(c.log_dir, home / ".local/state/targetscope/logs");
    EXPECT_EQ(c.log_level, "info");
}

// LIB-CFG-03
TEST(Config, LIB_CFG_03_RustAndCppAgree) { GTEST_SKIP() << "doc/testing/shared_libraries.md#bootstrap-configuration-unit LIB-CFG-03"; }
