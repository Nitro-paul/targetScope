// Settings cache (doc/shared_libraries.md#settings-cache), same behavior as in Rust.
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
    // Notes the newest id of database_handler:events, reads all settings, then follows
    // data_changed events for the settings table (table "all" reloads everything).
    explicit SettingsCache(Bus& bus);

    // nullopt if the setting is unset (active = false) or unknown.
    std::optional<std::string> get(const std::string& name) const;
    std::optional<std::filesystem::path> get_path(const std::string& name) const;  // expands "~"

    // Called (from the event thread) when one of the given settings changes.
    void watch(std::vector<std::string> names, std::function<void()> callback);

private:
    mutable std::mutex mutex_;
    std::unordered_map<std::string, model::Setting> settings_;
    // TODO: watchers
};

}  // namespace targetscope
