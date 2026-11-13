// doc/shared_libraries.md#clock-and-failpoints
#pragma once

#include <chrono>
#include <cstdint>

namespace targetscope {

class Clock {
   public:
    virtual ~Clock() = default;
    virtual std::chrono::system_clock::time_point now() const = 0;
    virtual std::chrono::steady_clock::time_point monotonic() const = 0;
    virtual void sleep(std::chrono::milliseconds d) const = 0;
};

class SystemClock final : public Clock {
   public:
    std::chrono::system_clock::time_point now() const override;
    std::chrono::steady_clock::time_point monotonic() const override;
    void sleep(std::chrono::milliseconds d) const override;
};

std::int64_t unix_ms(std::chrono::system_clock::time_point t);

}
