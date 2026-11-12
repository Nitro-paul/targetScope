// Logging setup with spdlog: stderr + daily rotated file <log_dir>/<program>.log
// (doc/shared_libraries.md#logging).
#pragma once

#include <string>

#include "targetscope/config.hpp"

namespace targetscope::logging {

void init(const std::string& program, const BootstrapConfig& config);

}  // namespace targetscope::logging
