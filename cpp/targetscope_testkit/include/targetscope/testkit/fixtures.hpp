// doc/testing/overview.md#reference-data
#pragma once

#include <filesystem>
#include <string>

#include "targetscope/model.hpp"

namespace targetscope::testkit {

std::filesystem::path testdata(const std::string& rel);
std::string stage_image(const std::filesystem::path& staging_dir, const std::string& name);
model::TargetType tt_ref();

}
