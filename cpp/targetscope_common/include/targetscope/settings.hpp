// doc/shared_libraries.md#settings-cache
#pragma once

#include <filesystem>
#include <functional>
#include <mutex>
#include <optional>
#include <string>
#include <unordered_map>
#include <vector>

#include "targetscope/bus.hpp"
#include "targetscope/model.hpp"

namespace targetscope {

class SettingsCache {
   public:
    explicit SettingsCache(Bus& bus);

    std::optional<std::string> get(const std::string& name) const;
    std::optional<std::filesystem::path> get_path(const std::string& name) const;

    void watch(std::vector<std::string> names, std::function<void()> callback);

   private:
    mutable std::mutex mutex_;
    std::unordered_map<std::string, model::Setting> settings_;
};

}
