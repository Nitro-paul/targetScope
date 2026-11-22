// doc/shared_libraries.md#program-startup
#pragma once

#include <chrono>
#include <memory>
#include <string>

#include "targetscope/bus.hpp"
#include "targetscope/config.hpp"

namespace targetscope::program {

struct Started {
    BootstrapConfig config;
    std::unique_ptr<Bus> bus;
};

Started start(const std::string& name);

bool shutting_down();
bool wait_for_shutdown(std::chrono::milliseconds timeout);
void request_shutdown();

}
