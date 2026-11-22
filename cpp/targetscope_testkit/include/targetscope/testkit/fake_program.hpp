// doc/testing/overview.md#test-kit
#pragma once

#include <chrono>
#include <functional>
#include <memory>
#include <mutex>
#include <nlohmann/json.hpp>
#include <optional>
#include <string>
#include <utility>
#include <vector>

#include "targetscope/bus.hpp"

namespace targetscope::testkit {

struct Answer {
    nlohmann::json payload = nlohmann::json::object();
    std::optional<std::string> error_code;
    std::string error_message = "fake error";
    std::chrono::milliseconds delay{0};
    bool no_answer = false;
    std::function<nlohmann::json(const nlohmann::json&)> compute;
};

class FakeProgram {
   public:
    FakeProgram(const std::string& redis_url, std::string name);
    ~FakeProgram();
    void on(const std::string& type, Answer answer);
    std::vector<std::pair<std::string, nlohmann::json>> requests() const;
    std::size_t count(const std::string& type) const;
    void publish(const std::string& type, const nlohmann::json& payload);
    Bus& bus() { return *bus_; }

   private:
    std::unique_ptr<Bus> bus_;
    mutable std::mutex mutex_;
    std::vector<std::pair<std::string, Answer>> answers_;
    std::vector<std::pair<std::string, nlohmann::json>> requests_;
};

}
