// doc/testing/overview.md#test-kit
#pragma once

#include <filesystem>
#include <string>

namespace targetscope::testkit {

class FrameFeeder {
   public:
    FrameFeeder(const std::string& service, const std::string& source, double fps, int width, int height);
    ~FrameFeeder();
    void pause();
    void resume();
};

}
