// doc/testing/overview.md#test-kit
#pragma once

#include <nlohmann/json.hpp>
#include <string>

namespace targetscope::testkit {

class TestBus {
   public:
    explicit TestBus(const std::string& redis_url);
    nlohmann::json send(const std::string& program, const std::string& type, const nlohmann::json& payload,
                        const std::string& request_id = {});
    nlohmann::json send_raw(const std::string& program, const std::string& data);
};

}
