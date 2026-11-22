#include "targetscope/settings.hpp"

#include <gtest/gtest.h>

#include <atomic>
#include <thread>

#include "targetscope/testkit/fake_program.hpp"
#include "targetscope/testkit/redis_test_server.hpp"

// doc/testing/shared_libraries.md#settings-cache-integration

using namespace targetscope;
using namespace std::chrono_literals;
using nlohmann::json;

namespace {

json setting(const std::string& name, const std::string& value, bool active = true) {
    return json{{"name", name},      {"display_name", name}, {"active", active},       {"value", value},
                {"optional", false}, {"default_value", ""},  {"default_active", true}, {"value_rule", "text"}};
}

struct Fixture {
    testkit::RedisTestServer redis;
    testkit::FakeProgram db{redis.url(), "database_handler"};
    std::mutex m;
    std::map<std::string, json> settings;
    Fixture() {
        db.on("read_settings", {.compute = [this](const json& p) {
                  std::lock_guard l(m);
                  json list = json::array();
                  for (auto& [k, v] : settings) {
                      bool match = true;
                      for (auto& c : p["filter"]["conditions"])
                          if (c["field"] == "name" && c["value"] != k) match = false;
                      if (match) list.push_back(v);
                  }
                  return json{{"settings", list}};
              }});
    }
    void put(const std::string& n, const std::string& v, bool active = true) {
        std::lock_guard l(m);
        settings[n] = setting(n, v, active);
    }
    void changed(const std::string& n) {
        db.publish("data_changed", json{{"table", "settings"}, {"key", n}, {"action", "updated"}});
    }
    Bus bus{redis.url(), "me"};
};

template <class F>
bool eventually(F f, std::chrono::milliseconds t = 2000ms) {
    auto end = std::chrono::steady_clock::now() + t;
    while (std::chrono::steady_clock::now() < end) {
        if (f()) return true;
        std::this_thread::sleep_for(20ms);
    }
    return f();
}

}

// LIB-SET-01
TEST(Settings, LIB_SET_01_Load) {
    Fixture f;
    f.put("camera_fps", "30");
    f.put("camera_source", "v4l2");
    SettingsCache c(f.bus);
    EXPECT_EQ(c.get("camera_fps"), "30");
    EXPECT_EQ(c.get("camera_source"), "v4l2");
    EXPECT_FALSE(c.get("unknown"));
}

// LIB-SET-02
TEST(Settings, LIB_SET_02_UnsetOptionalSetting) {
    Fixture f;
    f.put("export_dir", "~/x", false);
    SettingsCache c(f.bus);
    EXPECT_FALSE(c.get("export_dir"));
}

// LIB-SET-03
TEST(Settings, LIB_SET_03_ParsedValues) {
    Fixture f;
    f.put("camera_fps", "30");
    SettingsCache c(f.bus);
    EXPECT_EQ(c.get_parsed<unsigned>("camera_fps"), 30u);
    f.put("camera_fps", "abc");
    f.changed("camera_fps");
    ASSERT_TRUE(eventually([&] { return c.get("camera_fps") == "abc"; }));
    EXPECT_FALSE(c.get_parsed<unsigned>("camera_fps"));
}

// LIB-SET-04
TEST(Settings, LIB_SET_04_Paths) {
    Fixture f;
    f.put("a", "~/TargetScope/exports");
    f.put("b", "/srv/exports");
    SettingsCache c(f.bus);
    EXPECT_EQ(c.get_path("a"), std::filesystem::path(std::getenv("HOME")) / "TargetScope/exports");
    EXPECT_EQ(c.get_path("b"), std::filesystem::path("/srv/exports"));
}

// LIB-SET-05
TEST(Settings, LIB_SET_05_ChangeEvent) {
    Fixture f;
    f.put("camera_fps", "30");
    SettingsCache c(f.bus);
    f.put("camera_fps", "15");
    f.changed("camera_fps");
    EXPECT_TRUE(eventually([&] { return c.get("camera_fps") == "15"; }, 1000ms));
}

// LIB-SET-06
TEST(Settings, LIB_SET_06_NoGapAtStartup) {
    GTEST_SKIP() << "needs the failpoint settings.after_note_id (test build with TARGETSCOPE_FAILPOINTS)";
}

// LIB-SET-07
TEST(Settings, LIB_SET_07_Watch) {
    Fixture f;
    f.put("camera_fps", "30");
    f.put("live_feed_fps", "15");
    SettingsCache c(f.bus);
    std::atomic<int> calls{0};
    c.watch({"camera_fps"}, [&] { ++calls; });
    f.put("camera_fps", "60");
    f.changed("camera_fps");
    ASSERT_TRUE(eventually([&] { return calls == 1; }));
    f.put("live_feed_fps", "10");
    f.changed("live_feed_fps");
    std::this_thread::sleep_for(400ms);
    EXPECT_EQ(calls, 1);
}

// LIB-SET-08
TEST(Settings, LIB_SET_08_ReloadEverything) {
    Fixture f;
    f.put("a", "1");
    SettingsCache c(f.bus);
    f.put("a", "2");
    f.put("b", "3");
    f.db.publish("data_changed", json{{"table", "all"}, {"key", nullptr}, {"action", "reload"}});
    EXPECT_TRUE(eventually([&] { return c.get("a") == "2" && c.get("b") == "3"; }, 1000ms));
}
