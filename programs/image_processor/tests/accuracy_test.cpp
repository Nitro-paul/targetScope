#include <gtest/gtest.h>

// IP-ACC
// doc/testing/image_processor.md#accuracy-acceptance-test-set

// IP-ACC-01
TEST(Accuracy, IP_ACC_01_Photos) { GTEST_SKIP() << "no photos in testdata/photos/ yet (see testdata/README.md)"; }

// IP-ACC-02
TEST(Accuracy, IP_ACC_02_LiveVideos) { GTEST_SKIP() << "no videos in testdata/videos/ yet (see testdata/README.md)"; }

// IP-ACC-03
TEST(Accuracy, IP_ACC_03_OcclusionVideos) {
    GTEST_SKIP() << "no videos in testdata/videos/ yet (see testdata/README.md)";
}

// IP-ACC-04
TEST(Accuracy, IP_ACC_04_PaperSwapVideos) {
    GTEST_SKIP() << "no videos in testdata/videos/ yet (see testdata/README.md)";
}

// IP-ACC-05
TEST(Accuracy, IP_ACC_05_TargetTypeProposals) {
    GTEST_SKIP() << "no photos in testdata/photos/ yet (see testdata/README.md)";
}
