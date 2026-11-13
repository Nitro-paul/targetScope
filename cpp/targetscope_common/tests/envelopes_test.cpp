// doc/testing/shared_libraries.md#envelopes-unit
#include <gtest/gtest.h>

#include "targetscope/messages.hpp"

using namespace targetscope;

static RequestEnvelope ping(std::optional<std::string> reply_to) {
    return RequestEnvelope{"ping", MESSAGE_VERSION, new_uuid(), std::move(reply_to), json::object()};
}

// LIB-ENV-01
TEST(Envelopes, LIB_ENV_01_RequestEnvelope) {
    auto j = to_json(ping("a:responses"));
    for (auto k : {"type", "version", "request_id", "reply_to", "payload"}) EXPECT_TRUE(j.contains(k)) << k;
    EXPECT_EQ(j.size(), 5u);
    EXPECT_EQ(j["request_id"].get<std::string>()[14], '4');
}

// LIB-ENV-02
TEST(Envelopes, LIB_ENV_02_RequestWithoutReplyTo) { EXPECT_FALSE(to_json(ping(std::nullopt)).contains("reply_to")); }

// LIB-ENV-03
TEST(Envelopes, LIB_ENV_03_ResponseTypeName) {
    auto req = ping("x:responses");
    req.type = "create_session";
    auto r = ok_response(req, json::object());
    EXPECT_EQ(r.type, "create_session_response");
    EXPECT_EQ(r.request_id, req.request_id);
}

// LIB-ENV-04
TEST(Envelopes, LIB_ENV_04_OkResponse) {
    auto j = to_json(ok_response(ping("x:responses"), json{{"a", 1}}));
    EXPECT_EQ(j["status"], "ok");
    EXPECT_TRUE(j["error"].is_null());
    EXPECT_EQ(j["payload"], (json{{"a", 1}}));
}

// LIB-ENV-05
TEST(Envelopes, LIB_ENV_05_ErrorResponse) {
    ServiceError e(ErrorCode::SettingNotSet, "export_dir is not set", json{{"setting", "export_dir"}});
    auto j = to_json(error_response(ping("x:responses"), e));
    EXPECT_EQ(j["status"], "error");
    EXPECT_TRUE(j["payload"].is_null());
    EXPECT_EQ(j["error"]["code"], "setting_not_set");
    EXPECT_EQ(j["error"]["details"], (json{{"setting", "export_dir"}}));
}

// LIB-ENV-06
TEST(Envelopes, LIB_ENV_06_EventEnvelope) {
    messages::ShotDetected s{1, 1, 0.25, 10, 0.9, 7};
    EventEnvelope e{"shot_detected", MESSAGE_VERSION, new_uuid(), "image_processor", 1794393600123, s};
    auto j = to_json(e);
    for (auto k : {"type", "version", "event_id", "source", "timestamp_ms", "payload"}) EXPECT_TRUE(j.contains(k)) << k;
    EXPECT_EQ(j["payload"]["hole_id"], 1);
}

// LIB-ENV-07
TEST(Envelopes, LIB_ENV_07_UnknownRequestType) { GTEST_SKIP() << "doc/testing/shared_libraries.md#envelopes-unit LIB-ENV-07"; }

// LIB-ENV-08
TEST(Envelopes, LIB_ENV_08_UnknownVersion) { GTEST_SKIP() << "doc/testing/shared_libraries.md#envelopes-unit LIB-ENV-08"; }
