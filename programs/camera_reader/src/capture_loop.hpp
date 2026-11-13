// Capture thread (doc/programs/camera_reader.md#capture-loop):
// create source from settings -> open (retry every 2 s) -> apply controls -> publish frames;
// 10 failed reads in a row or 2 s without a frame -> disconnected; reopen on settings change.
#pragma once

#include <atomic>
#include <chrono>
#include <cstdint>
#include <functional>
#include <memory>
#include <mutex>
#include <optional>
#include <string>
#include <thread>

#include "frame_source.hpp"

namespace camera_reader {

enum class CameraState { Connected, Disconnected, Disabled };

struct CameraStatus {
    CameraState state = CameraState::Disabled;
    std::string camera_source;  // v4l2 | rtsp | none
    std::string device;
    std::string camera_key;
    std::optional<SourceInfo> info;
    std::uint64_t frames_published = 0;
    std::string message;
};

inline constexpr auto RETRY_INTERVAL = std::chrono::seconds(2);
inline constexpr auto FRAME_TIMEOUT = std::chrono::seconds(2);
inline constexpr int MAX_FAILED_READS = 10;

class CaptureLoop {
public:
    using SourceFactory = std::function<std::unique_ptr<FrameSource>()>;
    using StateCallback = std::function<void(const CameraStatus&)>;  // -> camera_state_changed

    CaptureLoop(SourceFactory factory, StateCallback on_state);
    ~CaptureLoop();

    void start();
    void stop();
    void reopen();  // a reopen setting changed
    CameraStatus status() const;

private:
    void run();
    SourceFactory factory_;
    StateCallback on_state_;
    mutable std::mutex mutex_;
    CameraStatus status_;
    std::atomic<bool> stop_{false};
    std::atomic<bool> reopen_{false};
    std::thread thread_;
};

}  // namespace camera_reader
