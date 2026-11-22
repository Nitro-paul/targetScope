// doc/programs/image_processor.md#architecture
#pragma once

#include <atomic>
#include <condition_variable>
#include <deque>
#include <functional>
#include <future>
#include <mutex>
#include <nlohmann/json.hpp>
#include <thread>

#include "live_pipeline.hpp"
#include "state.hpp"

namespace image_processor {

class ProcessingThread {
   public:
    using Command = std::function<nlohmann::json(ProcessorState&, LivePipeline&)>;

    ProcessingThread(LiveEvents events, RuntimeProvider runtime, std::string raw_service, std::string live_service);
    ~ProcessingThread();

    void start();
    void stop();
    std::future<nlohmann::json> submit(Command cmd);

   private:
    void run();
    LiveEvents events_;
    RuntimeProvider runtime_;
    std::string raw_service_, live_service_;
    ProcessorState state_;
    LivePipeline pipeline_;
    std::mutex mutex_;
    std::deque<std::pair<Command, std::promise<nlohmann::json>>> queue_;
    std::atomic<bool> stop_{false};
    std::thread thread_;
};

}
