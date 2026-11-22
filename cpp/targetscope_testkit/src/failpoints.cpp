// doc/testing/overview.md#failpoints
#include "targetscope/testkit/failpoints.hpp"

namespace targetscope::testkit {

std::string failpoint_env(const std::string& name, const std::string& action) { return name + "=" + action; }

}
