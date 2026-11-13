// doc/testing/overview.md#test-kit
#pragma once

#include <filesystem>
#include <string>

namespace targetscope::testkit {

class RedisTestServer {
   public:
    RedisTestServer();
    ~RedisTestServer();
    std::string url() const;
    void stop();
    void restart();
    int port() const { return port_; }

   private:
    int port_{};
    std::filesystem::path dir_;
};

}
