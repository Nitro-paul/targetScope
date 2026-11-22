#include <gtest/gtest.h>

#include <filesystem>
#include <opencv2/imgcodecs.hpp>

#include "rig.hpp"

// IP-PPR
// doc/testing/image_processor.md#paper-change-integration

using namespace image_processor;
using namespace image_processor::testing;

namespace {

Scene with(std::vector<cv::Point2d> holes) {
    Scene s;
    s.holes_mm = std::move(holes);
    return s;
}

void paper_removed(Rig& r, const Scene& s) {
    r.start(s, true);
    r.feed(render(s), 3);
    Scene off = s;
    off.paper = false;
    r.feed(render(off), 65);
}

const std::vector<cv::Point2d> FOUR{{80, 100}, {120, 105}, {100, 70}, {60, 60}};

}

// IP-PPR-01
TEST(PaperChange, IP_PPR_01_RequestedChange) {
    Rig r;
    Scene s = with({});
    r.start(s, true);
    for (auto& h : std::vector<cv::Point2d>{{90, 100}, {110, 100}, {100, 120}}) {
        s.holes_mm.push_back(h);
        r.feed(render(s), 7);
    }
    ASSERT_EQ(r.events_of("shot_detected").size(), 3u);
    auto result = r.pipeline.new_paper(1);
    EXPECT_TRUE(result["changed"]);
    EXPECT_EQ(result["live_paper"], 2);
    ASSERT_TRUE(result["previous_paper_staged_file"].is_string());
    const std::string name = result["previous_paper_staged_file"];
    EXPECT_EQ(name.rfind("target_", 0), 0u);
    cv::Mat img = cv::imread((r.dirs.staging_dir / name).string());
    ASSERT_FALSE(img.empty());
    EXPECT_EQ(std::max(img.cols, img.rows), 2400);
    cv::Mat diff;
    std::vector<cv::Mat> ch;
    cv::split(img, ch);
    cv::absdiff(ch[0], ch[2], diff);
    EXPECT_LT(cv::mean(diff)[0], 3.0);
    for (auto& f : std::filesystem::directory_iterator(r.dirs.staging_dir)) EXPECT_NE(f.path().extension(), ".part");
    EXPECT_TRUE(r.state.live->holes.empty() || !r.state.live->calibration);
    EXPECT_EQ(r.state.live->live_paper, 2);
}

// IP-PPR-02
TEST(PaperChange, IP_PPR_02_RequestAfterAnAutomaticChange) {
    Rig r;
    Scene s = with(FOUR);
    paper_removed(r, s);
    Scene fresh = with({});
    r.feed(render(fresh), 3);
    ASSERT_EQ(r.events_of("paper_replaced").size(), 1u);
    auto result = r.pipeline.new_paper(1);
    EXPECT_FALSE(result["changed"]);
    EXPECT_EQ(result["live_paper"], 2);
    EXPECT_TRUE(result["previous_paper_staged_file"].is_null());
    EXPECT_EQ(r.events_of("paper_replaced").size(), 1u);
}

// IP-PPR-03
TEST(PaperChange, IP_PPR_03_ChangeBeforeCalibration) {
    Rig r;
    r.pipeline.start(tt_ref(), 4.0);
    auto result = r.pipeline.new_paper(1);
    EXPECT_TRUE(result["changed"]);
    EXPECT_TRUE(result["previous_paper_staged_file"].is_null());
    EXPECT_EQ(result["live_paper"], 2);
}

// IP-PPR-04
TEST(PaperChange, IP_PPR_04_AutomaticSwap) {
    Rig r;
    paper_removed(r, with(FOUR));
    EXPECT_EQ(r.pipeline.status().state, "lost");
    r.feed(render(with({})), 3);
    auto replaced = r.events_of("paper_replaced");
    ASSERT_EQ(replaced.size(), 1u);
    EXPECT_EQ(replaced[0]["live_paper"], 2);
    ASSERT_TRUE(replaced[0]["previous_paper_staged_file"].is_string());
    EXPECT_TRUE(
        std::filesystem::exists(r.dirs.staging_dir / replaced[0]["previous_paper_staged_file"].get<std::string>()));
    EXPECT_EQ(r.pipeline.status().state, "calibrated");
}

// IP-PPR-05
TEST(PaperChange, IP_PPR_05_SamePaperBack) {
    Rig r;
    Scene s = with(FOUR);
    paper_removed(r, s);
    r.feed(render(s), 3);
    EXPECT_TRUE(r.events_of("paper_replaced").empty());
    EXPECT_EQ(r.state.live->live_paper, 1);
    EXPECT_EQ(r.pipeline.status().state, "calibrated");
}

// IP-PPR-06
TEST(PaperChange, IP_PPR_06_OldPaperWithoutHoles) {
    Rig r;
    paper_removed(r, with({}));
    r.feed(render(with({})), 3);
    EXPECT_TRUE(r.events_of("paper_replaced").empty());
    EXPECT_EQ(r.state.live->live_paper, 1);
}

// IP-PPR-07
TEST(PaperChange, IP_PPR_07_TwentyPercentLimit) {
    const std::vector<cv::Point2d> five{{80, 100}, {120, 105}, {100, 70}, {60, 60}, {140, 140}};
    {
        Rig r;
        paper_removed(r, with(five));
        r.feed(render(with({{80, 100}})), 3);
        EXPECT_EQ(r.events_of("paper_replaced").size(), 1u);
    }
    {
        Rig r;
        paper_removed(r, with(five));
        r.feed(render(with({{80, 100}, {120, 105}})), 3);
        EXPECT_TRUE(r.events_of("paper_replaced").empty());
    }
}

// IP-PPR-08
TEST(PaperChange, IP_PPR_08_ShotsAroundTheChange) {
    Rig r;
    Scene s = with(FOUR);
    r.start(s, true);
    s.holes_mm.push_back({100, 130});
    r.feed(render(s), 8);
    ASSERT_EQ(r.events_of("shot_detected").size(), 1u);
    Scene off = s;
    off.paper = false;
    r.feed(render(off), 65);
    Scene fresh = with({});
    r.feed(render(fresh), 3);
    ASSERT_EQ(r.events_of("paper_replaced").size(), 1u);
    fresh.holes_mm.push_back({100, 80});
    r.feed(render(fresh), 8);
    auto shots = r.events_of("shot_detected");
    ASSERT_EQ(shots.size(), 2u);
    EXPECT_EQ(shots[0]["live_paper"], 1);
    EXPECT_EQ(shots[1]["live_paper"], 2);
}

// IP-PPR-09
TEST(PaperChange, IP_PPR_09_CountingKept) {
    Rig r;
    r.start(with(FOUR), true);
    EXPECT_TRUE(r.state.live->counting);
    r.pipeline.new_paper(1);
    EXPECT_TRUE(r.state.live->counting);
}

// IP-PPR-10
TEST(PaperChange, IP_PPR_10_HolesOnTheNewPaper) {
    Rig r;
    paper_removed(r, with(FOUR));
    r.feed(render(with({{50, 150}, {150, 50}})), 3);
    ASSERT_EQ(r.events_of("paper_replaced").size(), 1u);
    ASSERT_EQ(r.state.live->holes.size(), 2u);
    for (auto& h : r.state.live->holes) EXPECT_FALSE(h.counted);
    EXPECT_TRUE(r.events_of("shot_detected").empty());
    Scene next = with({{50, 150}, {150, 50}, {100, 130}});
    r.feed(render(next), 8);
    auto shots = r.events_of("shot_detected");
    ASSERT_EQ(shots.size(), 1u);
    EXPECT_EQ(shots[0]["live_paper"], 2);
    int counted = 0;
    for (auto& h : r.state.live->holes) counted += h.counted;
    EXPECT_EQ(counted, 1);
}
