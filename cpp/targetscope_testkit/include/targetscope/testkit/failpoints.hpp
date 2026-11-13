// doc/testing/overview.md#failpoints
#pragma once

#include <string>

namespace targetscope::testkit {

std::string failpoint_env(const std::string& name, const std::string& action);

}
