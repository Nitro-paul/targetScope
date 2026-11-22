// doc/programs/image_processor.md#architecture
#include "processing_thread.hpp"

#include <spdlog/spdlog.h>

#include <cstring>
#include <opencv2/imgproc.hpp>

#include "targetscope/frames.hpp"
#include "targetscope/messages.hpp"

namespace image_processor {

using namespace std::chrono_literals;

ProcessingThread::ProcessingThread(LiveEvents events, RuntimeProvider runtime, std::string raw_service,
                                   std::string live_service)
    : events_(events),
      runtime_(std::move(runtime)),
      raw_service_(std::move(raw_service)),
      live_service_(std::move(live_service)),
      pipeline_(state_, events) {}

ProcessingThread::~ProcessingThread() { stop(); }

void ProcessingThread::start() {
    thread_ = std::thread([this] { run(); });
}

void ProcessingThread::stop() {
    stop_ = true;
    if (thread_.joinable()) thread_.join();
}

std::future<nlohmann::json> ProcessingThread::submit(Command cmd) {
    std::promise<nlohmann::json> promise;
    auto future = promise.get_future();
    {
        std::lock_guard lock(mutex_);
        queue_.emplace_back(std::move(cmd), std::move(promise));
    }
    return future;
}

// doc/programs/image_processor.md#architecture
void ProcessingThread::run() {
    std::unique_ptr<targetscope::frames::FrameSubscriber> subscriber;
    std::unique_ptr<targetscope::frames::FramePublisher> publisher;
    std::uint64_t live_seq = 0;
    cv::Size publisher_size;
    auto backoff = 200ms;
    while (!stop_) {
        for (;;) {
            std::pair<Command, std::promise<nlohmann::json>> item;
            {
                std::lock_guard lock(mutex_);
                if (queue_.empty()) break;
                item = std::move(queue_.front());
                queue_.pop_front();
            }
            try {
                state_.runtime = runtime_();
                item.second.set_value(item.first(state_, pipeline_));
            } catch (...) {
                item.second.set_exception(std::current_exception());
            }
        }
        if (!subscriber) {
            try {
                subscriber = std::make_unique<targetscope::frames::FrameSubscriber>(raw_service_);
            } catch (const std::exception& e) {
                spdlog::warn("subscribing to the raw frames failed error={}", e.what());
                std::this_thread::sleep_for(backoff);
                continue;
            }
        }
        std::optional<targetscope::frames::Frame> frame;
        try {
            frame = subscriber->wait(30ms);
        } catch (const std::exception& e) {
            spdlog::warn("receiving frames failed error={}", e.what());
            subscriber.reset();
            continue;
        }
        const auto now = std::chrono::steady_clock::now();
        if (!frame) {
            if (state_.live && state_.last_frame_time != std::chrono::steady_clock::time_point{} &&
                now - state_.last_frame_time > 2s) {
                pipeline_.frames_stopped();
            }
            continue;
        }
        const auto& h = frame->header();
        if (h.format != TARGETSCOPE_PIXEL_BGR8 || h.width == 0 || h.height == 0) continue;
        {
            cv::Mat view(static_cast<int>(h.height), static_cast<int>(h.width), CV_8UC3,
                         const_cast<std::uint8_t*>(frame->data().data()), h.stride);
            view.copyTo(state_.newest_raw);
        }
        state_.newest_raw_seq = h.seq;
        state_.last_frame_time = now;
        state_.runtime = runtime_();
        const auto timestamp = h.timestamp_ns;
        const auto seq = h.seq;
        frame.reset();
        if (!state_.live) continue;
        try {
            auto out = pipeline_.process(state_.newest_raw, seq);
            if (!out) continue;
            if (!publisher || publisher_size != out->size()) {
                publisher = std::make_unique<targetscope::frames::FramePublisher>(
                    live_service_, static_cast<std::size_t>(out->cols) * out->rows * 3);
                publisher_size = out->size();
            }
            TargetScopeFrameHeader lh{};
            lh.seq = live_seq++;
            lh.timestamp_ns = timestamp;
            lh.source_seq = seq;
            lh.width = static_cast<std::uint32_t>(out->cols);
            lh.height = static_cast<std::uint32_t>(out->rows);
            lh.stride = static_cast<std::uint32_t>(out->cols) * 3;
            lh.format = TARGETSCOPE_PIXEL_BGR8;
            lh.px_per_mm = pipeline_.px_per_mm();
            publisher->publish(lh, [&](std::span<std::uint8_t> dst) {
                for (int y = 0; y < out->rows; ++y)
                    std::memcpy(dst.data() + static_cast<std::size_t>(y) * lh.stride, out->ptr(y), lh.stride);
            });
        } catch (const std::exception& e) {
            spdlog::error("processing a frame failed error={}", e.what());
        }
    }
}

}
