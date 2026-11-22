// doc/shared_libraries.md#redis-bus
#include "targetscope/bus.hpp"

#include <spdlog/spdlog.h>
#include <sw/redis++/redis++.h>
#include <unistd.h>

#include <condition_variable>
#include <future>
#include <map>
#include <mutex>
#include <thread>
#include <unordered_map>

#include "targetscope/failpoint.hpp"

#ifndef TARGETSCOPE_VERSION
#define TARGETSCOPE_VERSION "0.0.0"
#endif

namespace targetscope {

namespace {

using namespace std::chrono_literals;
namespace sr = sw::redis;

constexpr auto BLOCK = 500ms;
constexpr long long BATCH = 100;
constexpr auto MIN_BACKOFF = 100ms;
constexpr auto MAX_BACKOFF = 5000ms;

using Attrs = std::vector<std::pair<std::string, std::string>>;
using Item = std::pair<std::string, sr::Optional<Attrs>>;
using ItemStream = std::vector<Item>;
using ReadResult = std::unordered_map<std::string, ItemStream>;

struct Entry {
    std::string id;
    std::string data;
};

// doc/shared_libraries.md#bootstrap-configuration
sr::ConnectionOptions parse_url(const std::string& url, std::chrono::milliseconds socket_timeout) {
    sr::ConnectionOptions o;
    std::string rest = url;
    if (auto p = rest.find("://"); p != std::string::npos) rest = rest.substr(p + 3);
    if (auto at = rest.find('@'); at != std::string::npos) {
        std::string cred = rest.substr(0, at);
        rest = rest.substr(at + 1);
        if (auto c = cred.find(':'); c != std::string::npos) {
            o.user = cred.substr(0, c);
            o.password = cred.substr(c + 1);
        } else {
            o.password = cred;
        }
    }
    if (auto s = rest.find('/'); s != std::string::npos) {
        std::string db = rest.substr(s + 1);
        rest = rest.substr(0, s);
        if (!db.empty()) o.db = std::stoi(db);
    }
    o.port = 6379;
    if (auto c = rest.rfind(':'); c != std::string::npos) {
        o.port = std::stoi(rest.substr(c + 1));
        rest = rest.substr(0, c);
    }
    o.host = rest.empty() ? "127.0.0.1" : rest;
    o.socket_timeout = socket_timeout;
    o.connect_timeout = 2000ms;
    return o;
}

std::unique_ptr<sr::Redis> connect(const std::string& url, std::chrono::milliseconds socket_timeout,
                                   std::size_t pool = 1) {
    sr::ConnectionPoolOptions pool_options;
    pool_options.size = pool;
    pool_options.wait_timeout = 2000ms;
    return std::make_unique<sr::Redis>(parse_url(url, socket_timeout), pool_options);
}

std::vector<Entry> flatten(const ReadResult& r) {
    std::vector<Entry> out;
    for (const auto& [stream, items] : r) {
        for (const auto& [id, attrs] : items) {
            Entry e{id, {}};
            if (attrs)
                for (const auto& [k, v] : *attrs)
                    if (k == "data") e.data = v;
            out.push_back(std::move(e));
        }
    }
    return out;
}

std::string hostname() {
    char buf[256] = {};
    if (gethostname(buf, sizeof buf - 1) != 0) return "localhost";
    return buf;
}

}

ServiceError BusError::to_service_error() const {
    if (code) return ServiceError(*code, what());
    return ServiceError(ErrorCode::Unavailable, what());
}

struct Bus::Impl {
    std::string url;
    std::string program;
    std::string consumer;
    std::shared_ptr<const Clock> clock;
    std::int64_t started_at = 0;
    std::unique_ptr<sr::Redis> cmd;

    std::mutex pending_mutex;
    std::unordered_map<std::string, std::promise<ResponseEnvelope>> pending;

    std::atomic<bool> stopping{false};
    std::atomic<std::uint64_t> reconnect_count{0};
    std::thread response_thread;
    std::thread serve_thread;
    std::mutex stop_mutex;
    std::condition_variable stop_cv;

    void sleep_interruptible(std::chrono::milliseconds d) {
        std::unique_lock lock(stop_mutex);
        stop_cv.wait_for(lock, d, [&] { return stopping.load(); });
    }

    std::string stream(const std::string& program_name, const char* kind) const { return program_name + ":" + kind; }

    void xadd(const std::string& key, long long maxlen, const std::string& data) {
        std::vector<std::pair<std::string, std::string>> fields{{"data", data}};
        cmd->xadd(key, "*", fields.begin(), fields.end(), maxlen, true);
    }

    std::string newest_id(sr::Redis& r, const std::string& key) {
        std::vector<Item> items;
        r.xrevrange(key, "+", "-", 1, std::back_inserter(items));
        return items.empty() ? "0-0" : items.front().first;
    }

    void read_responses();
    void serve_loop(Handler handler);
    void create_group(sr::Redis& r, const std::string& stream_key);
    void claim_all(sr::Redis& r, const std::string& stream_key);
    void handle_entry(const Entry& entry, const std::string& stream_key, const Handler& handler);
    std::optional<std::pair<std::string, ResponseEnvelope>> process(const json& raw, const Handler& handler);
};

void Bus::Impl::create_group(sr::Redis& r, const std::string& stream_key) {
    try {
        r.xgroup_create(stream_key, program, "0", true);
    } catch (const sr::Error& e) {
        if (std::string(e.what()).find("BUSYGROUP") == std::string::npos) throw;
    }
}

// doc/shared_libraries.md#redis-bus
void Bus::Impl::claim_all(sr::Redis& r, const std::string& stream_key) {
    std::string cursor = "0";
    do {
        auto reply =
            r.command("XAUTOCLAIM", stream_key, program, consumer, "0", cursor, "COUNT", std::to_string(BATCH));
        if (!reply || reply->type != REDIS_REPLY_ARRAY || reply->elements < 1 ||
            reply->element[0]->type != REDIS_REPLY_STRING)
            break;
        cursor = reply->element[0]->str;
    } while (cursor != "0-0" && cursor != "0");
}

void Bus::Impl::read_responses() {
    const std::string key = stream(program, "responses");
    std::unique_ptr<sr::Redis> r;
    std::string last_id;
    auto backoff = MIN_BACKOFF;
    while (!stopping) {
        try {
            if (!r) {
                r = connect(url, 5000ms);
                if (last_id.empty()) last_id = newest_id(*r, key);
            }
            ReadResult result;
            r->xread(key, last_id, BLOCK, BATCH, std::inserter(result, result.end()));
            backoff = MIN_BACKOFF;
            for (const auto& e : flatten(result)) {
                last_id = e.id;
                ResponseEnvelope resp;
                try {
                    resp = parse_response(json::parse(e.data));
                } catch (const std::exception&) {
                    spdlog::error("unparseable response entry entry={}", e.id);
                    continue;
                }
                std::promise<ResponseEnvelope> waiting;
                bool found = false;
                {
                    std::lock_guard lock(pending_mutex);
                    auto it = pending.find(resp.request_id);
                    if (it != pending.end()) {
                        waiting = std::move(it->second);
                        pending.erase(it);
                        found = true;
                    }
                }
                if (found)
                    waiting.set_value(std::move(resp));
                else
                    spdlog::warn("late response dropped request_id={}", resp.request_id);
            }
        } catch (const sr::Error& ex) {
            spdlog::warn("reading responses failed error={}", ex.what());
            r.reset();
            sleep_interruptible(backoff);
            backoff = std::min(backoff * 2, MAX_BACKOFF);
        }
    }
}

Bus::Bus(const std::string& redis_url, std::string program, std::shared_ptr<const Clock> clock)
    : impl_(std::make_unique<Impl>()) {
    impl_->url = redis_url;
    impl_->program = std::move(program);
    impl_->consumer = hostname() + "-" + std::to_string(getpid());
    impl_->clock = clock ? std::move(clock) : std::make_shared<SystemClock>();
    impl_->started_at =
        std::chrono::duration_cast<std::chrono::seconds>(impl_->clock->now().time_since_epoch()).count();
    try {
        impl_->cmd = connect(redis_url, 5000ms, 4);
        impl_->cmd->ping();
    } catch (const sr::Error& e) {
        BusError err(std::string("redis: ") + e.what());
        err.code = ErrorCode::Unavailable;
        throw err;
    }
    impl_->response_thread = std::thread([this] { impl_->read_responses(); });
}

Bus::~Bus() { stop(); }

void Bus::stop() {
    {
        std::lock_guard lock(impl_->stop_mutex);
        impl_->stopping = true;
    }
    impl_->stop_cv.notify_all();
    if (impl_->serve_thread.joinable() && impl_->serve_thread.get_id() != std::this_thread::get_id())
        impl_->serve_thread.join();
    if (impl_->response_thread.joinable() && impl_->response_thread.get_id() != std::this_thread::get_id())
        impl_->response_thread.join();
}

bool Bus::is_stopping() const { return impl_->stopping; }
std::uint64_t Bus::reconnects() const { return impl_->reconnect_count; }
const std::string& Bus::program() const { return impl_->program; }

// doc/interfaces.md#timeouts
json Bus::request_raw(const char* stream, const char* type, const json& payload, std::chrono::seconds timeout,
                      const std::string& request_id) {
    RequestEnvelope env;
    env.type = type;
    env.request_id = request_id.empty() ? new_uuid() : request_id;
    env.reply_to = impl_->stream(impl_->program, "responses");
    env.payload = payload;
    std::promise<ResponseEnvelope> promise;
    auto future = promise.get_future();
    {
        std::lock_guard lock(impl_->pending_mutex);
        impl_->pending[env.request_id] = std::move(promise);
    }
    try {
        impl_->xadd(stream, MAXLEN_REQUESTS, to_json(env).dump());
    } catch (const sr::Error& e) {
        std::lock_guard lock(impl_->pending_mutex);
        impl_->pending.erase(env.request_id);
        BusError err(std::string("redis: ") + e.what());
        err.code = ErrorCode::Unavailable;
        throw err;
    }
    if (future.wait_for(timeout) != std::future_status::ready) {
        std::lock_guard lock(impl_->pending_mutex);
        impl_->pending.erase(env.request_id);
        BusError err(std::string("no response in time: ") + type);
        err.code = ErrorCode::Timeout;
        throw err;
    }
    ResponseEnvelope resp = future.get();
    if (!resp.ok) {
        std::string message = resp.error.value("message", std::string("error response"));
        BusError err(message);
        err.code = error_code_from(resp.error.value("code", std::string())).value_or(ErrorCode::InvalidMessage);
        throw err;
    }
    return resp.payload;
}

// doc/interfaces.md#event-envelope
void Bus::publish(const std::string& type, const json& payload) {
    EventEnvelope env;
    env.type = type;
    env.event_id = new_uuid();
    env.source = impl_->program;
    env.timestamp_ms = unix_ms(impl_->clock->now());
    env.payload = payload;
    try {
        impl_->xadd(impl_->stream(impl_->program, "events"), MAXLEN_EVENTS, to_json(env).dump());
    } catch (const sr::Error& e) {
        spdlog::warn("publishing event failed type={} error={}", type, e.what());
    }
}

// doc/interfaces.md#who-listens-to-which-events
std::string Bus::newest_event_id(const std::string& source) {
    try {
        return impl_->newest_id(*impl_->cmd, impl_->stream(source, "events"));
    } catch (const sr::Error& e) {
        BusError err(std::string("redis: ") + e.what());
        err.code = ErrorCode::Unavailable;
        throw err;
    }
}

void Bus::read_events(const std::string& source, std::string after_id, EventHandler handler,
                      const std::atomic<bool>* cancel) {
    const std::string key = impl_->stream(source, "events");
    std::unique_ptr<sr::Redis> r;
    auto backoff = MIN_BACKOFF;
    auto done = [&] { return impl_->stopping.load() || (cancel && cancel->load()); };
    while (!done()) {
        try {
            if (!r) r = connect(impl_->url, 5000ms);
            ReadResult result;
            r->xread(key, after_id, BLOCK, BATCH, std::inserter(result, result.end()));
            backoff = MIN_BACKOFF;
            for (const auto& e : flatten(result)) {
                after_id = e.id;
                try {
                    EventEnvelope env = parse_event(json::parse(e.data));
                    if (env.version != MESSAGE_VERSION) continue;
                    handler(env.type, env.payload);
                } catch (const json::exception&) {
                    spdlog::warn("skipping unparseable event entry={}", e.id);
                }
            }
        } catch (const sr::Error& ex) {
            spdlog::warn("reading events failed source={} error={}", source, ex.what());
            r.reset();
            impl_->sleep_interruptible(backoff);
            backoff = std::min(backoff * 2, MAX_BACKOFF);
        }
    }
}

// doc/shared_libraries.md#redis-bus
void Bus::serve(Handler handler) {
    impl_->serve_thread = std::thread([this, handler = std::move(handler)] { impl_->serve_loop(handler); });
}

std::optional<std::pair<std::string, ResponseEnvelope>> Bus::Impl::process(const json& raw, const Handler& handler) {
    RequestEnvelope env;
    try {
        env = parse_request(raw);
    } catch (const ServiceError& e) {
        spdlog::error("invalid request envelope error={}", e.what());
        if (!raw.is_object() || !raw.contains("reply_to") || !raw["reply_to"].is_string() ||
            !raw.contains("request_id") || !raw["request_id"].is_string())
            return std::nullopt;
        RequestEnvelope stub;
        stub.type = raw.value("type", std::string("unknown"));
        stub.request_id = raw["request_id"];
        return std::pair{raw["reply_to"].get<std::string>(),
                         error_response(stub, ServiceError(ErrorCode::InvalidMessage, e.what()))};
    }
    ResponseEnvelope resp;
    try {
        if (env.version != MESSAGE_VERSION)
            throw ServiceError(ErrorCode::InvalidMessage, "unsupported version " + std::to_string(env.version));
        if (env.type == "ping") {
            resp = ok_response(env, json{{"program", program},
                                         {"program_version", TARGETSCOPE_VERSION},
                                         {"language", "cpp"},
                                         {"started_at", started_at}});
        } else {
            resp = ok_response(env, handler(env.type, env.payload));
        }
    } catch (const NoResponse&) {
        return std::nullopt;
    } catch (const ServiceError& e) {
        spdlog::info("request{{request_id={} type={}}}: request failed code=\"{}\" {}", env.request_id, env.type,
                     to_string(e.code), e.what());
        resp = error_response(env, e);
    } catch (const std::exception& e) {
        spdlog::error("request{{request_id={} type={}}}: handler failed {}", env.request_id, env.type, e.what());
        resp = error_response(env, ServiceError(ErrorCode::ProcessingFailed, e.what()));
    }
    if (!env.reply_to) return std::nullopt;
    return std::pair{*env.reply_to, std::move(resp)};
}

void Bus::Impl::handle_entry(const Entry& entry, const std::string& stream_key, const Handler& handler) {
    std::optional<std::pair<std::string, ResponseEnvelope>> response;
    try {
        response = process(json::parse(entry.data), handler);
    } catch (const json::exception&) {
        spdlog::error("unparseable request entry entry={}", entry.id);
    }
    if (TS_FAILPOINT("bus.before_response")) return;
    if (response) xadd(response->first, MAXLEN_RESPONSES, to_json(response->second).dump());
    if (TS_FAILPOINT("bus.before_ack")) return;
    cmd->xack(stream_key, program, entry.id);
}

void Bus::Impl::serve_loop(Handler handler) {
    const std::string key = stream(program, "requests");
    std::unique_ptr<sr::Redis> r;
    bool pending_phase = true;
    bool first = true;
    auto backoff = MIN_BACKOFF;
    while (!stopping) {
        try {
            if (!r) {
                r = connect(url, 5000ms);
                create_group(*r, key);
                claim_all(*r, key);
                pending_phase = true;
                if (!first) ++reconnect_count;
                first = false;
            }
            ReadResult result;
            r->xreadgroup(program, consumer, key, pending_phase ? "0" : ">", BLOCK, BATCH,
                          std::inserter(result, result.end()));
            backoff = MIN_BACKOFF;
            auto entries = flatten(result);
            if (pending_phase && entries.empty()) pending_phase = false;
            for (const auto& e : entries) handle_entry(e, key, handler);
        } catch (const sr::Error& ex) {
            spdlog::warn("reading requests failed error={}", ex.what());
            r.reset();
            sleep_interruptible(backoff);
            backoff = std::min(backoff * 2, MAX_BACKOFF);
        }
    }
}

}
