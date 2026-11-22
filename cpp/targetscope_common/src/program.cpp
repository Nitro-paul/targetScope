// doc/shared_libraries.md#program-startup
#include "targetscope/program.hpp"

#include <spdlog/spdlog.h>

#include <atomic>
#include <csignal>
#include <thread>

#include "targetscope/frames.hpp"
#include "targetscope/logging.hpp"

namespace targetscope::program {

namespace {
std::atomic<bool> g_shutdown{false};
void on_signal(int) { g_shutdown = true; }
}

bool shutting_down() { return g_shutdown; }

void request_shutdown() { g_shutdown = true; }

bool wait_for_shutdown(std::chrono::milliseconds timeout) {
    auto end = std::chrono::steady_clock::now() + timeout;
    while (!g_shutdown && std::chrono::steady_clock::now() < end)
        std::this_thread::sleep_for(std::chrono::milliseconds(20));
    return g_shutdown;
}

// doc/shared_libraries.md#program-startup
Started start(const std::string& name) {
    struct sigaction sa{};
    sa.sa_handler = on_signal;
    sigemptyset(&sa.sa_mask);
    sigaction(SIGTERM, &sa, nullptr);
    sigaction(SIGINT, &sa, nullptr);
    signal(SIGPIPE, SIG_IGN);

    Started s;
    s.config = BootstrapConfig::from_env();
    logging::init(name, s.config);
    try {
        frames::cleanup_dead_nodes();
    } catch (const std::exception& e) {
        spdlog::warn("cleaning up iceoryx2 resources failed error={}", e.what());
    }
    auto backoff = std::chrono::milliseconds(100);
    while (!g_shutdown) {
        try {
            s.bus = std::make_unique<Bus>(s.config.redis_url, name);
            break;
        } catch (const BusError& e) {
            spdlog::warn("redis not reachable error={}", e.what());
            wait_for_shutdown(backoff);
            backoff = std::min(backoff * 2, std::chrono::milliseconds(5000));
        }
    }
    if (s.bus) spdlog::info("started program=\"{}\"", name);
    return s;
}

}
