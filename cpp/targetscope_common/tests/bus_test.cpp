#include "targetscope/bus.hpp"

#include <gtest/gtest.h>
#include <sw/redis++/redis++.h>

#include <atomic>
#include <random>
#include <thread>

#include "targetscope/testkit/event_probe.hpp"
#include "targetscope/testkit/fake_program.hpp"
#include "targetscope/testkit/redis_test_server.hpp"
#include "targetscope/testkit/test_bus.hpp"

// doc/testing/shared_libraries.md#redis-bus-integration

using namespace targetscope;
using namespace std::chrono_literals;
using nlohmann::json;

namespace {

sw::redis::Redis direct(const testkit::RedisTestServer& s) {
    sw::redis::ConnectionOptions o;
    o.host = "127.0.0.1";
    o.port = s.port();
    return sw::redis::Redis(o);
}

struct Echo {
    std::atomic<int> calls{0};
    std::vector<std::string> types;
    std::mutex m;
    Bus::Handler handler() {
        return [this](const std::string& type, const json& payload) -> json {
            ++calls;
            std::lock_guard l(m);
            types.push_back(type);
            return payload;
        };
    }
};

}

// LIB-BUS-01
TEST(Bus, LIB_BUS_01_RequestAndResponse) {
    testkit::RedisTestServer redis;
    Bus b(redis.url(), "b");
    Echo echo;
    b.serve(echo.handler());
    testkit::TestBus a(redis.url());
    auto r = a.send("b", "ping", json::object());
    EXPECT_EQ(r["program"], "b");
    EXPECT_TRUE(r.contains("program_version"));
    EXPECT_EQ(r["language"], "cpp");
    EXPECT_LE(r["started_at"].get<std::int64_t>(), std::time(nullptr));
}

// LIB-BUS-02
TEST(Bus, LIB_BUS_02_ResponsesAreMatched) {
    testkit::RedisTestServer redis;
    Bus b(redis.url(), "b");
    b.serve([](const std::string&, const json& p) -> json {
        std::this_thread::sleep_for(std::chrono::milliseconds(p["n"].get<int>() % 7));
        return p;
    });
    testkit::TestBus a(redis.url());
    std::vector<std::thread> threads;
    std::atomic<int> ok{0};
    for (int i = 0; i < 100; ++i)
        threads.emplace_back([&, i] {
            auto r = a.send("b", "echo", json{{"n", i}});
            if (r["n"] == i) ++ok;
        });
    for (auto& t : threads) t.join();
    EXPECT_EQ(ok, 100);
}

// LIB-BUS-03
TEST(Bus, LIB_BUS_03_Timeout) {
    testkit::RedisTestServer redis;
    testkit::TestBus a(redis.url());
    auto start = std::chrono::steady_clock::now();
    try {
        a.send("nobody", "x", json::object(), {}, 1s);
        FAIL();
    } catch (const BusError& e) {
        EXPECT_EQ(e.code, ErrorCode::Timeout);
    }
    auto took = std::chrono::steady_clock::now() - start;
    EXPECT_GE(took, 1s);
    EXPECT_LT(took, 1500ms);
}

// LIB-BUS-04
TEST(Bus, LIB_BUS_04_LateResponseIsDropped) {
    testkit::RedisTestServer redis;
    testkit::FakeProgram b(redis.url(), "b");
    b.on("slow", {.payload = json{{"v", 1}}, .delay = 1500ms});
    b.on("fast", {.payload = json{{"v", 2}}});
    testkit::TestBus a(redis.url());
    EXPECT_THROW(a.send("b", "slow", json::object(), {}, 1s), BusError);
    std::this_thread::sleep_for(1000ms);
    EXPECT_EQ(a.send("b", "fast", json::object())["v"], 2);
}

// LIB-BUS-05
TEST(Bus, LIB_BUS_05_ResendWithTheSameId) {
    testkit::RedisTestServer redis;
    Bus b(redis.url(), "b");
    std::mutex m;
    std::vector<std::string> ids;
    b.serve([](const std::string&, const json&) -> json { return json::object(); });
    testkit::TestBus a(redis.url());
    const std::string id = new_uuid();
    a.send("b", "x", json::object(), id);
    a.send("b", "x", json::object(), id);
    auto r = direct(redis);
    std::vector<std::pair<std::string, sw::redis::Optional<std::vector<std::pair<std::string, std::string>>>>> items;
    r.xrange("b:requests", "-", "+", std::back_inserter(items));
    ASSERT_EQ(items.size(), 2u);
    for (auto& i : items) EXPECT_NE((*i.second)[0].second.find(id), std::string::npos);
}

// LIB-BUS-06
TEST(Bus, LIB_BUS_06_RemoteError) {
    testkit::RedisTestServer redis;
    testkit::FakeProgram b(redis.url(), "b");
    b.on("get", {.error_code = "not_found", .error_message = "no such thing"});
    testkit::TestBus a(redis.url());
    try {
        a.send("b", "get", json::object());
        FAIL();
    } catch (const BusError& e) {
        EXPECT_EQ(e.code, ErrorCode::NotFound);
        EXPECT_STREQ(e.what(), "no such thing");
    }
}

// LIB-BUS-07
TEST(Bus, LIB_BUS_07_StreamEntryFormat) {
    testkit::RedisTestServer redis;
    testkit::TestBus a(redis.url());
    EXPECT_THROW(a.send("nobody", "x", json{{"k", "ä"}}, {}, 1s), BusError);
    auto r = direct(redis);
    std::vector<std::pair<std::string, sw::redis::Optional<std::vector<std::pair<std::string, std::string>>>>> items;
    r.xrange("nobody:requests", "-", "+", std::back_inserter(items));
    ASSERT_EQ(items.size(), 1u);
    ASSERT_EQ(items[0].second->size(), 1u);
    EXPECT_EQ((*items[0].second)[0].first, "data");
    auto env = json::parse((*items[0].second)[0].second);
    EXPECT_EQ(env["type"], "x");
    EXPECT_EQ(env["payload"]["k"], "ä");
}

// LIB-BUS-08
TEST(Bus, LIB_BUS_08_UnknownRequestType) {
    testkit::RedisTestServer redis;
    testkit::FakeProgram b(redis.url(), "b");
    testkit::TestBus a(redis.url());
    Bus c(redis.url(), "c");
    c.serve([](const std::string& type, const json&) -> json {
        throw ServiceError(ErrorCode::InvalidMessage, "unknown type " + type);
    });
    try {
        a.send("c", "nonsense", json::object());
        FAIL();
    } catch (const BusError& e) {
        EXPECT_EQ(e.code, ErrorCode::InvalidMessage);
    }
    std::this_thread::sleep_for(200ms);
    auto r = direct(redis);
    auto pending = r.command("XPENDING", "c:requests", "c");
    EXPECT_EQ(pending->element[0]->integer, 0);
}

// LIB-BUS-09
TEST(Bus, LIB_BUS_09_UnparseableEntry) {
    testkit::RedisTestServer redis;
    Echo echo;
    Bus b(redis.url(), "b");
    b.serve(echo.handler());
    testkit::TestBus a(redis.url());
    a.send_raw("b", "garbage");
    std::this_thread::sleep_for(500ms);
    EXPECT_EQ(echo.calls, 0);
    auto r = direct(redis);
    EXPECT_EQ(r.command("XPENDING", "b:requests", "b")->element[0]->integer, 0);
    EXPECT_EQ(r.xlen("b:responses"), 0);
}

// LIB-BUS-10
TEST(Bus, LIB_BUS_10_UnknownVersion) {
    testkit::RedisTestServer redis;
    Echo echo;
    Bus b(redis.url(), "b");
    b.serve(echo.handler());
    testkit::TestBus a(redis.url());
    json env{{"type", "x"},
             {"version", 99},
             {"request_id", new_uuid()},
             {"reply_to", "z:responses"},
             {"payload", json::object()}};
    a.send_raw("b", env.dump());
    std::this_thread::sleep_for(500ms);
    EXPECT_EQ(echo.calls, 0);
    auto r = direct(redis);
    std::vector<std::pair<std::string, sw::redis::Optional<std::vector<std::pair<std::string, std::string>>>>> items;
    r.xrange("z:responses", "-", "+", std::back_inserter(items));
    ASSERT_EQ(items.size(), 1u);
    auto resp = json::parse((*items[0].second)[0].second);
    EXPECT_EQ(resp["error"]["code"], "invalid_message");
    EXPECT_EQ(r.command("XPENDING", "b:requests", "b")->element[0]->integer, 0);
}

// LIB-BUS-11
TEST(Bus, LIB_BUS_11_NoReplyTo) {
    testkit::RedisTestServer redis;
    Echo echo;
    Bus b(redis.url(), "b");
    b.serve(echo.handler());
    testkit::TestBus a(redis.url());
    json env{{"type", "x"}, {"version", 1}, {"request_id", new_uuid()}, {"payload", json::object()}};
    a.send_raw("b", env.dump());
    std::this_thread::sleep_for(500ms);
    EXPECT_EQ(echo.calls, 1);
    auto r = direct(redis);
    EXPECT_EQ(r.command("XPENDING", "b:requests", "b")->element[0]->integer, 0);
}

// LIB-BUS-12
TEST(Bus, LIB_BUS_12_PendingRequestsFirst) {
    testkit::RedisTestServer redis;
    auto r = direct(redis);
    r.command<void>("XGROUP", "CREATE", "b:requests", "b", "0", "MKSTREAM");
    for (int i = 0; i < 2; ++i) {
        json env{{"type", "x"}, {"version", 1}, {"request_id", new_uuid()}, {"payload", json{{"n", i}}}};
        std::vector<std::pair<std::string, std::string>> f{{"data", env.dump()}};
        r.xadd("b:requests", "*", f.begin(), f.end());
    }
    r.command("XREADGROUP", "GROUP", "b", "old-consumer", "COUNT", "1", "STREAMS", "b:requests", ">");
    std::mutex m;
    std::vector<int> order;
    Bus b(redis.url(), "b");
    b.serve([&](const std::string&, const json& p) -> json {
        std::lock_guard l(m);
        order.push_back(p["n"]);
        return json::object();
    });
    std::this_thread::sleep_for(800ms);
    std::lock_guard l(m);
    ASSERT_EQ(order.size(), 2u);
    EXPECT_EQ(order[0], 0);
    EXPECT_EQ(order[1], 1);
}

// LIB-BUS-13
TEST(Bus, LIB_BUS_13_ConsumerGroupAlreadyExists) {
    testkit::RedisTestServer redis;
    {
        Bus b(redis.url(), "b");
        b.serve([](const std::string&, const json&) -> json { return json::object(); });
        testkit::TestBus a(redis.url());
        a.send("b", "ping", json::object());
    }
    Bus b(redis.url(), "b");
    b.serve([](const std::string&, const json&) -> json { return json::object(); });
    testkit::TestBus a(redis.url());
    EXPECT_NO_THROW(a.send("b", "ping", json::object()));
}

// LIB-BUS-14
TEST(Bus, LIB_BUS_14_ConsumerName) {
    testkit::RedisTestServer redis;
    Bus b(redis.url(), "b");
    b.serve([](const std::string&, const json&) -> json { return json::object(); });
    testkit::TestBus a(redis.url());
    a.send("b", "ping", json::object());
    auto r = direct(redis);
    auto reply = r.command("XINFO", "CONSUMERS", "b:requests", "b");
    ASSERT_GE(reply->elements, 1u);
    std::string name;
    auto* c = reply->element[0];
    for (size_t i = 0; i + 1 < c->elements; i += 2)
        if (std::string(c->element[i]->str) == "name") name = c->element[i + 1]->str;
    EXPECT_NE(name.find("-" + std::to_string(getpid())), std::string::npos);
}

// LIB-BUS-15
TEST(Bus, LIB_BUS_15_StreamCaps) {
    testkit::RedisTestServer redis;
    Bus b(redis.url(), "b");
    for (int i = 0; i < 25000; ++i) b.publish("e", json::object());
    auto r = direct(redis);
    EXPECT_LE(r.xlen("b:events"), 11000);
    EXPECT_GE(r.xlen("b:events"), 10000);
}

// LIB-BUS-16
TEST(Bus, LIB_BUS_16_PublishedEvent) {
    testkit::RedisTestServer redis;
    Bus b(redis.url(), "b");
    b.publish("shot_detected", json{{"hole_id", 1}});
    auto r = direct(redis);
    std::vector<std::pair<std::string, sw::redis::Optional<std::vector<std::pair<std::string, std::string>>>>> items;
    r.xrange("b:events", "-", "+", std::back_inserter(items));
    ASSERT_EQ(items.size(), 1u);
    auto e = json::parse((*items[0].second)[0].second);
    EXPECT_EQ(e["source"], "b");
    EXPECT_EQ(e["type"], "shot_detected");
    EXPECT_EQ(e["event_id"].get<std::string>().size(), 36u);
    EXPECT_EQ(e["event_id"].get<std::string>()[14], '4');
    EXPECT_NEAR(e["timestamp_ms"].get<double>(), unix_ms(std::chrono::system_clock::now()), 1000);
    EXPECT_EQ(e["payload"]["hole_id"], 1);
}

// LIB-BUS-17
TEST(Bus, LIB_BUS_17_ConsumerGroupReaderKeepsItsPosition) {
    GTEST_SKIP() << "consumer-group event readers are not used by the C++ programs";
}

// LIB-BUS-18
TEST(Bus, LIB_BUS_18_PlainReaderMissesNothingAtStartup) {
    testkit::RedisTestServer redis;
    Bus b(redis.url(), "b");
    Bus reader(redis.url(), "reader");
    std::string after = reader.newest_event_id("b");
    for (int i = 0; i < 3; ++i) b.publish("e", json{{"n", i}});
    std::atomic<int> got{0};
    std::atomic<bool> cancel{false};
    std::thread t([&] { reader.read_events("b", after, [&](const std::string&, const json&) { ++got; }, &cancel); });
    std::this_thread::sleep_for(500ms);
    cancel = true;
    t.join();
    EXPECT_EQ(got, 3);
}

// LIB-BUS-19
TEST(Bus, LIB_BUS_19_PlainReaderContinuity) {
    testkit::RedisTestServer redis;
    Bus b(redis.url(), "b");
    Bus reader(redis.url(), "reader");
    std::string after = reader.newest_event_id("b");
    std::vector<int> seen;
    std::atomic<bool> cancel{false};
    std::thread t([&] {
        reader.read_events("b", after, [&](const std::string&, const json& p) { seen.push_back(p["n"]); }, &cancel);
    });
    for (int i = 0; i < 1000; ++i) {
        b.publish("e", json{{"n", i}});
        if (i % 100 == 99) std::this_thread::sleep_for(20ms);
    }
    std::this_thread::sleep_for(500ms);
    cancel = true;
    t.join();
    ASSERT_EQ(seen.size(), 1000u);
    for (int i = 0; i < 1000; ++i) ASSERT_EQ(seen[i], i);
}

// LIB-BUS-20
TEST(Bus, LIB_BUS_20_RedisRestart) {
    testkit::RedisTestServer redis;
    Bus b(redis.url(), "b");
    b.serve([](const std::string&, const json& p) -> json { return p; });
    testkit::TestBus a(redis.url());
    EXPECT_NO_THROW(a.send("b", "x", json::object()));
    redis.stop();
    EXPECT_THROW(a.send("b", "x", json::object(), {}, 1s), BusError);
    std::this_thread::sleep_for(1500ms);
    redis.restart();
    bool ok = false;
    for (int i = 0; i < 30 && !ok; ++i) {
        try {
            a.send("b", "x", json::object(), {}, 2s);
            ok = true;
        } catch (const BusError&) {
        }
    }
    EXPECT_TRUE(ok);
    EXPECT_GE(b.reconnects(), 1u);
}

// LIB-BUS-21
TEST(Bus, LIB_BUS_21_PingIsAnsweredByTheServeLoop) {
    testkit::RedisTestServer redis;
    Echo echo;
    Bus b(redis.url(), "b");
    b.serve(echo.handler());
    testkit::TestBus a(redis.url());
    a.send("b", "ping", json::object());
    EXPECT_EQ(echo.calls, 0);
}

// LIB-BUS-22
TEST(Bus, LIB_BUS_22_ServeConcurrentAnswersAtOnce) { GTEST_SKIP() << "serve_concurrent is a Rust-only function"; }

// LIB-BUS-23
TEST(Bus, LIB_BUS_23_ManyPendingRequestsAtStart) { GTEST_SKIP() << "serve_concurrent is a Rust-only function"; }
