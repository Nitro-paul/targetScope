// doc/testing/overview.md#test-kit
#pragma once

#include <memory>
#include <nlohmann/json.hpp>
#include <string>

#include "targetscope/bus.hpp"

namespace targetscope::testkit {

class TestBus {
   public:
    explicit TestBus(const std::string& redis_url);
    ~TestBus();

    nlohmann::json send(const std::string& program, const std::string& type, const nlohmann::json& payload,
                        const std::string& request_id = {}, std::chrono::seconds timeout = std::chrono::seconds(10));

    void send_raw(const std::string& program, const std::string& data);

    Bus& bus() { return *bus_; }

   private:
    std::string url_;
    std::unique_ptr<Bus> bus_;
};

}
