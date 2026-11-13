// doc/testing/overview.md#failpoints
// doc/shared_libraries.md#clock-and-failpoints
#pragma once

#include <string_view>

namespace targetscope::failpoints {

bool hit(std::string_view name);

}  // namespace targetscope::failpoints

#if defined(TARGETSCOPE_FAILPOINTS)
#define TS_FAILPOINT(name) ::targetscope::failpoints::hit(name)
#else
#define TS_FAILPOINT(name) false
#endif
