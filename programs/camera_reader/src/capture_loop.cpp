// doc/programs/camera_reader.md#capture-loop
#include "capture_loop.hpp"

#include <spdlog/spdlog.h>

#include <cstring>
#include <ctime>
#include <opencv2/imgproc.hpp>

#include "targetscope/frames.hpp"

namespace camera_reader {

namespace {

std::int64_t steady_ms() {
    return std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::steady_clock::now().time_since_epoch())
        .count();
}

std::uint64_t monotonic_ns() {
    timespec ts{};
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return static_cast<std::uint64_t>(ts.tv_sec) * 1000000000ull + static_cast<std::uint64_t>(ts.tv_nsec);
}

bool same(const CameraStatus& a, const CameraStatus& b) {
    auto size = [](const CameraStatus& s) { return s.info ? s.info->width_px * 100000 + s.info->height_px : 0; };
    return a.state == b.state && a.message == b.message && a.camera_key == b.camera_key && size(a) == size(b);
}

}

const char* to_string(CameraState s) {
    switch (s) {
        case CameraState::Connected:
            return "connected";
        case CameraState::Disconnected:
            return "disconnected";
        default:
            return "disabled";
    }
}

CaptureLoop::CaptureLoop(SourceFactory factory, StateCallback on_state, std::string service, Timing timing)
    : factory_(std::move(factory)), on_state_(std::move(on_state)), service_(std::move(service)), timing_(timing) {}

CaptureLoop::~CaptureLoop() { stop(); }

void CaptureLoop::start() {
    thread_ = std::thread([this] { run(); });
    watchdog_ = std::thread([this] { watchdog(); });
}

void CaptureLoop::stop() {
    {
        std::lock_guard lock(mutex_);
        stop_ = true;
    }
    cv_.notify_all();
    if (thread_.joinable()) thread_.join();
    if (watchdog_.joinable()) watchdog_.join();
}

// doc/programs/camera_reader.md#capture-loop
void CaptureLoop::reopen() {
    {
        std::lock_guard lock(mutex_);
        reopen_ = true;
    }
    cv_.notify_all();
}

CameraStatus CaptureLoop::status() const {
    std::lock_guard lock(mutex_);
    return status_;
}

// doc/programs/camera_reader.md#capture-loop
void CaptureLoop::set_status(const CameraStatus& next) {
    CameraStatus snapshot;
    bool changed;
    {
        std::lock_guard lock(mutex_);
        changed = !same(status_, next);
        status_ = next;
        snapshot = status_;
    }
    if (changed) {
        spdlog::info("camera state={} key=\"{}\" message=\"{}\"", to_string(snapshot.state), snapshot.camera_key,
                     snapshot.message);
        if (on_state_) on_state_(snapshot);
    }
}

bool CaptureLoop::wait_for_change(std::chrono::milliseconds max) {
    std::unique_lock lock(mutex_);
    cv_.wait_for(lock, max, [&] { return stop_.load() || reopen_.load(); });
    bool reopened = reopen_.exchange(false);
    return reopened || stop_;
}

// doc/programs/camera_reader.md#capture-loop
void CaptureLoop::watchdog() {
    while (!stop_) {
        {
            std::unique_lock lock(mutex_);
            cv_.wait_for(lock, std::chrono::milliseconds(100), [&] { return stop_.load(); });
        }
        if (!reading_) continue;
        if (steady_ms() - last_frame_ms_ > timing_.frame_timeout.count()) {
            CameraStatus s = status();
            if (s.state == CameraState::Connected) {
                s.state = CameraState::Disconnected;
                s.message = "no frames from " + s.camera_key;
                set_status(s);
            }
        }
    }
}

void CaptureLoop::run() {
    std::uint64_t seq = 0;
    std::uint64_t published = 0;
    while (!stop_) {
        reopen_ = false;
        SourceChoice choice = factory_();
        CameraStatus st;
        st.camera_source = choice.camera_source;
        st.camera_key = choice.camera_key;
        st.frames_published = published;
        if (!choice.source) {
            st.state = choice.camera_source == "none" ? CameraState::Disabled : CameraState::Disconnected;
            st.message = choice.message;
            set_status(st);
            wait_for_change(st.state == CameraState::Disabled ? std::chrono::hours(1) : timing_.retry_interval);
            continue;
        }
        SourceInfo info;
        try {
            info = choice.source->open();
        } catch (const std::exception& e) {
            st.state = CameraState::Disconnected;
            st.message = "cannot open " + (choice.camera_key.empty() ? choice.camera_source : choice.camera_key) +
                         ": " + e.what();
            set_status(st);
            choice.source->close();
            wait_for_change(timing_.retry_interval);
            continue;
        }
        st.state = CameraState::Connected;
        st.info = info;
        st.device = choice.source->device();
        if (!choice.source->camera_key().empty()) {
            choice.camera_key = choice.source->camera_key();
            st.camera_key = choice.camera_key;
        }
        st.message = "connected";
        set_status(st);
        if (on_opened_) {
            try {
                on_opened_(*choice.source, st);
            } catch (const std::exception& e) {
                spdlog::warn("applying stored controls failed error={}", e.what());
            }
        }

        std::unique_ptr<targetscope::frames::FramePublisher> publisher;
        std::string end_reason;
        int failed = 0;
        cv::Mat frame;
        last_frame_ms_ = steady_ms();
        reading_ = true;
        while (!stop_ && !reopen_) {
            try {
                choice.source->read(frame);
                failed = 0;
            } catch (const std::exception&) {
                if (++failed >= timing_.max_failed_reads) {
                    end_reason = "reading from " + choice.camera_key + " failed " + std::to_string(failed) + " times";
                    break;
                }
                continue;
            }
            const std::uint64_t timestamp = monotonic_ns();
            const std::int64_t now = steady_ms();
            if (now - last_frame_ms_ > timing_.frame_timeout.count()) {
                last_frame_ms_ = now;
                end_reason = "no frames from " + choice.camera_key + " for " +
                             std::to_string(timing_.frame_timeout.count()) + " ms";
                break;
            }
            last_frame_ms_ = now;
            if (frame.channels() != 3 || frame.depth() != CV_8U) {
                cv::Mat converted;
                if (frame.channels() == 1)
                    cv::cvtColor(frame, converted, cv::COLOR_GRAY2BGR);
                else if (frame.channels() == 4)
                    cv::cvtColor(frame, converted, cv::COLOR_BGRA2BGR);
                else
                    frame.convertTo(converted, CV_8UC3);
                frame = converted;
            }
            if (!publisher) {
                try {
                    publisher = std::make_unique<targetscope::frames::FramePublisher>(
                        service_, static_cast<std::size_t>(info.width_px) * info.height_px * 3);
                } catch (const std::exception& e) {
                    spdlog::error("creating the frame publisher failed error={}", e.what());
                    end_reason = std::string("frame publisher: ") + e.what();
                    break;
                }
            }
            TargetScopeFrameHeader h{};
            h.seq = seq;
            h.source_seq = seq;
            h.timestamp_ns = timestamp;
            h.width = static_cast<std::uint32_t>(frame.cols);
            h.height = static_cast<std::uint32_t>(frame.rows);
            h.stride = static_cast<std::uint32_t>(frame.cols) * 3;
            h.format = TARGETSCOPE_PIXEL_BGR8;
            h.px_per_mm = 0;
            try {
                publisher->publish(h, [&](std::span<std::uint8_t> out) {
                    for (int y = 0; y < frame.rows; ++y)
                        std::memcpy(out.data() + static_cast<std::size_t>(y) * h.stride, frame.ptr(y), h.stride);
                });
            } catch (const std::exception& e) {
                spdlog::error("publishing a frame failed error={}", e.what());
                end_reason = std::string("publishing: ") + e.what();
                break;
            }
            ++seq;
            ++published;
            {
                std::lock_guard lock(mutex_);
                status_.frames_published = published;
            }
        }
        reading_ = false;
        choice.source->close();
        publisher.reset();
        if (!end_reason.empty() && !stop_) {
            st.state = CameraState::Disconnected;
            st.message = end_reason;
            st.info.reset();
            st.frames_published = published;
            set_status(st);
            wait_for_change(timing_.retry_interval);
        }
    }
}

}
