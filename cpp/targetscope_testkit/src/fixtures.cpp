// doc/testing/overview.md#reference-data
#include "targetscope/testkit/fixtures.hpp"

#include <fstream>

#include "targetscope/messages.hpp"
#include "targetscope/targets.hpp"

namespace targetscope::testkit {

std::filesystem::path testdata(const std::string& rel) { return std::filesystem::path(TARGETSCOPE_TESTDATA_DIR) / rel; }

std::string stage_image(const std::filesystem::path& staging_dir, const std::string& name) {
    std::string staged = "upload_" + new_uuid() + std::filesystem::path(name).extension().string();
    std::filesystem::copy_file(testdata("images/" + name), staging_dir / staged);
    return staged;
}

model::TargetType tt_ref() {
    std::ifstream in(testdata("target_types/tt_ref.json"));
    auto j = nlohmann::json::parse(in);
    model::TargetType t = j.at("target_type").get<model::TargetType>();
    t.rings = targets::generate_rings(t);
    return t;
}

}
