// doc/testing/overview.md#test-kit
#include "targetscope/testkit/event_probe.hpp"

#include <stdexcept>

namespace targetscope::testkit {

EventProbe::EventProbe(const std::string& redis_url, const std::string& program)
    : bus_(std::make_unique<Bus>(redis_url, "probe_" + new_uuid().substr(0, 8))) {
    std::string after = bus_->newest_event_id(program);
    thread_ = std::thread([this, program, after] {
        bus_->read_events(
            program, after,
            [this](const std::string& type, const nlohmann::json& payload) {
                {
                    std::lock_guard lock(mutex_);
                    events_.emplace_back(type, payload);
                }
                cv_.notify_all();
            },
            &cancel_);
    });
}

EventProbe::~EventProbe() {
    cancel_ = true;
    if (thread_.joinable()) thread_.join();
}

nlohmann::json EventProbe::expect(const std::string& type, std::chrono::milliseconds timeout) {
    std::unique_lock lock(mutex_);
    auto deadline = std::chrono::steady_clock::now() + timeout;
    for (;;) {
        for (auto it = events_.begin(); it != events_.end(); ++it)
            if (it->first == type) {
                auto payload = it->second;
                events_.erase(it);
                return payload;
            }
        if (cv_.wait_until(lock, deadline) == std::cv_status::timeout) {
            for (auto it = events_.begin(); it != events_.end(); ++it)
                if (it->first == type) {
                    auto payload = it->second;
                    events_.erase(it);
                    return payload;
                }
            throw std::runtime_error("event " + type + " did not arrive");
        }
    }
}

void EventProbe::expect_none(std::chrono::milliseconds within) {
    std::this_thread::sleep_for(within);
    std::lock_guard lock(mutex_);
    if (!events_.empty()) throw std::runtime_error("unexpected event " + events_.front().first);
}

std::vector<std::pair<std::string, nlohmann::json>> EventProbe::drain() {
    std::lock_guard lock(mutex_);
    std::vector<std::pair<std::string, nlohmann::json>> out(events_.begin(), events_.end());
    events_.clear();
    return out;
}

}
