// doc/testing/camera_reader.md#setup
#pragma once

#include <mutex>
#include <vector>

#include "capture_loop.hpp"
#include "fake_source.hpp"
#include "targetscope/frames.hpp"
#include "targetscope/messages.hpp"

namespace camera_reader::testing {

inline std::string unique_service() { return "targetscope/test/" + targetscope::new_uuid(); }

inline Timing fast_timing() { return Timing{std::chrono::milliseconds(100), std::chrono::milliseconds(400), 10}; }

struct Recorder {
    std::mutex mutex;
    std::vector<CameraStatus> states;
    void operator()(const CameraStatus& s) {
        std::lock_guard lock(mutex);
        states.push_back(s);
    }
    std::vector<CameraStatus> get() {
        std::lock_guard lock(mutex);
        return states;
    }
};

template <class F>
bool eventually(F f, std::chrono::milliseconds t = std::chrono::milliseconds(3000)) {
    auto end = std::chrono::steady_clock::now() + t;
    while (std::chrono::steady_clock::now() < end) {
        if (f()) return true;
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
    }
    return f();
}

}
