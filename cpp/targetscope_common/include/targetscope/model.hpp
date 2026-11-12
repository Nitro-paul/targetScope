// Model types used by the C++ programs (doc/interfaces.md#shared-types), with
// nlohmann::json to_json/from_json. Field names are the JSON names of schemas/common/.
#pragma once

#include <cstdint>
#include <optional>
#include <string>
#include <vector>

#include <nlohmann/json.hpp>

namespace targetscope::model {

struct Shot {
    double angle_turns{};
    double radius_mm{};
};

struct Ring {
    std::int64_t score{};
    double radius_mm{};
};

struct TargetType {
    std::optional<std::int64_t> id;
    std::optional<std::string> uuid;
    std::string name;
    double paper_width_mm{};
    double paper_height_mm{};
    double center_x_mm{};
    double center_y_mm{};
    double inner_white_radius_mm{};
    std::int64_t inner_white_score{};
    double black_radius_mm{};
    std::int64_t black_score{};
    double outer_radius_mm{};
    std::int64_t outer_score{};
    std::vector<Ring> rings;  // output only, see targets.hpp
    bool locked{};
    bool retired{};
};

struct PixelPoint {
    double x{};
    double y{};
};

// Paper corners in image pixels (origin top-left, x right, y down).
struct Corners {
    PixelPoint top_left;
    PixelPoint top_right;
    PixelPoint bottom_right;
    PixelPoint bottom_left;
};

struct CalibrationStatus {
    std::string state;                 // TODO: enum (doc/interfaces.md#calibrationstatus)
    std::optional<std::string> method;  // auto | manual
    std::optional<std::int64_t> target_type_id;
    std::optional<Corners> corners;
    std::optional<double> px_per_mm;
    std::optional<std::string> message;
    std::int64_t updated_ms{};
};

struct MenuEntry {
    std::int64_t value{};
    std::string label;
};

struct CameraControl {
    std::int64_t id{};
    std::string name;
    std::string kind;  // TODO: enum (doc/interfaces.md#cameracontrol)
    std::int64_t min{}, max{}, step{}, default_value{};  // JSON "default"
    std::int64_t value{};
    std::vector<MenuEntry> menu;
    bool inactive{};
};

struct Setting {
    std::string name;
    std::string display_name;
    bool active{};
    std::string value;
    bool optional{};
    std::string default_value;
    bool default_active{};
    std::string value_rule;
};

struct Condition {
    std::string field;
    std::string op;  // eq, ne, lt, le, gt, ge, like, in
    nlohmann::json value;
};

struct Filter {
    std::vector<Condition> conditions;
    // TODO: order_by, limit, offset
};

void to_json(nlohmann::json& j, const Shot& v);
void from_json(const nlohmann::json& j, Shot& v);
// TODO: to_json / from_json for every type above

}  // namespace targetscope::model
