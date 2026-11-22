// doc/shared_libraries.md#bus
#pragma once

#include <atomic>
#include <chrono>
#include <functional>
#include <memory>
#include <nlohmann/json.hpp>
#include <optional>
#include <stdexcept>
#include <string>

#include "targetscope/clock.hpp"
#include "targetscope/messages.hpp"

namespace targetscope {

struct BusError : std::runtime_error {
    using std::runtime_error::runtime_error;
    std::optional<ErrorCode> code;
    ServiceError to_service_error() const;
};

// doc/testing/overview.md#test-kit
struct NoResponse {};

// doc/interfaces.md#streams-and-keys
inline constexpr long long MAXLEN_REQUESTS = 10000;
inline constexpr long long MAXLEN_EVENTS = 10000;
inline constexpr long long MAXLEN_RESPONSES = 1000;

class Bus {
   public:
    Bus(const std::string& redis_url, std::string program, std::shared_ptr<const Clock> clock = nullptr);
    ~Bus();
    Bus(const Bus&) = delete;
    Bus& operator=(const Bus&) = delete;

    template <class Req>
    typename Req::Response request(const Req& req);

    using Handler = std::function<nlohmann::json(const std::string& type, const nlohmann::json& payload)>;

    void serve(Handler handler);

    void publish(const std::string& type, const nlohmann::json& payload);

    std::string newest_event_id(const std::string& source);

    using EventHandler = std::function<void(const std::string& type, const nlohmann::json& payload)>;
    void read_events(const std::string& source, std::string after_id, EventHandler handler,
                     const std::atomic<bool>* cancel = nullptr);

    nlohmann::json request_raw(const char* stream, const char* type, const nlohmann::json& payload,
                               std::chrono::seconds timeout, const std::string& request_id = {});

    void stop();
    bool is_stopping() const;
    std::uint64_t reconnects() const;
    const std::string& program() const;

   private:
    struct Impl;
    std::unique_ptr<Impl> impl_;
};

template <class Req>
typename Req::Response Bus::request(const Req& req) {
    nlohmann::json payload = req.payload();
    return request_raw(Req::STREAM, Req::TYPE, payload, Req::TIMEOUT).template get<typename Req::Response>();
}

}
