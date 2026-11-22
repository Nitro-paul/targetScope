// doc/programs/camera_reader.md#camera-controls
#include "v4l2_devices.hpp"

#include <fcntl.h>
#include <linux/videodev2.h>
#include <sys/ioctl.h>
#include <unistd.h>

#include <algorithm>
#include <cstring>
#include <filesystem>
#include <set>

namespace camera_reader {

namespace {

struct Fd {
    int fd;
    explicit Fd(const std::string& path) : fd(::open(path.c_str(), O_RDONLY | O_NONBLOCK)) {}
    ~Fd() {
        if (fd >= 0) ::close(fd);
    }
};

int video_number(const std::string& path) {
    auto pos = path.rfind("video");
    return pos == std::string::npos ? 1 << 30 : std::atoi(path.c_str() + pos + 5);
}

std::vector<std::string> video_nodes() {
    std::vector<std::string> nodes;
    std::error_code ec;
    for (auto& e : std::filesystem::directory_iterator("/dev", ec)) {
        const std::string name = e.path().filename().string();
        if (name.rfind("video", 0) == 0 && name.size() > 5 && std::isdigit(static_cast<unsigned char>(name[5])))
            nodes.push_back(e.path().string());
    }
    std::sort(nodes.begin(), nodes.end(), [](auto& a, auto& b) { return video_number(a) < video_number(b); });
    return nodes;
}

bool query(int fd, v4l2_capability& cap) {
    std::memset(&cap, 0, sizeof cap);
    if (ioctl(fd, VIDIOC_QUERYCAP, &cap) != 0) return false;
    const __u32 caps = (cap.capabilities & V4L2_CAP_DEVICE_CAPS) ? cap.device_caps : cap.capabilities;
    return (caps & V4L2_CAP_VIDEO_CAPTURE) != 0;
}

void add_fps(std::vector<double>& fps, double v) {
    if (v <= 0) return;
    for (double x : fps)
        if (std::abs(x - v) < 0.01) return;
    fps.push_back(v);
}

DeviceFormat& format_for(std::vector<DeviceFormat>& formats, int w, int h) {
    for (auto& f : formats)
        if (f.width_px == w && f.height_px == h) return f;
    formats.push_back({w, h, {}});
    return formats.back();
}

void enumerate_intervals(int fd, __u32 pixel_format, int w, int h, DeviceFormat& out) {
    for (__u32 i = 0;; ++i) {
        v4l2_frmivalenum iv{};
        iv.index = i;
        iv.pixel_format = pixel_format;
        iv.width = w;
        iv.height = h;
        if (ioctl(fd, VIDIOC_ENUM_FRAMEINTERVALS, &iv) != 0) break;
        if (iv.type == V4L2_FRMIVAL_TYPE_DISCRETE) {
            add_fps(out.fps, static_cast<double>(iv.discrete.denominator) / iv.discrete.numerator);
        } else {
            add_fps(out.fps, static_cast<double>(iv.stepwise.min.denominator) / iv.stepwise.min.numerator);
            add_fps(out.fps, static_cast<double>(iv.stepwise.max.denominator) / iv.stepwise.max.numerator);
            break;
        }
    }
}

std::vector<DeviceFormat> enumerate_formats(int fd) {
    std::vector<DeviceFormat> formats;
    for (__u32 f = 0;; ++f) {
        v4l2_fmtdesc desc{};
        desc.index = f;
        desc.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        if (ioctl(fd, VIDIOC_ENUM_FMT, &desc) != 0) break;
        for (__u32 s = 0;; ++s) {
            v4l2_frmsizeenum size{};
            size.index = s;
            size.pixel_format = desc.pixelformat;
            if (ioctl(fd, VIDIOC_ENUM_FRAMESIZES, &size) != 0) break;
            if (size.type == V4L2_FRMSIZE_TYPE_DISCRETE) {
                auto& df = format_for(formats, size.discrete.width, size.discrete.height);
                enumerate_intervals(fd, desc.pixelformat, size.discrete.width, size.discrete.height, df);
            } else {
                for (auto [w, h] : {std::pair{size.stepwise.max_width, size.stepwise.max_height},
                                    std::pair{size.stepwise.min_width, size.stepwise.min_height}}) {
                    auto& df = format_for(formats, w, h);
                    enumerate_intervals(fd, desc.pixelformat, w, h, df);
                }
                break;
            }
        }
    }
    for (auto& f : formats) std::sort(f.fps.begin(), f.fps.end(), std::greater<>());
    std::sort(formats.begin(), formats.end(), [](auto& a, auto& b) {
        return a.width_px != b.width_px ? a.width_px > b.width_px : a.height_px > b.height_px;
    });
    return formats;
}

}

// doc/programs/camera_reader.md#camera-controls
std::vector<CaptureDevice> list_capture_devices() {
    std::vector<CaptureDevice> devices;
    for (const auto& path : video_nodes()) {
        Fd fd(path);
        if (fd.fd < 0) continue;
        v4l2_capability cap;
        if (!query(fd.fd, cap)) continue;
        CaptureDevice d;
        d.path = path;
        d.name = reinterpret_cast<const char*>(cap.card);
        d.bus_info = reinterpret_cast<const char*>(cap.bus_info);
        d.formats = enumerate_formats(fd.fd);
        devices.push_back(std::move(d));
    }
    return devices;
}

// doc/programs/camera_reader.md#sources
std::optional<ResolvedDevice> resolve_camera(const std::string& bus_info) {
    for (const auto& path : video_nodes()) {
        Fd fd(path);
        if (fd.fd < 0) continue;
        v4l2_capability cap;
        if (!query(fd.fd, cap)) continue;
        const std::string found = reinterpret_cast<const char*>(cap.bus_info);
        if (bus_info.empty() || bus_info == found) return ResolvedDevice{path, found};
    }
    return std::nullopt;
}

std::optional<std::string> resolve_device(const std::string& bus_info) {
    auto r = resolve_camera(bus_info);
    if (!r) return std::nullopt;
    return r->path;
}

}
