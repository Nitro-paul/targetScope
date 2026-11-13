// doc/shared_libraries.md#clock-and-failpoints
#include "targetscope/clock.hpp"

#include <thread>

namespace targetscope {

std::chrono::system_clock::time_point SystemClock::now() const { return std::chrono::system_clock::now(); }

std::chrono::steady_clock::time_point SystemClock::monotonic() const { return std::chrono::steady_clock::now(); }

void SystemClock::sleep(std::chrono::milliseconds d) const { std::this_thread::sleep_for(d); }

std::int64_t unix_ms(std::chrono::system_clock::time_point t) {
    return std::chrono::duration_cast<std::chrono::milliseconds>(t.time_since_epoch()).count();
}

}  // namespace targetscope
