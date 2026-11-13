// doc/programs/image_processor.md#architecture
#pragma once

#include <functional>
#include <future>
#include <nlohmann/json.hpp>
#include <thread>

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
    std::future<nlohmann::json> submit(Command cmd);

   private:
    void run();
    ProcessorState state_;
    std::thread thread_;
};

}
