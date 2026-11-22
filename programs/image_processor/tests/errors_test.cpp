#include <gtest/gtest.h>

// IP-ERR
// doc/testing/image_processor.md#error-handling-integration

// IP-ERR-01
TEST(Errors, IP_ERR_01_FramesStop) { GTEST_SKIP() << "see the process test Process.IP_ERR_01_FramesStopAndComeBack"; }

// IP-ERR-02
TEST(Errors, IP_ERR_02_FramesComeBack) {
    GTEST_SKIP() << "see the process test Process.IP_ERR_01_FramesStopAndComeBack";
}

// IP-ERR-03
TEST(Errors, IP_ERR_03_Shutdown) { GTEST_SKIP() << "see the process test Process.IP_ERR_03_Shutdown"; }
