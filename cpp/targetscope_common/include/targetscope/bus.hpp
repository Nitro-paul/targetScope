// Redis bus for the C++ programs (doc/shared_libraries.md#bus).
#pragma once

#include <functional>
#include <stdexcept>
#include <string>

#include <nlohmann/json.hpp>

#include "targetscope/messages.hpp"

namespace targetscope {

// Thrown by handlers; the serve loop turns it into an error response.
struct ServiceError : std::runtime_error {
    ServiceError(ErrorCode code, const std::string& message, nlohmann::json details = nullptr)
        : std::runtime_error(message), code(code), details(std::move(details)) {}
    ErrorCode code;
    nlohmann::json details;
};

// Thrown by Bus::request when the response is an error or doesn't arrive in time.
struct BusError : std::runtime_error {
    using std::runtime_error::runtime_error;
    std::optional<ErrorCode> code;  // remote error code, or Timeout
};

class Bus {
public:
    Bus(const std::string& redis_url, std::string program);
    ~Bus();

    // Sends a request and waits for the response (Req::TIMEOUT).
    template <class Req>
    typename Req::Response request(const Req& req);

    using Handler = std::function<nlohmann::json(const std::string& type, const nlohmann::json& payload)>;

    // Request loop on <program>:requests in its own thread: reads pending, then new requests,
    // calls the handler, sends the response, acknowledges. Answers ping itself.
    void serve(Handler handler);

    void publish(const std::string& type, const nlohmann::json& payload);

    // Plain read of <source>:events after the given entry id ("$" = only new entries).
    using EventHandler = std::function<void(const std::string& type, const nlohmann::json& payload)>;
    void read_events(const std::string& source, std::string after_id, EventHandler handler);

    void stop();

private:
    nlohmann::json request_raw(const char* stream, const char* type, const nlohmann::json& payload,
                               std::chrono::seconds timeout);
    // TODO: redis-plus-plus connection, response reader thread, pending map request_id -> promise
};

template <class Req>
typename Req::Response Bus::request(const Req& req) {
    nlohmann::json payload = req;
    return request_raw(Req::STREAM, Req::TYPE, payload, Req::TIMEOUT).template get<typename Req::Response>();
}

}  // namespace targetscope
