// doc/shared_libraries.md#bus
#pragma once

#include <functional>
#include <nlohmann/json.hpp>
#include <optional>
#include <stdexcept>
#include <string>

#include "targetscope/messages.hpp"

namespace targetscope {

struct BusError : std::runtime_error {
    using std::runtime_error::runtime_error;
    std::optional<ErrorCode> code;
};

class Bus {
   public:
    Bus(const std::string& redis_url, std::string program);
    ~Bus();

    template <class Req>
    typename Req::Response request(const Req& req);

    using Handler = std::function<nlohmann::json(const std::string& type, const nlohmann::json& payload)>;

    void serve(Handler handler);

    void publish(const std::string& type, const nlohmann::json& payload);

    using EventHandler = std::function<void(const std::string& type, const nlohmann::json& payload)>;
    void read_events(const std::string& source, std::string after_id, EventHandler handler);

    void stop();

   private:
    nlohmann::json request_raw(const char* stream, const char* type, const nlohmann::json& payload,
                               std::chrono::seconds timeout);
};

template <class Req>
typename Req::Response Bus::request(const Req& req) {
    nlohmann::json payload = req.payload();
    return request_raw(Req::STREAM, Req::TYPE, payload, Req::TIMEOUT).template get<typename Req::Response>();
}

}
