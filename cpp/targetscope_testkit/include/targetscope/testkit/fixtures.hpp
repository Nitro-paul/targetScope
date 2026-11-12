// Reference data (doc/testing/overview.md#reference-data).
#pragma once

#include <filesystem>
#include <string>

#include "targetscope/model.hpp"

namespace targetscope::testkit {

std::filesystem::path testdata(const std::string& rel);
// Copies testdata/images/<name> into the staging folder; returns the staged name.
std::string stage_image(const std::filesystem::path& staging_dir, const std::string& name);
// The reference target type TT-REF.
model::TargetType tt_ref();

}  // namespace targetscope::testkit
