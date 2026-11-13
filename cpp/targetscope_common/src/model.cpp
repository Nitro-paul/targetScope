// doc/interfaces.md#shared-types
#include "targetscope/model.hpp"

namespace targetscope::model {

void to_json(json& j, const Shot& v) { j = json{{"angle_turns", v.angle_turns}, {"radius_mm", v.radius_mm}}; }
void from_json(const json& j, Shot& v) {
    j.at("angle_turns").get_to(v.angle_turns);
    j.at("radius_mm").get_to(v.radius_mm);
}

void to_json(json& j, const Ring& v) { j = json{{"score", v.score}, {"radius_mm", v.radius_mm}}; }
void from_json(const json& j, Ring& v) {
    j.at("score").get_to(v.score);
    j.at("radius_mm").get_to(v.radius_mm);
}

void to_json(json& j, const TargetType& v) {
    j = json{{"name", v.name},
             {"paper_width_mm", v.paper_width_mm},
             {"paper_height_mm", v.paper_height_mm},
             {"center_x_mm", v.center_x_mm},
             {"center_y_mm", v.center_y_mm},
             {"inner_white_radius_mm", v.inner_white_radius_mm},
             {"inner_white_score", v.inner_white_score},
             {"black_radius_mm", v.black_radius_mm},
             {"black_score", v.black_score},
             {"outer_radius_mm", v.outer_radius_mm},
             {"outer_score", v.outer_score},
             {"rings", v.rings},
             {"locked", v.locked},
             {"retired", v.retired}};
    if (v.id) j["id"] = *v.id;
    if (v.uuid) j["uuid"] = *v.uuid;
}

void from_json(const json& j, TargetType& v) {
    get_optional(j, "id", v.id);
    get_optional(j, "uuid", v.uuid);
    j.at("name").get_to(v.name);
    j.at("paper_width_mm").get_to(v.paper_width_mm);
    j.at("paper_height_mm").get_to(v.paper_height_mm);
    j.at("center_x_mm").get_to(v.center_x_mm);
    j.at("center_y_mm").get_to(v.center_y_mm);
    j.at("inner_white_radius_mm").get_to(v.inner_white_radius_mm);
    j.at("inner_white_score").get_to(v.inner_white_score);
    j.at("black_radius_mm").get_to(v.black_radius_mm);
    j.at("black_score").get_to(v.black_score);
    j.at("outer_radius_mm").get_to(v.outer_radius_mm);
    j.at("outer_score").get_to(v.outer_score);
    v.rings = j.contains("rings") && j["rings"].is_array() ? j["rings"].get<std::vector<Ring>>() : std::vector<Ring>{};
    v.locked = j.value("locked", false);
    v.retired = j.value("retired", false);
}

void to_json(json& j, const PixelPoint& v) { j = json{{"x", v.x}, {"y", v.y}}; }
void from_json(const json& j, PixelPoint& v) {
    j.at("x").get_to(v.x);
    j.at("y").get_to(v.y);
}

void to_json(json& j, const Corners& v) {
    j = json{{"top_left", v.top_left},
             {"top_right", v.top_right},
             {"bottom_right", v.bottom_right},
             {"bottom_left", v.bottom_left}};
}
void from_json(const json& j, Corners& v) {
    j.at("top_left").get_to(v.top_left);
    j.at("top_right").get_to(v.top_right);
    j.at("bottom_right").get_to(v.bottom_right);
    j.at("bottom_left").get_to(v.bottom_left);
}

void to_json(json& j, const CalibrationStatus& v) {
    j = json{{"state", v.state},
             {"method", optional_json(v.method)},
             {"target_type_id", optional_json(v.target_type_id)},
             {"corners", optional_json(v.corners)},
             {"px_per_mm", optional_json(v.px_per_mm)},
             {"message", optional_json(v.message)},
             {"updated_ms", v.updated_ms}};
}
void from_json(const json& j, CalibrationStatus& v) {
    j.at("state").get_to(v.state);
    get_optional(j, "method", v.method);
    get_optional(j, "target_type_id", v.target_type_id);
    get_optional(j, "corners", v.corners);
    get_optional(j, "px_per_mm", v.px_per_mm);
    get_optional(j, "message", v.message);
    j.at("updated_ms").get_to(v.updated_ms);
}

void to_json(json& j, const MenuEntry& v) { j = json{{"value", v.value}, {"label", v.label}}; }
void from_json(const json& j, MenuEntry& v) {
    j.at("value").get_to(v.value);
    j.at("label").get_to(v.label);
}

void to_json(json& j, const CameraControl& v) {
    j = json{{"id", v.id},
             {"name", v.name},
             {"kind", v.kind},
             {"min", v.min},
             {"max", v.max},
             {"step", v.step},
             {"default", v.default_value},
             {"value", v.value},
             {"menu", v.menu},
             {"inactive", v.inactive}};
}
void from_json(const json& j, CameraControl& v) {
    j.at("id").get_to(v.id);
    j.at("name").get_to(v.name);
    j.at("kind").get_to(v.kind);
    j.at("min").get_to(v.min);
    j.at("max").get_to(v.max);
    j.at("step").get_to(v.step);
    j.at("default").get_to(v.default_value);
    j.at("value").get_to(v.value);
    v.menu = j.contains("menu") ? j["menu"].get<std::vector<MenuEntry>>() : std::vector<MenuEntry>{};
    j.at("inactive").get_to(v.inactive);
}

void to_json(json& j, const Setting& v) {
    j = json{{"name", v.name},
             {"display_name", v.display_name},
             {"active", v.active},
             {"value", v.value},
             {"optional", v.optional},
             {"default_value", v.default_value},
             {"default_active", v.default_active},
             {"value_rule", v.value_rule}};
}
void from_json(const json& j, Setting& v) {
    j.at("name").get_to(v.name);
    j.at("display_name").get_to(v.display_name);
    j.at("active").get_to(v.active);
    v.value = j.value("value", std::string{});
    v.optional = j.value("optional", false);
    v.default_value = j.value("default_value", std::string{});
    v.default_active = j.value("default_active", true);
    v.value_rule = j.value("value_rule", std::string{});
}

void to_json(json& j, const Condition& v) { j = json{{"field", v.field}, {"op", v.op}, {"value", v.value}}; }
void to_json(json& j, const Filter& v) { j = json{{"conditions", v.conditions}}; }

}
