// doc/testing/overview.md#test-kit
#pragma once

#include <sys/types.h>

#include <filesystem>
#include <string>

namespace targetscope::testkit {

class RedisTestServer {
   public:
    RedisTestServer();
    ~RedisTestServer();
    RedisTestServer(const RedisTestServer&) = delete;
    RedisTestServer& operator=(const RedisTestServer&) = delete;
    std::string url() const;
    void stop();
    void restart();
    int port() const { return port_; }

   private:
    bool spawn();
    int port_{};
    pid_t pid_ = -1;
    std::filesystem::path dir_;
};

}
