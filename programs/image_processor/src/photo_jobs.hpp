// doc/programs/image_processor.md#architecture
#pragma once

#include <filesystem>
#include <functional>
#include <future>
#include <nlohmann/json.hpp>

namespace image_processor {

class PhotoJobs {
   public:
    PhotoJobs(std::filesystem::path staging_dir, int threads = 2);
    ~PhotoJobs();

    std::future<nlohmann::json> find_paper(const nlohmann::json& payload);
    std::future<nlohmann::json> prepare_target_photo(const nlohmann::json& payload);
    std::future<nlohmann::json> detect_shots_in_photo(const nlohmann::json& payload);

   private:
    std::filesystem::path staging_dir_;
};

}
