// doc/frame_transport.md
#pragma once

#include <chrono>
#include <cstddef>
#include <cstdint>
#include <functional>
#include <memory>
#include <optional>
#include <span>
#include <string>

extern "C" {
#include "frame_header.h"
}

namespace targetscope::frames {

inline constexpr const char* FRAMES_RAW = "targetscope/frames/raw";
inline constexpr const char* FRAMES_LIVE = "targetscope/frames/live";
inline constexpr const char* NOTIFY_SUFFIX = "/notify";

// doc/frame_transport.md#iceoryx2-services
inline constexpr std::size_t MAX_PUBLISHERS = 1;
inline constexpr std::size_t MAX_SUBSCRIBERS = 4;
inline constexpr std::size_t HISTORY_SIZE = 0;
inline constexpr std::size_t SUBSCRIBER_MAX_BUFFER_SIZE = 2;
inline constexpr bool ENABLE_SAFE_OVERFLOW = true;

struct FrameHeader : TargetScopeFrameHeader {
    static constexpr const char* IOX2_TYPE_NAME = TARGETSCOPE_FRAME_HEADER_TYPE_NAME;
};

static_assert(sizeof(TargetScopeFrameHeader) == 48);
static_assert(alignof(TargetScopeFrameHeader) == 8);
static_assert(offsetof(TargetScopeFrameHeader, seq) == 0);
static_assert(offsetof(TargetScopeFrameHeader, timestamp_ns) == 8);
static_assert(offsetof(TargetScopeFrameHeader, source_seq) == 16);
static_assert(offsetof(TargetScopeFrameHeader, width) == 24);
static_assert(offsetof(TargetScopeFrameHeader, height) == 28);
static_assert(offsetof(TargetScopeFrameHeader, stride) == 32);
static_assert(offsetof(TargetScopeFrameHeader, format) == 36);
static_assert(offsetof(TargetScopeFrameHeader, px_per_mm) == 40);

// doc/frame_transport.md#subscriber-rules
class Frame {
   public:
    const TargetScopeFrameHeader& header() const;
    std::span<const std::uint8_t> data() const;

   private:
    friend class FrameSubscriber;
    struct Impl;
    std::shared_ptr<Impl> impl_;
};

// doc/frame_transport.md#iceoryx2-services
class FramePublisher {
   public:
    FramePublisher(const std::string& service, std::size_t initial_max_len);
    ~FramePublisher();
    FramePublisher(const FramePublisher&) = delete;
    FramePublisher& operator=(const FramePublisher&) = delete;

    void publish(const TargetScopeFrameHeader& header, const std::function<void(std::span<std::uint8_t>)>& fill);

    std::size_t initial_max_len() const;

   private:
    struct Impl;
    std::unique_ptr<Impl> impl_;
};

// doc/frame_transport.md#subscriber-rules
class FrameSubscriber {
   public:
    explicit FrameSubscriber(const std::string& service);
    ~FrameSubscriber();
    FrameSubscriber(const FrameSubscriber&) = delete;
    FrameSubscriber& operator=(const FrameSubscriber&) = delete;

    std::optional<Frame> wait(std::chrono::milliseconds timeout);

   private:
    struct Impl;
    std::unique_ptr<Impl> impl_;
};

// doc/frame_transport.md#stale-shared-memory
void cleanup_dead_nodes();

}
