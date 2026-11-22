// doc/testing/camera_reader.md#setup
#pragma once

#include <map>
#include <memory>
#include <mutex>

#include "targetscope/frames.hpp"
#include "targetscope/testkit/event_probe.hpp"
#include "targetscope/testkit/fake_program.hpp"
#include "targetscope/testkit/process.hpp"
#include "targetscope/testkit/redis_test_server.hpp"
#include "targetscope/testkit/temp_data_dir.hpp"
#include "targetscope/testkit/test_bus.hpp"

namespace camera_reader::testing {

using nlohmann::json;
using namespace std::chrono_literals;

inline json make_setting(const std::string& name, const std::string& value, bool active = true) {
    return json{{"name", name},      {"display_name", name}, {"active", active},       {"value", value},
                {"optional", false}, {"default_value", ""},  {"default_active", true}, {"value_rule", "text"}};
}

// doc/testing/camera_reader.md#setup
struct ProcessEnv {
    targetscope::testkit::RedisTestServer redis;
    targetscope::testkit::TempDataDir dirs;
    targetscope::testkit::FakeProgram db{redis.url(), "database_handler"};
    targetscope::testkit::TestBus test{redis.url()};
    std::mutex mutex;
    std::map<std::string, json> settings;
    std::vector<json> saved;
    std::unique_ptr<targetscope::testkit::EventProbe> events;
    std::unique_ptr<targetscope::testkit::ProgramProcess> process;

    ProcessEnv(const std::string& camera_source, int w = 640, int h = 480, int fps = 30) {
        put("camera_source", camera_source);
        put("camera_device", "");
        put("camera_rtsp_url", "", false);
        put("camera_width_px", std::to_string(w));
        put("camera_height_px", std::to_string(h));
        put("camera_fps", std::to_string(fps));
        put("camera_controls", "{}");
        db.on("read_settings", {.compute = [this](const json& p) {
                  std::lock_guard l(mutex);
                  json list = json::array();
                  for (auto& [k, v] : settings) {
                      bool match = true;
                      for (auto& c : p["filter"]["conditions"])
                          if (c["field"] == "name" && c["value"] != k) match = false;
                      if (match) list.push_back(v);
                  }
                  return json{{"settings", list}};
              }});
        db.on("save_setting", {.compute = [this](const json& p) {
                  {
                      std::lock_guard l(mutex);
                      settings[p["name"]] = p;
                      saved.push_back(p);
                  }
                  db.publish("data_changed", json{{"table", "settings"}, {"key", p["name"]}, {"action", "updated"}});
                  return json{{"name", p["name"]}};
              }});
    }

    void put(const std::string& name, const std::string& value, bool active = true) {
        std::lock_guard l(mutex);
        settings[name] = make_setting(name, value, active);
    }

    void change(const std::string& name, const std::string& value, bool active = true) {
        put(name, value, active);
        db.publish("data_changed", json{{"table", "settings"}, {"key", name}, {"action", "updated"}});
    }

    void start() {
        events = std::make_unique<targetscope::testkit::EventProbe>(redis.url(), "camera_reader");
        process = std::make_unique<targetscope::testkit::ProgramProcess>(CAMERA_READER_BIN, dirs.env(redis.url()));
        auto deadline = std::chrono::steady_clock::now() + 10s;
        for (;;) {
            try {
                test.send("camera_reader", "ping", json::object(), {}, 1s);
                return;
            } catch (const targetscope::BusError&) {
                if (std::chrono::steady_clock::now() > deadline) throw;
            }
        }
    }

    json call(const std::string& type, const json& payload = json::object()) {
        return test.send("camera_reader", type, payload);
    }

    json status() { return call("camera_status"); }

    bool wait_state(const std::string& state, std::chrono::milliseconds t = 8000ms) {
        auto end = std::chrono::steady_clock::now() + t;
        while (std::chrono::steady_clock::now() < end) {
            if (status()["state"] == state) return true;
            std::this_thread::sleep_for(100ms);
        }
        return false;
    }
};

inline bool has_real_camera() { return !std::filesystem::exists("/dev/video0") ? false : true; }

}
