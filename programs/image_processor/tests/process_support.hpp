// doc/testing/image_processor.md#setup
#pragma once

#include <map>
#include <memory>
#include <mutex>
#include <opencv2/imgproc.hpp>

#include "synthetic.hpp"
#include "targetscope/frames.hpp"
#include "targetscope/testkit/event_probe.hpp"
#include "targetscope/testkit/fake_program.hpp"
#include "targetscope/testkit/frame_feeder.hpp"
#include "targetscope/testkit/process.hpp"
#include "targetscope/testkit/redis_test_server.hpp"
#include "targetscope/testkit/temp_data_dir.hpp"
#include "targetscope/testkit/test_bus.hpp"

namespace image_processor::testing {

using nlohmann::json;
using namespace std::chrono_literals;

inline json make_setting(const std::string& name, const std::string& value, bool active = true) {
    return json{{"name", name},      {"display_name", name}, {"active", active},       {"value", value},
                {"optional", false}, {"default_value", ""},  {"default_active", true}, {"value_rule", "text"}};
}

struct Camera {
    std::mutex mutex;
    cv::Mat frame;
    void set(const cv::Mat& f) {
        std::lock_guard lock(mutex);
        frame = f.clone();
    }
    targetscope::testkit::FeedFrame next() {
        std::lock_guard lock(mutex);
        targetscope::testkit::FeedFrame out;
        out.width = frame.cols;
        out.height = frame.rows;
        out.bgr.assign(frame.data, frame.data + frame.total() * 3);
        return out;
    }
};

struct ProcessEnv {
    targetscope::testkit::RedisTestServer redis;
    targetscope::testkit::TempDataDir dirs;
    targetscope::testkit::FakeProgram db{redis.url(), "database_handler"};
    targetscope::testkit::FakeProgram camera_reader{redis.url(), "camera_reader"};
    targetscope::testkit::TestBus test{redis.url()};
    std::mutex mutex;
    std::map<std::string, json> settings;
    std::vector<json> saved;
    Camera camera;
    std::unique_ptr<targetscope::testkit::FrameFeeder> feeder;
    std::unique_ptr<targetscope::testkit::EventProbe> events;
    std::unique_ptr<targetscope::testkit::ProgramProcess> process;

    ProcessEnv() {
        put("live_long_side_px", "1200");
        put("capture_long_side_px", "2400");
        put("detection_confirm_frames", "5");
        put("staging_dir", dirs.staging_dir.string());
        put("lens_calibrations", "{}");
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
        camera_reader.on("camera_status", {.payload = json{{"state", "connected"},
                                                           {"camera_source", "v4l2"},
                                                           {"device", "/dev/video0"},
                                                           {"camera_key", "fake-cam"},
                                                           {"width_px", 1920},
                                                           {"height_px", 1080},
                                                           {"fps", 30.0},
                                                           {"frames_published", 0}}});
    }

    void put(const std::string& name, const std::string& value, bool active = true) {
        std::lock_guard l(mutex);
        settings[name] = make_setting(name, value, active);
    }

    void start(double fps = 30) {
        events = std::make_unique<targetscope::testkit::EventProbe>(redis.url(), "image_processor");
        process = std::make_unique<targetscope::testkit::ProgramProcess>(IMAGE_PROCESSOR_BIN, dirs.env(redis.url()));
        auto deadline = std::chrono::steady_clock::now() + 10s;
        for (;;) {
            try {
                test.send("image_processor", "ping", json::object(), {}, 1s);
                break;
            } catch (const targetscope::BusError&) {
                if (std::chrono::steady_clock::now() > deadline) throw;
            }
        }
        start_feeding(fps);
    }

    void start_feeding(double fps = 30) {
        feeder = std::make_unique<targetscope::testkit::FrameFeeder>("targetscope/frames/raw", fps,
                                                                     [this](std::uint64_t) { return camera.next(); });
    }

    json call(const std::string& type, const json& payload = json::object()) {
        return test.send("image_processor", type, payload);
    }

    json live_start(const Scene& s) {
        return call("live_start", {{"target_type", json(s.target)}, {"bullet_diameter_mm", s.bullet_mm}});
    }

    bool wait_state(const std::string& state, std::chrono::milliseconds t = 5000ms) {
        auto end = std::chrono::steady_clock::now() + t;
        while (std::chrono::steady_clock::now() < end) {
            if (call("live_calibration_status")["state"] == state) return true;
            std::this_thread::sleep_for(100ms);
        }
        return false;
    }
};

}
