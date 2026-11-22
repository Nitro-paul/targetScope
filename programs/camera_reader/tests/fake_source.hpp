// doc/testing/overview.md#camera-sources
#pragma once

#include <chrono>
#include <deque>
#include <memory>
#include <mutex>
#include <stdexcept>
#include <thread>

#include "frame_source.hpp"

namespace camera_reader::testing {

// doc/testing/camera_reader.md#setup
struct FakeCamera {
    struct Step {
        bool fail = false;
        std::chrono::milliseconds stall{0};
    };
    std::mutex mutex;
    SourceInfo info{640, 480, 50.0};
    int fail_open = 0;
    std::deque<Step> script;
    int opens = 0;
    int closes = 0;
    long frames = 0;

    static cv::Mat pattern(int w, int h, long index) {
        cv::Mat m(h, w, CV_8UC3);
        for (int y = 0; y < h; ++y)
            for (int x = 0; x < w; ++x)
                for (int c = 0; c < 3; ++c) m.at<cv::Vec3b>(y, x)[c] = static_cast<uchar>(x + 3 * y + 7 * c + index);
        return m;
    }
};

class FakeSource final : public FrameSource {
   public:
    explicit FakeSource(std::shared_ptr<FakeCamera> camera) : camera_(std::move(camera)) {}

    SourceInfo open() override {
        std::lock_guard lock(camera_->mutex);
        ++camera_->opens;
        if (camera_->fail_open > 0) {
            --camera_->fail_open;
            throw std::runtime_error("fake camera not present");
        }
        return camera_->info;
    }

    void read(cv::Mat& bgr) override {
        FakeCamera::Step step;
        SourceInfo info;
        long index;
        {
            std::lock_guard lock(camera_->mutex);
            info = camera_->info;
            if (!camera_->script.empty()) {
                step = camera_->script.front();
                camera_->script.pop_front();
            }
            index = camera_->frames;
        }
        if (step.stall.count() > 0) std::this_thread::sleep_for(step.stall);
        if (step.fail) {
            std::this_thread::sleep_for(std::chrono::milliseconds(2));
            throw std::runtime_error("fake read failure");
        }
        std::this_thread::sleep_for(std::chrono::microseconds(static_cast<long>(1e6 / info.fps)));
        bgr = FakeCamera::pattern(info.width_px, info.height_px, index);
        std::lock_guard lock(camera_->mutex);
        ++camera_->frames;
    }

    void close() override {
        std::lock_guard lock(camera_->mutex);
        ++camera_->closes;
    }
    bool has_controls() const override { return false; }

   private:
    std::shared_ptr<FakeCamera> camera_;
};

}
