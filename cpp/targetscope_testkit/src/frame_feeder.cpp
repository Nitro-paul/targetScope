// doc/testing/overview.md#test-kit
#include "targetscope/testkit/frame_feeder.hpp"

#include <chrono>
#include <cstring>
#include <ctime>

namespace targetscope::testkit {

FrameFeeder::FrameFeeder(const std::string& service, double fps, Generator generator)
    : publisher_(service, 1920 * 1080 * 3), fps_(fps), generator_(std::move(generator)) {
    thread_ = std::thread([this] { run(); });
}

FrameFeeder::~FrameFeeder() {
    stop_ = true;
    if (thread_.joinable()) thread_.join();
}

std::optional<std::uint64_t> FrameFeeder::timestamp_of(std::uint64_t seq) const {
    std::lock_guard lock(mutex_);
    auto it = timestamps_.find(seq);
    if (it == timestamps_.end()) return std::nullopt;
    return it->second;
}

void FrameFeeder::pause() { paused_ = true; }
void FrameFeeder::resume() { paused_ = false; }

void FrameFeeder::run() {
    using clock = std::chrono::steady_clock;
    const auto period = std::chrono::duration_cast<clock::duration>(std::chrono::duration<double>(1.0 / fps_));
    auto next = clock::now();
    std::uint64_t index = 0;
    std::uint64_t seq = 0;
    while (!stop_) {
        next += period;
        if (!paused_) {
            FeedFrame f = generator_(index++);
            timespec ts{};
            clock_gettime(CLOCK_MONOTONIC, &ts);
            TargetScopeFrameHeader h{};
            h.seq = seq;
            h.source_seq = seq;
            h.timestamp_ns = static_cast<std::uint64_t>(ts.tv_sec) * 1000000000ull + ts.tv_nsec;
            h.width = f.width;
            h.height = f.height;
            h.stride = f.width * 3;
            h.format = TARGETSCOPE_PIXEL_BGR8;
            {
                std::lock_guard lock(mutex_);
                timestamps_[seq] = h.timestamp_ns;
                if (timestamps_.size() > 512) timestamps_.erase(timestamps_.begin());
            }
            ++seq;
            publisher_.publish(h,
                               [&](std::span<std::uint8_t> out) { std::memcpy(out.data(), f.bgr.data(), out.size()); });
            ++published_;
        }
        std::this_thread::sleep_until(next);
    }
}

}
