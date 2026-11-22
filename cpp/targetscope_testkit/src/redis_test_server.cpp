// doc/testing/overview.md#test-kit
#include "targetscope/testkit/redis_test_server.hpp"

#include <arpa/inet.h>
#include <fcntl.h>
#include <netinet/in.h>
#include <signal.h>
#include <sw/redis++/redis++.h>
#include <sys/socket.h>
#include <sys/wait.h>
#include <unistd.h>

#include <chrono>
#include <stdexcept>
#include <thread>

#include "targetscope/testkit/temp_data_dir.hpp"

namespace targetscope::testkit {

namespace {
int free_port() {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    sockaddr_in addr{};
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    addr.sin_port = 0;
    bind(fd, reinterpret_cast<sockaddr*>(&addr), sizeof addr);
    socklen_t len = sizeof addr;
    getsockname(fd, reinterpret_cast<sockaddr*>(&addr), &len);
    int port = ntohs(addr.sin_port);
    close(fd);
    return port;
}
}

// doc/testing/overview.md#test-kit
RedisTestServer::RedisTestServer() {
    dir_ = make_temp_dir("redis");
    for (int attempt = 0; attempt < 5; ++attempt) {
        port_ = free_port();
        if (spawn()) return;
    }
    throw std::runtime_error("redis-server did not start");
}

RedisTestServer::~RedisTestServer() {
    stop();
    std::error_code ec;
    std::filesystem::remove_all(dir_, ec);
}

std::string RedisTestServer::url() const { return "redis://127.0.0.1:" + std::to_string(port_) + "/"; }

bool RedisTestServer::spawn() {
    pid_ = fork();
    if (pid_ < 0) throw std::runtime_error("fork failed");
    if (pid_ == 0) {
        int null = open("/dev/null", O_RDWR);
        dup2(null, 1);
        dup2(null, 2);
        std::string port = std::to_string(port_);
        execlp("redis-server", "redis-server", "--port", port.c_str(), "--bind", "127.0.0.1", "--appendonly", "yes",
               "--appendfsync", "everysec", "--save", "", "--dir", dir_.c_str(), static_cast<char*>(nullptr));
        _exit(127);
    }
    auto deadline = std::chrono::steady_clock::now() + std::chrono::seconds(10);
    while (std::chrono::steady_clock::now() < deadline) {
        int status = 0;
        if (waitpid(pid_, &status, WNOHANG) == pid_) {
            pid_ = -1;
            return false;
        }
        try {
            sw::redis::ConnectionOptions o;
            o.host = "127.0.0.1";
            o.port = port_;
            o.connect_timeout = std::chrono::milliseconds(200);
            o.socket_timeout = std::chrono::milliseconds(500);
            sw::redis::Redis r(o);
            auto info = r.info("server");
            if (info.find("process_id:" + std::to_string(pid_)) != std::string::npos) return true;
        } catch (const sw::redis::Error&) {
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(50));
    }
    kill(pid_, SIGKILL);
    waitpid(pid_, nullptr, 0);
    pid_ = -1;
    return false;
}

void RedisTestServer::stop() {
    if (pid_ <= 0) return;
    kill(pid_, SIGTERM);
    waitpid(pid_, nullptr, 0);
    pid_ = -1;
}

void RedisTestServer::restart() {
    stop();
    if (!spawn()) throw std::runtime_error("redis-server did not restart");
}

}
