// doc/testing/overview.md#test-kit
#pragma once

#include <sys/types.h>

#include <chrono>
#include <filesystem>
#include <string>
#include <utility>
#include <vector>

namespace targetscope::testkit {

class ProgramProcess {
   public:
    ProgramProcess(const std::filesystem::path& binary, const std::vector<std::pair<std::string, std::string>>& env);
    ~ProgramProcess();
    ProgramProcess(const ProgramProcess&) = delete;
    ProgramProcess& operator=(const ProgramProcess&) = delete;

    void terminate();
    bool wait_exit(std::chrono::milliseconds timeout);
    int exit_code() const { return exit_code_; }
    pid_t pid() const { return pid_; }

   private:
    pid_t pid_ = -1;
    int exit_code_ = -1;
};

}
