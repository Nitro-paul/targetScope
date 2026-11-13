// doc/testing/overview.md#test-kit
#pragma once

#include <chrono>
#include <nlohmann/json.hpp>
#include <string>

namespace targetscope::testkit {

class EventProbe {
   public:
    EventProbe(const std::string& redis_url, const std::string& program);
    nlohmann::json expect(const std::string& type, std::chrono::milliseconds timeout);
    void expect_none(std::chrono::milliseconds within);
};

}
