// Ring generation and target type validation (doc/shared_libraries.md#target-types), same as in Rust.
#pragma once

#include <optional>
#include <string>
#include <vector>

#include "targetscope/model.hpp"

namespace targetscope::targets {

// One ring per score from inner_white_score down to outer_score, radii spaced equally between anchors.
std::vector<model::Ring> generate_rings(const model::TargetType& t);

// Returns the first broken rule, or nullopt if the target type is valid.
std::optional<std::string> validate_target_type(const model::TargetType& t);

}  // namespace targetscope::targets
