// doc/programs/camera_reader.md#camera-controls
#include "camera_controls.hpp"

#include <fcntl.h>
#include <linux/videodev2.h>
#include <spdlog/spdlog.h>
#include <sys/ioctl.h>
#include <unistd.h>

#include <algorithm>
#include <cerrno>
#include <cstring>

#include "targetscope/messages.hpp"

namespace camera_reader {

namespace {

using targetscope::ErrorCode;
using targetscope::ServiceError;
using targetscope::model::CameraControl;

bool is_auto_mode(std::int64_t id) {
    switch (id) {
        case V4L2_CID_AUTOGAIN:
        case V4L2_CID_AUTO_WHITE_BALANCE:
        case V4L2_CID_HUE_AUTO:
        case V4L2_CID_EXPOSURE_AUTO:
        case V4L2_CID_FOCUS_AUTO:
            return true;
        default:
            return false;
    }
}

bool supported_type(__u32 type) {
    return type == V4L2_CTRL_TYPE_INTEGER || type == V4L2_CTRL_TYPE_BOOLEAN || type == V4L2_CTRL_TYPE_MENU ||
           type == V4L2_CTRL_TYPE_INTEGER_MENU || type == V4L2_CTRL_TYPE_BUTTON;
}

std::string kind_of(__u32 type) {
    switch (type) {
        case V4L2_CTRL_TYPE_BOOLEAN:
            return "boolean";
        case V4L2_CTRL_TYPE_MENU:
        case V4L2_CTRL_TYPE_INTEGER_MENU:
            return "menu";
        case V4L2_CTRL_TYPE_BUTTON:
            return "button";
        default:
            return "integer";
    }
}

}

CameraControls::CameraControls(const std::string& device) {
    fd_ = ::open(device.c_str(), O_RDWR | O_NONBLOCK);
    if (fd_ < 0) throw ServiceError(ErrorCode::Unavailable, "cannot open " + device + ": " + std::strerror(errno));
}

CameraControls::~CameraControls() {
    if (fd_ >= 0) ::close(fd_);
}

// doc/programs/camera_reader.md#camera-controls
std::vector<CameraControl> CameraControls::list() const {
    std::vector<CameraControl> out;
    v4l2_query_ext_ctrl q{};
    q.id = V4L2_CTRL_FLAG_NEXT_CTRL;
    while (ioctl(fd_, VIDIOC_QUERY_EXT_CTRL, &q) == 0) {
        const __u32 id = q.id;
        const bool skip = q.type == V4L2_CTRL_TYPE_CTRL_CLASS || (q.flags & V4L2_CTRL_FLAG_DISABLED) ||
                          (q.flags & V4L2_CTRL_FLAG_HAS_PAYLOAD) || !supported_type(q.type);
        if (!skip) {
            CameraControl c;
            c.id = id;
            c.name = q.name;
            c.kind = kind_of(q.type);
            c.min = q.minimum;
            c.max = q.maximum;
            c.step = std::max<std::int64_t>(q.step, 1);
            c.default_value = q.default_value;
            c.inactive = (q.flags & V4L2_CTRL_FLAG_INACTIVE) != 0;
            if (q.type == V4L2_CTRL_TYPE_BUTTON) {
                c.min = c.max = c.default_value = 0;
            } else {
                v4l2_ext_control ec{};
                ec.id = id;
                v4l2_ext_controls ecs{};
                ecs.which = V4L2_CTRL_ID2WHICH(id);
                ecs.count = 1;
                ecs.controls = &ec;
                if (ioctl(fd_, VIDIOC_G_EXT_CTRLS, &ecs) == 0) c.value = ec.value;
            }
            if (q.type == V4L2_CTRL_TYPE_MENU || q.type == V4L2_CTRL_TYPE_INTEGER_MENU) {
                for (std::int64_t i = q.minimum; i <= q.maximum; ++i) {
                    v4l2_querymenu m{};
                    m.id = id;
                    m.index = static_cast<__u32>(i);
                    if (ioctl(fd_, VIDIOC_QUERYMENU, &m) != 0) continue;
                    targetscope::model::MenuEntry e;
                    e.value = i;
                    e.label = q.type == V4L2_CTRL_TYPE_MENU ? std::string(reinterpret_cast<const char*>(m.name))
                                                            : std::to_string(m.value);
                    c.menu.push_back(std::move(e));
                }
            }
            out.push_back(std::move(c));
        }
        q = {};
        q.id = id | V4L2_CTRL_FLAG_NEXT_CTRL;
    }
    return out;
}

// doc/programs/camera_reader.md#camera-controls
std::vector<CameraControl> CameraControls::set(std::int64_t id, std::int64_t value) {
    auto controls = list();
    auto it = std::find_if(controls.begin(), controls.end(), [&](auto& c) { return c.id == id; });
    if (it == controls.end()) throw ServiceError(ErrorCode::ValidationFailed, "unknown control " + std::to_string(id));
    if (it->kind == "button") {
        value = 1;
    } else {
        if (value < it->min || value > it->max)
            throw ServiceError(ErrorCode::ValidationFailed, it->name + ": " + std::to_string(value) + " is outside " +
                                                                std::to_string(it->min) + ".." +
                                                                std::to_string(it->max));
        if (it->kind == "integer" && (value - it->min) % it->step != 0)
            throw ServiceError(
                ErrorCode::ValidationFailed,
                it->name + ": " + std::to_string(value) + " is not a multiple of the step " + std::to_string(it->step));
        if (it->kind == "menu" && !it->menu.empty() &&
            std::none_of(it->menu.begin(), it->menu.end(), [&](auto& m) { return m.value == value; }))
            throw ServiceError(ErrorCode::ValidationFailed,
                               it->name + ": " + std::to_string(value) + " is not a menu entry");
    }
    v4l2_ext_control ec{};
    ec.id = static_cast<__u32>(id);
    ec.value = value;
    v4l2_ext_controls ecs{};
    ecs.which = V4L2_CTRL_ID2WHICH(ec.id);
    ecs.count = 1;
    ecs.controls = &ec;
    if (ioctl(fd_, VIDIOC_S_EXT_CTRLS, &ecs) != 0)
        throw ServiceError(ErrorCode::ValidationFailed, it->name + ": " + std::strerror(errno));
    return list();
}

// doc/programs/camera_reader.md#camera-controls
void CameraControls::apply(const std::map<std::int64_t, std::int64_t>& stored) {
    std::vector<std::pair<std::int64_t, std::int64_t>> ordered;
    for (auto& [id, v] : stored)
        if (is_auto_mode(id)) ordered.emplace_back(id, v);
    for (auto& [id, v] : stored)
        if (!is_auto_mode(id)) ordered.emplace_back(id, v);
    for (auto [id, v] : ordered) {
        try {
            set(id, v);
        } catch (const ServiceError& e) {
            spdlog::warn("stored value of control {} rejected: {}", id, e.what());
        }
    }
}

// doc/programs/camera_reader.md#camera-controls
void CameraControls::reset_to_defaults() {
    auto controls = list();
    std::map<std::int64_t, std::int64_t> defaults;
    for (auto& c : controls)
        if (c.kind != "button") defaults[c.id] = c.default_value;
    apply(defaults);
}

}
