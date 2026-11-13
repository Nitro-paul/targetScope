// redis-server on a free port with its own temporary folder and AOF on; restartable.
#pragma once

#include <filesystem>
#include <string>

namespace targetscope::testkit {

class RedisTestServer {
public:
    RedisTestServer();   // starts
    ~RedisTestServer();  // stops, removes the folder
    std::string url() const;
    void stop();
    void restart();      // same folder and port
    int port() const { return port_; }
private:
    int port_{};
    std::filesystem::path dir_;
    // TODO: child pid
};

}  // namespace targetscope::testkit
