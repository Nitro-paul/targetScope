// doc/testing/overview.md#test-kit
#pragma once

#include <chrono>
#include <nlohmann/json.hpp>
#include <optional>
#include <string>
#include <utility>
#include <vector>

namespace targetscope::testkit {

struct Answer {
    nlohmann::json payload = nlohmann::json::object();
    std::optional<std::string> error_code;
    std::chrono::milliseconds delay{0};
    bool no_answer = false;
};

class FakeProgram {
   public:
    FakeProgram(const std::string& redis_url, std::string name);
    ~FakeProgram();
    void on(const std::string& type, Answer answer);
    std::vector<std::pair<std::string, nlohmann::json>> requests() const;
};

}
