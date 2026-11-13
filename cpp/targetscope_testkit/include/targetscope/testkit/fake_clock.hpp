// doc/testing/overview.md#test-kit
#pragma once

#include <chrono>

#include "targetscope/clock.hpp"

namespace targetscope::testkit {

class FakeClock final : public Clock {
   public:
    explicit FakeClock(std::chrono::system_clock::time_point start);
    void advance(std::chrono::milliseconds d);
    std::chrono::system_clock::time_point now() const override;
    std::chrono::steady_clock::time_point monotonic() const override;
    void sleep(std::chrono::milliseconds d) const override;
};

}
