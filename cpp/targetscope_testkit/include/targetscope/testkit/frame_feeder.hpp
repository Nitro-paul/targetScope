// doc/testing/overview.md#test-kit
#pragma once

#include <atomic>
#include <cstdint>
#include <functional>
#include <map>
#include <mutex>
#include <optional>
#include <string>
#include <thread>
#include <vector>

#include "targetscope/frames.hpp"

namespace targetscope::testkit {

struct FeedFrame {
    int width = 0;
    int height = 0;
    std::vector<std::uint8_t> bgr;
};

class FrameFeeder {
   public:
    using Generator = std::function<FeedFrame(std::uint64_t index)>;
    FrameFeeder(const std::string& service, double fps, Generator generator);
    ~FrameFeeder();
    void pause();
    void resume();
    std::uint64_t published() const { return published_; }
    std::optional<std::uint64_t> timestamp_of(std::uint64_t seq) const;

   private:
    void run();
    frames::FramePublisher publisher_;
    double fps_;
    Generator generator_;
    std::atomic<bool> stop_{false};
    std::atomic<bool> paused_{false};
    std::atomic<std::uint64_t> published_{0};
    mutable std::mutex mutex_;
    std::map<std::uint64_t, std::uint64_t> timestamps_;
    std::thread thread_;
};

}
