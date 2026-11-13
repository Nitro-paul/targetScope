// FakeSource for the unit tests: generated frames, scripted read failures and stalls
// (doc/testing/overview.md#camera-sources).
#pragma once

#include <chrono>
#include <deque>

#include "frame_source.hpp"

namespace camera_reader::testing {

class FakeSource final : public FrameSource {
public:
    struct Step {
        bool fail = false;
        std::chrono::milliseconds stall{0};
    };
    SourceInfo info{640, 480, 30.0};
    bool fail_open = false;
    std::deque<Step> script;  // consumed per read(); empty = good frames

    SourceInfo open() override { return info; /* TODO: fail_open */ }
    void read(cv::Mat& bgr) override { (void)bgr; /* TODO */ }
    void close() override {}
    bool has_controls() const override { return false; }
};

}  // namespace camera_reader::testing
