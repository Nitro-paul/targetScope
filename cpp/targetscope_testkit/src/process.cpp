// doc/testing/overview.md#test-kit
#include "targetscope/testkit/process.hpp"

#include <fcntl.h>
#include <signal.h>
#include <sys/wait.h>
#include <unistd.h>

#include <stdexcept>
#include <thread>

namespace targetscope::testkit {

ProgramProcess::ProgramProcess(const std::filesystem::path& binary,
                               const std::vector<std::pair<std::string, std::string>>& env) {
    pid_ = fork();
    if (pid_ < 0) throw std::runtime_error("fork failed");
    if (pid_ == 0) {
        for (const auto& [k, v] : env) setenv(k.c_str(), v.c_str(), 1);
        execl(binary.c_str(), binary.c_str(), static_cast<char*>(nullptr));
        _exit(127);
    }
}

ProgramProcess::~ProgramProcess() {
    if (pid_ > 0) {
        kill(pid_, SIGKILL);
        waitpid(pid_, nullptr, 0);
    }
}

bool ProgramProcess::wait_exit(std::chrono::milliseconds timeout) {
    if (pid_ <= 0) return true;
    auto deadline = std::chrono::steady_clock::now() + timeout;
    while (std::chrono::steady_clock::now() < deadline) {
        int status = 0;
        if (waitpid(pid_, &status, WNOHANG) == pid_) {
            exit_code_ = WIFEXITED(status) ? WEXITSTATUS(status) : 128 + WTERMSIG(status);
            pid_ = -1;
            return true;
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(20));
    }
    return false;
}

void ProgramProcess::terminate() {
    if (pid_ > 0) kill(pid_, SIGTERM);
}

}
