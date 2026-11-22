// doc/testing/overview.md#test-kit
#include "targetscope/testkit/fake_clock.hpp"

namespace targetscope::testkit {

FakeClock::FakeClock(std::chrono::system_clock::time_point start) : start_(start) {}

void FakeClock::advance(std::chrono::milliseconds d) {
    {
        std::lock_guard lock(mutex_);
        elapsed_ += d;
    }
    cv_.notify_all();
}

std::chrono::system_clock::time_point FakeClock::now() const {
    std::lock_guard lock(mutex_);
    return start_ + elapsed_;
}

std::chrono::steady_clock::time_point FakeClock::monotonic() const {
    std::lock_guard lock(mutex_);
    return mono_start_ + elapsed_;
}

// doc/testing/overview.md#test-kit
void FakeClock::sleep(std::chrono::milliseconds d) const {
    std::unique_lock lock(mutex_);
    const auto target = elapsed_ + d;
    cv_.wait(lock, [&] { return elapsed_ >= target; });
}

}
