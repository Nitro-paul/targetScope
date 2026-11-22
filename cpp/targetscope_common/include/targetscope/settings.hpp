// doc/shared_libraries.md#settings-cache
#pragma once

#include <atomic>
#include <filesystem>
#include <functional>
#include <mutex>
#include <optional>
#include <string>
#include <thread>
#include <unordered_map>
#include <vector>

#include "targetscope/bus.hpp"
#include "targetscope/model.hpp"

namespace targetscope {

class SettingsCache {
   public:
    explicit SettingsCache(Bus& bus);
    ~SettingsCache();
    SettingsCache(const SettingsCache&) = delete;
    SettingsCache& operator=(const SettingsCache&) = delete;

    std::optional<std::string> get(const std::string& name) const;
    std::optional<std::filesystem::path> get_path(const std::string& name) const;

    template <class T>
    std::optional<T> get_parsed(const std::string& name) const;

    void watch(std::vector<std::string> names, std::function<void()> callback);

   private:
    void follow(std::string after);
    void replace(const std::optional<std::string>& name, std::vector<model::Setting> fresh);

    Bus& bus_;
    mutable std::mutex mutex_;
    std::unordered_map<std::string, model::Setting> settings_;
    std::vector<std::pair<std::vector<std::string>, std::function<void()>>> watchers_;
    std::atomic<bool> cancel_{false};
    std::thread thread_;
};

template <class T>
std::optional<T> SettingsCache::get_parsed(const std::string& name) const {
    auto v = get(name);
    if (!v) return std::nullopt;
    try {
        if constexpr (std::is_same_v<T, double>) {
            std::size_t used = 0;
            double d = std::stod(*v, &used);
            if (used == 0) return std::nullopt;
            return d;
        } else {
            std::size_t used = 0;
            long long n = std::stoll(*v, &used);
            if (used == 0) return std::nullopt;
            return static_cast<T>(n);
        }
    } catch (const std::exception&) {
        return std::nullopt;
    }
}

}
