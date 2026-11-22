#include <gtest/gtest.h>

#include <filesystem>
#include <fstream>
#include <opencv2/imgcodecs.hpp>

#include "photo_jobs.hpp"
#include "synthetic.hpp"
#include "targetscope/messages.hpp"
#include "targetscope/testkit/temp_data_dir.hpp"

// IP-PHO
// doc/testing/image_processor.md#photos-and-target-type-proposals-integration

using namespace image_processor;
using namespace image_processor::testing;
using nlohmann::json;

namespace {

struct Jobs {
    targetscope::testkit::TempDataDir dirs;
    PhotoJobs jobs{[this] {
        Runtime r;
        r.staging_dir = dirs.staging_dir;
        return r;
    }};

    std::string stage(const cv::Mat& img, const std::string& ext = ".jpg") {
        const std::string name = "upload_" + targetscope::new_uuid() + ext;
        cv::imwrite((dirs.staging_dir / name).string(), img);
        return name;
    }

    json target_type_json(const Scene& s) { return json(s.target); }
};

Scene photo_scene(std::vector<cv::Point2d> holes = {}) {
    Scene s;
    s.holes_mm = std::move(holes);
    s.image = cv::Size(1600, 1200);
    s.pose.yaw_deg = 20;
    s.pose.pitch_deg = 8;
    s.pose.fill = 0.75;
    return s;
}

json corners_json(const Scene& s) { return json(to_corners(project_corners(s))); }

const std::vector<cv::Point2d> TEN{{80, 100}, {120, 105}, {100, 70},  {60, 60}, {140, 140},
                                   {70, 130}, {130, 70},  {100, 130}, {90, 85}, {150, 100}};

}

// IP-PHO-01
TEST(Photos, IP_PHO_01_PaperCorners) {
    Jobs j;
    Scene s = photo_scene();
    auto r = j.jobs.find_paper({{"staged_file", j.stage(render(s))}}).get();
    ASSERT_TRUE(r["corners"].is_object());
    EXPECT_EQ(r["width_px"], 1600);
    EXPECT_EQ(r["height_px"], 1200);
    auto found = to_points(r["corners"].get<targetscope::model::Corners>());
    auto truth = project_corners(s);
    const double limit = 0.01 * std::hypot(1600, 1200);
    for (int i = 0; i < 4; ++i) EXPECT_LT(cv::norm(found[i] - truth[i]), limit);
}

// IP-PHO-02
TEST(Photos, IP_PHO_02_NoPaper) {
    Jobs j;
    Scene s = photo_scene();
    s.paper = false;
    auto r = j.jobs.find_paper({{"staged_file", j.stage(render(s))}}).get();
    EXPECT_TRUE(r["corners"].is_null());
}

// IP-PHO-03
TEST(Photos, IP_PHO_03_Proposal) {
    Jobs j;
    Scene s = photo_scene();
    auto r = j.jobs
                 .prepare_target_photo({{"staged_file", j.stage(render(s))},
                                        {"corners", corners_json(s)},
                                        {"paper_width_mm", 200},
                                        {"paper_height_mm", 200}})
                 .get();
    EXPECT_EQ(r["staged_file"].get<std::string>().rfind("paper_", 0), 0u);
    EXPECT_TRUE(std::filesystem::exists(j.dirs.staging_dir / r["staged_file"].get<std::string>()));
    EXPECT_DOUBLE_EQ(r["px_per_mm"].get<double>(), 6.0);
    auto p = r["proposal"];
    ASSERT_FALSE(p["center_x_mm"].is_null());
    EXPECT_NEAR(p["center_x_mm"].get<double>(), 100, 1.0);
    EXPECT_NEAR(p["center_y_mm"].get<double>(), 100, 1.0);
    EXPECT_NEAR(p["black_radius_mm"].get<double>(), 30, 1.0);
    ASSERT_FALSE(p["inner_white_radius_mm"].is_null());
    EXPECT_NEAR(p["inner_white_radius_mm"].get<double>(), 5, 1.0);
    ASSERT_FALSE(p["outer_radius_mm"].is_null());
    EXPECT_NEAR(p["outer_radius_mm"].get<double>(), 90, 1.0);
}

// IP-PHO-04
TEST(Photos, IP_PHO_04_BlankPaper) {
    Jobs j;
    Scene s = photo_scene();
    s.blank = true;
    auto r = j.jobs
                 .prepare_target_photo({{"staged_file", j.stage(render(s))},
                                        {"corners", corners_json(s)},
                                        {"paper_width_mm", 200},
                                        {"paper_height_mm", 200}})
                 .get();
    for (auto& [k, v] : r["proposal"].items()) EXPECT_TRUE(v.is_null()) << k;
}

// IP-PHO-05
TEST(Photos, IP_PHO_05_ShotsInAPhoto) {
    Jobs j;
    Scene s = photo_scene(TEN);
    auto r = j.jobs
                 .detect_shots_in_photo({{"staged_file", j.stage(render(s))},
                                         {"target_type", j.target_type_json(s)},
                                         {"bullet_diameter_mm", 4.0}})
                 .get();
    ASSERT_EQ(r["shots"].size(), 10u);
    auto t = s.target;
    for (auto& truth : TEN) {
        double best = 1e9;
        for (auto& shot : r["shots"]) {
            auto p = shot_to_paper({shot["angle_turns"].get<double>(), shot["radius_mm"].get<double>()}, t);
            best = std::min(best, std::hypot(p.x - truth.x, p.y - truth.y));
        }
        EXPECT_LT(best, 0.5);
    }
    const std::string name = r["staged_file"];
    EXPECT_EQ(name.rfind("photo_", 0), 0u);
    EXPECT_EQ(cv::imread((j.dirs.staging_dir / name).string()).size(), cv::Size(2400, 2400));
}

// IP-PHO-06
TEST(Photos, IP_PHO_06_GivenCorners) {
    Jobs j;
    Scene s = photo_scene(TEN);
    s.background = 232;
    cv::Mat img = render(s);
    auto automatic = j.jobs.find_paper({{"staged_file", j.stage(img)}}).get();
    EXPECT_TRUE(automatic["corners"].is_null());
    auto r = j.jobs
                 .detect_shots_in_photo({{"staged_file", j.stage(img)},
                                         {"target_type", j.target_type_json(s)},
                                         {"bullet_diameter_mm", 4.0},
                                         {"corners", corners_json(s)}})
                 .get();
    EXPECT_EQ(r["shots"].size(), 10u);
}

// IP-PHO-07
TEST(Photos, IP_PHO_07_NoPaperNoCorners) {
    Jobs j;
    Scene s = photo_scene();
    s.paper = false;
    try {
        j.jobs
            .detect_shots_in_photo({{"staged_file", j.stage(render(s))},
                                    {"target_type", j.target_type_json(s)},
                                    {"bullet_diameter_mm", 4.0}})
            .get();
        FAIL();
    } catch (const targetscope::ServiceError& e) {
        EXPECT_EQ(e.code, targetscope::ErrorCode::ProcessingFailed);
    }
}

// IP-PHO-08
TEST(Photos, IP_PHO_08_UnreadableFile) {
    Jobs j;
    const std::string name = "upload_" + targetscope::new_uuid() + ".jpg";
    std::ofstream(j.dirs.staging_dir / name) << "this is not an image";
    try {
        j.jobs.find_paper({{"staged_file", name}}).get();
        FAIL();
    } catch (const targetscope::ServiceError& e) {
        EXPECT_EQ(e.code, targetscope::ErrorCode::IoError);
    }
}

// IP-PHO-09
TEST(Photos, IP_PHO_09_SizeWarning) {
    Jobs j;
    Scene s = photo_scene(TEN);
    s.extra_spot_mm = 12;
    s.extra_spot_pos = {170, 170};
    auto r = j.jobs
                 .detect_shots_in_photo({{"staged_file", j.stage(render(s))},
                                         {"target_type", j.target_type_json(s)},
                                         {"bullet_diameter_mm", 4.0}})
                 .get();
    EXPECT_FALSE(r["warnings"].empty());
    EXPECT_EQ(r["shots"].size(), 10u);
}

// IP-PHO-10
TEST(Photos, IP_PHO_10_NoLensCorrectionForPhotos) {
    Jobs j;
    Scene s = photo_scene(TEN);
    cv::Mat img = render(s);
    const std::string file = j.stage(img);
    json payload{{"staged_file", file}, {"target_type", j.target_type_json(s)}, {"bullet_diameter_mm", 4.0}};
    auto a = j.jobs.detect_shots_in_photo(payload).get();
    auto b = j.jobs.detect_shots_in_photo(payload).get();
    EXPECT_EQ(a["shots"], b["shots"]);
}

// IP-PHO-11
TEST(Photos, IP_PHO_11_PhotosDuringLiveMode) {
    GTEST_SKIP() << "see the process test IP_PRC_05 and the performance tests";
}
