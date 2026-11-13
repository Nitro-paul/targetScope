// doc/shared_libraries.md#target-types
#pragma once

#include <optional>
#include <string>
#include <vector>

#include "targetscope/model.hpp"

namespace targetscope::targets {

std::vector<model::Ring> generate_rings(const model::TargetType& t);
std::optional<std::string> validate_target_type(const model::TargetType& t);

}  // namespace targetscope::targets
