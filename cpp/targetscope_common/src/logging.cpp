// doc/shared_libraries.md#logging
#include "targetscope/logging.hpp"

#include <spdlog/sinks/daily_file_sink.h>
#include <spdlog/sinks/stdout_color_sinks.h>
#include <spdlog/spdlog.h>

#include <memory>
#include <vector>

namespace targetscope::logging {

void init(const std::string& program, const BootstrapConfig& config) {
    std::filesystem::create_directories(config.log_dir);
    std::vector<spdlog::sink_ptr> sinks;
    sinks.push_back(std::make_shared<spdlog::sinks::stderr_color_sink_mt>());
    sinks.push_back(std::make_shared<spdlog::sinks::daily_file_format_sink_mt>(
        (config.log_dir / (program + ".log")).string(), 0, 0));
    auto logger = std::make_shared<spdlog::logger>(program, sinks.begin(), sinks.end());
    logger->set_level(spdlog::level::from_str(config.log_level));
    spdlog::set_default_logger(logger);
    spdlog::flush_every(std::chrono::seconds(1));
}

}
