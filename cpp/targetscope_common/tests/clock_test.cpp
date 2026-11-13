// doc/testing/shared_libraries.md#clock-unit
#include "targetscope/clock.hpp"

#include <gtest/gtest.h>

using namespace targetscope;

// LIB-CLK-01
TEST(Clock, LIB_CLK_01_FakeTime) { GTEST_SKIP() << "doc/testing/shared_libraries.md#clock-unit LIB-CLK-01"; }

// LIB-CLK-02
TEST(Clock, LIB_CLK_02_RealClock) {
    SystemClock clock;
    auto a = clock.now();
    clock.sleep(std::chrono::milliseconds(100));
    auto d = std::chrono::duration_cast<std::chrono::milliseconds>(clock.now() - a).count();
    EXPECT_NEAR(d, 100, 20);
}
