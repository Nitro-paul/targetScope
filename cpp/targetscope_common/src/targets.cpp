// doc/shared_libraries.md#target-types
// doc/interfaces.md#targettype
#include "targetscope/targets.hpp"

#include <array>
#include <cmath>
#include <utility>

namespace targetscope::targets {

std::vector<model::Ring> generate_rings(const model::TargetType& t) {
    const std::array<std::pair<std::int64_t, double>, 3> anchors{{
        {t.inner_white_score, t.inner_white_radius_mm},
        {t.black_score, t.black_radius_mm},
        {t.outer_score, t.outer_radius_mm},
    }};
    auto radius = [&](std::int64_t s) {
        for (const auto& [score, r] : anchors)
            if (score == s) return r;
        const auto& a = s > anchors[1].first ? anchors[0] : anchors[1];
        const auto& b = s > anchors[1].first ? anchors[1] : anchors[2];
        return a.second + static_cast<double>(a.first - s) * (b.second - a.second) / static_cast<double>(a.first - b.first);
    };
    std::vector<model::Ring> rings;
    for (std::int64_t s = t.inner_white_score; s >= t.outer_score && s >= 1; --s) rings.push_back({s, radius(s)});
    return rings;
}

std::optional<std::string> validate_target_type(const model::TargetType& t) {
    for (double v : {t.paper_width_mm, t.paper_height_mm, t.center_x_mm, t.center_y_mm, t.inner_white_radius_mm, t.black_radius_mm,
                     t.outer_radius_mm})
        if (!std::isfinite(v)) return "all sizes are finite numbers";
    const std::array<std::pair<bool, const char*>, 11> rules{{
        {t.paper_width_mm > 0 && t.paper_height_mm > 0, "paper size > 0"},
        {t.inner_white_score > t.black_score, "inner_white_score > black_score"},
        {t.black_score > t.outer_score, "black_score > outer_score"},
        {t.outer_score >= 1, "outer_score >= 1"},
        {t.inner_white_radius_mm > 0, "inner_white_radius_mm > 0"},
        {t.inner_white_radius_mm < t.black_radius_mm, "inner_white_radius_mm < black_radius_mm"},
        {t.black_radius_mm < t.outer_radius_mm, "black_radius_mm < outer_radius_mm"},
        {t.center_x_mm - t.outer_radius_mm >= 0, "outermost ring on the paper (left edge)"},
        {t.center_x_mm + t.outer_radius_mm <= t.paper_width_mm, "outermost ring on the paper (right edge)"},
        {t.center_y_mm - t.outer_radius_mm >= 0, "outermost ring on the paper (top edge)"},
        {t.center_y_mm + t.outer_radius_mm <= t.paper_height_mm, "outermost ring on the paper (bottom edge)"},
    }};
    for (const auto& [ok, rule] : rules)
        if (!ok) return std::string(rule);
    return std::nullopt;
}

}  // namespace targetscope::targets
