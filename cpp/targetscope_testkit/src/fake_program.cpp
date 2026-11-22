// doc/testing/overview.md#test-kit
#include "targetscope/testkit/fake_program.hpp"

#include <thread>

namespace targetscope::testkit {

FakeProgram::FakeProgram(const std::string& redis_url, std::string name)
    : bus_(std::make_unique<Bus>(redis_url, std::move(name))) {
    bus_->serve([this](const std::string& type, const nlohmann::json& payload) -> nlohmann::json {
        Answer answer;
        {
            std::lock_guard lock(mutex_);
            requests_.emplace_back(type, payload);
            for (auto it = answers_.rbegin(); it != answers_.rend(); ++it)
                if (it->first == type) {
                    answer = it->second;
                    break;
                }
        }
        if (answer.delay.count() > 0) std::this_thread::sleep_for(answer.delay);
        if (answer.no_answer) throw NoResponse{};
        if (answer.error_code)
            throw ServiceError(error_code_from(*answer.error_code).value_or(ErrorCode::InvalidMessage),
                               answer.error_message);
        return answer.compute ? answer.compute(payload) : answer.payload;
    });
}

FakeProgram::~FakeProgram() { bus_->stop(); }

void FakeProgram::on(const std::string& type, Answer answer) {
    std::lock_guard lock(mutex_);
    answers_.emplace_back(type, std::move(answer));
}

std::vector<std::pair<std::string, nlohmann::json>> FakeProgram::requests() const {
    std::lock_guard lock(mutex_);
    return requests_;
}

std::size_t FakeProgram::count(const std::string& type) const {
    std::lock_guard lock(mutex_);
    std::size_t n = 0;
    for (auto& r : requests_) n += r.first == type;
    return n;
}

void FakeProgram::publish(const std::string& type, const nlohmann::json& payload) { bus_->publish(type, payload); }

}
