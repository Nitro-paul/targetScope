// doc/testing/overview.md#test-kit
#pragma once

#include <atomic>
#include <chrono>
#include <condition_variable>
#include <deque>
#include <memory>
#include <mutex>
#include <nlohmann/json.hpp>
#include <string>
#include <thread>

#include "targetscope/bus.hpp"

namespace targetscope::testkit {

class EventProbe {
   public:
    EventProbe(const std::string& redis_url, const std::string& program);
    ~EventProbe();
    nlohmann::json expect(const std::string& type, std::chrono::milliseconds timeout);
    void expect_none(std::chrono::milliseconds within);
    std::vector<std::pair<std::string, nlohmann::json>> drain();

   private:
    std::unique_ptr<Bus> bus_;
    std::atomic<bool> cancel_{false};
    std::mutex mutex_;
    std::condition_variable cv_;
    std::deque<std::pair<std::string, nlohmann::json>> events_;
    std::thread thread_;
};

}
