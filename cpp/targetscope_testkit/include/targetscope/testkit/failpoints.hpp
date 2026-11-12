// Switches named failpoints on (doc/testing/overview.md#failpoints).
#pragma once

#include <string>

namespace targetscope::testkit {

// Value of the FAILPOINTS environment variable for a program under test,
// e.g. "bus.before_ack=abort".
std::string failpoint_env(const std::string& name, const std::string& action);

}  // namespace targetscope::testkit
