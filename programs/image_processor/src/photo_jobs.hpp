// doc/programs/image_processor.md#architecture
#pragma once

#include <atomic>
#include <condition_variable>
#include <deque>
#include <filesystem>
#include <functional>
#include <future>
#include <mutex>
#include <nlohmann/json.hpp>
#include <thread>
#include <vector>

#include "state.hpp"

namespace image_processor {

class PhotoJobs {
   public:
    PhotoJobs(RuntimeProvider runtime, int threads = 2);
    ~PhotoJobs();

    std::future<nlohmann::json> find_paper(const nlohmann::json& payload);
    std::future<nlohmann::json> prepare_target_photo(const nlohmann::json& payload);
    std::future<nlohmann::json> detect_shots_in_photo(const nlohmann::json& payload);

   private:
    std::future<nlohmann::json> submit(std::function<nlohmann::json()> job);
    void work();
    RuntimeProvider runtime_;
    std::mutex mutex_;
    std::condition_variable cv_;
    std::deque<std::packaged_task<nlohmann::json()>> queue_;
    bool stop_ = false;
    std::vector<std::thread> workers_;
};

}
