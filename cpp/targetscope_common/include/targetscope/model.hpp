// doc/interfaces.md#shared-types
// doc/shared_libraries.md#c-library-targetscope_common_cpp
#pragma once

#include <cstdint>
#include <nlohmann/json.hpp>
#include <optional>
#include <string>
#include <vector>

namespace targetscope::model {

using nlohmann::json;

// doc/interfaces.md#shot
struct Shot {
    double angle_turns{};
    double radius_mm{};
};

// doc/interfaces.md#ring
struct Ring {
    std::int64_t score{};
    double radius_mm{};
};

// doc/interfaces.md#targettype
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
    std::vector<Ring> rings;
    bool locked{};
    bool retired{};
};

// doc/interfaces.md#corners
struct PixelPoint {
    double x{};
    double y{};
};

struct Corners {
    PixelPoint top_left;
    PixelPoint top_right;
    PixelPoint bottom_right;
    PixelPoint bottom_left;
};

// doc/interfaces.md#calibrationstatus
struct CalibrationStatus {
    std::string state = "not_calibrated";
    std::optional<std::string> method;
    std::optional<std::int64_t> target_type_id;
    std::optional<Corners> corners;
    std::optional<double> px_per_mm;
    std::optional<std::string> message;
    std::int64_t updated_ms{};
};

// doc/interfaces.md#cameracontrol
struct MenuEntry {
    std::int64_t value{};
    std::string label;
};

struct CameraControl {
    std::int64_t id{};
    std::string name;
    std::string kind;
    std::int64_t min{}, max{}, step{}, default_value{};
    std::int64_t value{};
    std::vector<MenuEntry> menu;
    bool inactive{};
};

// doc/interfaces.md#setting
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

// doc/interfaces.md#filter
struct Condition {
    std::string field;
    std::string op;
    json value;
};

struct Filter {
    std::vector<Condition> conditions;
};

void to_json(json& j, const Shot& v);
void from_json(const json& j, Shot& v);
void to_json(json& j, const Ring& v);
void from_json(const json& j, Ring& v);
void to_json(json& j, const TargetType& v);
void from_json(const json& j, TargetType& v);
void to_json(json& j, const PixelPoint& v);
void from_json(const json& j, PixelPoint& v);
void to_json(json& j, const Corners& v);
void from_json(const json& j, Corners& v);
void to_json(json& j, const CalibrationStatus& v);
void from_json(const json& j, CalibrationStatus& v);
void to_json(json& j, const MenuEntry& v);
void from_json(const json& j, MenuEntry& v);
void to_json(json& j, const CameraControl& v);
void from_json(const json& j, CameraControl& v);
void to_json(json& j, const Setting& v);
void from_json(const json& j, Setting& v);
void to_json(json& j, const Condition& v);
void to_json(json& j, const Filter& v);

// doc/shared_libraries.md#messages
template <class T>
void get_optional(const json& j, const char* key, std::optional<T>& out) {
    auto it = j.find(key);
    if (it == j.end() || it->is_null()) {
        out.reset();
    } else {
        out = it->get<T>();
    }
}

template <class T>
json optional_json(const std::optional<T>& v) {
    return v ? json(*v) : json(nullptr);
}

}
