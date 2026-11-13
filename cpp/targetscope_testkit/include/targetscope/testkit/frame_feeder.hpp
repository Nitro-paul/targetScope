// Publishes frames on targetscope/frames/raw or /live from images, videos or the synthetic
// renderer, at a chosen frame rate and resolution.
#pragma once

#include <filesystem>
#include <string>

namespace targetscope::testkit {

class FrameFeeder {
public:
    // source: an image or video file, or "synthetic:<scene>"
    FrameFeeder(const std::string& service, const std::string& source, double fps, int width, int height);
    ~FrameFeeder();
    void pause();   // stop publishing (camera stalls)
    void resume();
};

}  // namespace targetscope::testkit
