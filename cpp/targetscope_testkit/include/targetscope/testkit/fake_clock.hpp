// doc/testing/overview.md#test-kit
#pragma once

#include <chrono>
#include <condition_variable>
#include <mutex>

#include "targetscope/clock.hpp"

namespace targetscope::testkit {

class FakeClock final : public Clock {
   public:
    explicit FakeClock(std::chrono::system_clock::time_point start);
    void advance(std::chrono::milliseconds d);
    std::chrono::system_clock::time_point now() const override;
    std::chrono::steady_clock::time_point monotonic() const override;
    void sleep(std::chrono::milliseconds d) const override;

   private:
    mutable std::mutex mutex_;
    mutable std::condition_variable cv_;
    std::chrono::milliseconds elapsed_{0};
    std::chrono::system_clock::time_point start_;
    std::chrono::steady_clock::time_point mono_start_ = std::chrono::steady_clock::now();
};

}
