// Processing thread: always subscribes to targetscope/frames/raw and keeps the newest frame;
// runs the live pipeline in live mode; publishes on targetscope/frames/live. Commands arrive
// through a thread-safe queue and are answered with std::promise.
#pragma once

#include <functional>
#include <future>
#include <thread>

#include <nlohmann/json.hpp>

#include "live_pipeline.hpp"
#include "state.hpp"

namespace image_processor {

class ProcessingThread {
public:
    using Command = std::function<nlohmann::json(ProcessorState&, LivePipeline&)>;

    explicit ProcessingThread(LiveEvents events);
    ~ProcessingThread();

    void start();
    void stop();
    // Runs the command on the processing thread between two frames.
    std::future<nlohmann::json> submit(Command cmd);

private:
    void run();
    ProcessorState state_;
    std::thread thread_;
    // TODO: command queue, frame subscriber, live publisher, lens corrector
};

}  // namespace image_processor
