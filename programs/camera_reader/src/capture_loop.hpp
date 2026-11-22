// doc/programs/camera_reader.md#capture-loop
#pragma once

#include <atomic>
#include <chrono>
#include <condition_variable>
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

const char* to_string(CameraState s);

struct CameraStatus {
    CameraState state = CameraState::Disabled;
    std::string camera_source;
    std::string device;
    std::string camera_key;
    std::optional<SourceInfo> info;
    std::uint64_t frames_published = 0;
    std::string message;
};

// doc/programs/camera_reader.md#capture-loop
struct SourceChoice {
    std::unique_ptr<FrameSource> source;
    std::string camera_source;
    std::string camera_key;
    std::string message;
};

struct Timing {
    std::chrono::milliseconds retry_interval{2000};
    std::chrono::milliseconds frame_timeout{2000};
    int max_failed_reads = 10;
};

inline constexpr auto RETRY_INTERVAL = std::chrono::seconds(2);
inline constexpr auto FRAME_TIMEOUT = std::chrono::seconds(2);
inline constexpr int MAX_FAILED_READS = 10;

class CaptureLoop {
   public:
    using SourceFactory = std::function<SourceChoice()>;
    using StateCallback = std::function<void(const CameraStatus&)>;
    using OpenedCallback = std::function<void(FrameSource&, const CameraStatus&)>;

    CaptureLoop(SourceFactory factory, StateCallback on_state, std::string service, Timing timing = {});
    ~CaptureLoop();

    void set_opened_callback(OpenedCallback cb) { on_opened_ = std::move(cb); }
    void start();
    void stop();
    void reopen();
    CameraStatus status() const;

   private:
    void run();
    void watchdog();
    bool wait_for_change(std::chrono::milliseconds max);
    void set_status(const CameraStatus& next);

    SourceFactory factory_;
    StateCallback on_state_;
    OpenedCallback on_opened_;
    std::string service_;
    Timing timing_;
    mutable std::mutex mutex_;
    std::condition_variable cv_;
    CameraStatus status_;
    std::atomic<bool> stop_{false};
    std::atomic<bool> reopen_{false};
    std::atomic<std::int64_t> last_frame_ms_{0};
    std::atomic<bool> reading_{false};
    std::thread thread_;
    std::thread watchdog_;
};

}
