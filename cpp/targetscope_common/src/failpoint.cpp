// doc/testing/overview.md#failpoints
// doc/shared_libraries.md#clock-and-failpoints
#include "targetscope/failpoint.hpp"

#include <chrono>
#include <cstdlib>
#include <string>
#include <thread>

namespace targetscope::failpoints {

bool hit(std::string_view name) {
    const char* env = std::getenv("FAILPOINTS");
    if (!env) return false;
    std::string spec(env);
    std::size_t pos = 0;
    while (pos <= spec.size()) {
        std::size_t end = spec.find(';', pos);
        if (end == std::string::npos) end = spec.size();
        std::string item = spec.substr(pos, end - pos);
        pos = end + 1;
        std::size_t eq = item.find('=');
        if (eq == std::string::npos || item.substr(0, eq) != name) continue;
        std::string action = item.substr(eq + 1);
        if (action == "return(abort)") std::abort();
        if (action.rfind("sleep(", 0) == 0) {
            std::this_thread::sleep_for(std::chrono::milliseconds(std::stol(action.substr(6))));
            return false;
        }
        return action.rfind("return", 0) == 0;
    }
    return false;
}

}  // namespace targetscope::failpoints
