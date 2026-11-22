// doc/shared_libraries.md#settings-cache
#include "targetscope/settings.hpp"

#include <spdlog/spdlog.h>

#include <set>

#include "targetscope/config.hpp"
#include "targetscope/failpoint.hpp"

namespace targetscope {

namespace {
constexpr const char* DATABASE_HANDLER = "database_handler";

model::Filter name_filter(const std::string& name) {
    model::Filter f;
    f.conditions.push_back(model::Condition{"name", "eq", name});
    return f;
}
}

// doc/shared_libraries.md#settings-cache
SettingsCache::SettingsCache(Bus& bus) : bus_(bus) {
    std::string after = bus.newest_event_id(DATABASE_HANDLER);
    if (TS_FAILPOINT("settings.after_note_id")) throw BusError("failpoint settings.after_note_id");
    auto all = bus.request(messages::ReadSettings{}).settings;
    for (auto& s : all) settings_[s.name] = std::move(s);
    thread_ = std::thread([this, after = std::move(after)] { follow(after); });
}

SettingsCache::~SettingsCache() {
    cancel_ = true;
    if (thread_.joinable()) thread_.join();
}

std::optional<std::string> SettingsCache::get(const std::string& name) const {
    std::lock_guard lock(mutex_);
    auto it = settings_.find(name);
    if (it == settings_.end() || !it->second.active) return std::nullopt;
    return it->second.value;
}

std::optional<std::filesystem::path> SettingsCache::get_path(const std::string& name) const {
    auto v = get(name);
    if (!v || v->empty()) return std::nullopt;
    return expand_home(*v);
}

void SettingsCache::watch(std::vector<std::string> names, std::function<void()> callback) {
    std::lock_guard lock(mutex_);
    watchers_.emplace_back(std::move(names), std::move(callback));
}

void SettingsCache::replace(const std::optional<std::string>& name, std::vector<model::Setting> fresh) {
    std::vector<std::string> changed;
    std::vector<std::function<void()>> to_call;
    {
        std::lock_guard lock(mutex_);
        std::unordered_map<std::string, model::Setting> incoming;
        for (auto& s : fresh) incoming[s.name] = std::move(s);
        std::set<std::string> names;
        if (name) {
            names.insert(*name);
        } else {
            for (auto& [k, v] : settings_) names.insert(k);
            for (auto& [k, v] : incoming) names.insert(k);
        }
        auto effective = [](const model::Setting* s) -> std::optional<std::string> {
            if (s && s->active) return s->value;
            return std::nullopt;
        };
        for (const auto& n : names) {
            auto old_it = settings_.find(n);
            auto new_it = incoming.find(n);
            const model::Setting* old_s = old_it == settings_.end() ? nullptr : &old_it->second;
            const model::Setting* new_s = new_it == incoming.end() ? nullptr : &new_it->second;
            if (effective(old_s) != effective(new_s)) changed.push_back(n);
            if (new_s)
                settings_[n] = *new_s;
            else
                settings_.erase(n);
        }
        for (auto& [names_watched, cb] : watchers_)
            for (const auto& n : names_watched)
                if (std::find(changed.begin(), changed.end(), n) != changed.end()) {
                    to_call.push_back(cb);
                    break;
                }
    }
    for (auto& cb : to_call) cb();
}

void SettingsCache::follow(std::string after) {
    bus_.read_events(
        DATABASE_HANDLER, std::move(after),
        [this](const std::string& type, const nlohmann::json& payload) {
            if (type != "data_changed") return;
            std::optional<std::string> name;
            const std::string table = payload.value("table", std::string());
            if (table == "settings") {
                if (!payload.contains("key") || !payload["key"].is_string()) return;
                name = payload["key"].get<std::string>();
            } else if (table != "all") {
                return;
            }
            try {
                model::Filter filter = name ? name_filter(*name) : model::Filter{};
                auto fresh = bus_.request(messages::ReadSettings{filter}).settings;
                replace(name, std::move(fresh));
            } catch (const std::exception& e) {
                spdlog::warn("reading settings failed error={}", e.what());
            }
        },
        &cancel_);
}

}
