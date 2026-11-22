// doc/testing/overview.md#test-kit
#include "targetscope/testkit/test_bus.hpp"

#include <sw/redis++/redis++.h>

namespace targetscope::testkit {

TestBus::TestBus(const std::string& redis_url)
    : url_(redis_url), bus_(std::make_unique<Bus>(redis_url, "test_" + new_uuid().substr(0, 8))) {}

TestBus::~TestBus() = default;

nlohmann::json TestBus::send(const std::string& program, const std::string& type, const nlohmann::json& payload,
                             const std::string& request_id, std::chrono::seconds timeout) {
    std::string stream = program + ":requests";
    return bus_->request_raw(stream.c_str(), type.c_str(), payload, timeout, request_id);
}

void TestBus::send_raw(const std::string& program, const std::string& data) {
    std::string rest = url_.substr(url_.find("://") + 3);
    sw::redis::ConnectionOptions o;
    o.host = rest.substr(0, rest.find(':'));
    o.port = std::stoi(rest.substr(rest.find(':') + 1));
    sw::redis::Redis r(o);
    std::vector<std::pair<std::string, std::string>> fields{{"data", data}};
    r.xadd(program + ":requests", "*", fields.begin(), fields.end());
}

}
