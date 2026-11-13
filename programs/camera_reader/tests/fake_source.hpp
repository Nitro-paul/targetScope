// doc/testing/overview.md#camera-sources
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
    std::deque<Step> script;

    SourceInfo open() override { return info; }
    void read(cv::Mat& bgr) override { (void)bgr; }
    void close() override {}
    bool has_controls() const override { return false; }
};

}
