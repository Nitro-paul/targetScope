// Reads <program>:events and waits for expected events, with a timeout.
#pragma once

#include <chrono>
#include <string>

#include <nlohmann/json.hpp>

namespace targetscope::testkit {

class EventProbe {
public:
    EventProbe(const std::string& redis_url, const std::string& program);
    // Payload of the next event of this type; throws after the timeout.
    nlohmann::json expect(const std::string& type, std::chrono::milliseconds timeout);
    // Throws if any event arrives within the duration.
    void expect_none(std::chrono::milliseconds within);
};

}  // namespace targetscope::testkit
