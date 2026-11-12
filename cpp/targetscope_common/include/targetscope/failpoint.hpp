// Named failpoints, compiled into test builds only (doc/testing/overview.md#failpoints).
// A test switches a failpoint on to return an error, wait, or abort the process.
#pragma once

#include <string_view>

namespace targetscope::failpoints {

// Returns true if the failpoint is set to "return an error"; waits or aborts if set so.
bool hit(std::string_view name);

}  // namespace targetscope::failpoints

#if defined(TARGETSCOPE_FAILPOINTS)
#define TS_FAILPOINT(name) ::targetscope::failpoints::hit(name)
#else
#define TS_FAILPOINT(name) false
#endif
