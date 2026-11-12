// Time source (doc/shared_libraries.md#clock-and-failpoints), same as in Rust.
#pragma once

#include <chrono>

namespace targetscope {

class Clock {
public:
    virtual ~Clock() = default;
    virtual std::chrono::system_clock::time_point now() const = 0;        // wall clock
    virtual std::chrono::steady_clock::time_point monotonic() const = 0;  // durations, timeouts
    virtual void sleep(std::chrono::milliseconds d) const = 0;
};

// The real clock, used in production.
class SystemClock final : public Clock {
public:
    std::chrono::system_clock::time_point now() const override;
    std::chrono::steady_clock::time_point monotonic() const override;
    void sleep(std::chrono::milliseconds d) const override;
};

}  // namespace targetscope
