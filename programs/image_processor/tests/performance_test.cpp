#include <gtest/gtest.h>

#include <chrono>
#include <opencv2/imgcodecs.hpp>

#include "lens_calibration.hpp"
#include "photo_jobs.hpp"
#include "rig.hpp"

// IP-PRF
// doc/testing/image_processor.md#performance-performance

using namespace image_processor;
using namespace image_processor::testing;

// IP-PRF-01
TEST(Performance, IP_PRF_01_LivePipeline) {
    Rig r;
    Scene s;
    s.holes_mm = {{80, 100}, {120, 105}, {100, 70}, {60, 60}, {140, 140}};
    cv::Mat frame = render(s);
    r.start(s, true);
    r.feed(frame, 3);
    const int n = 100;
    std::chrono::duration<double> worst{0}, total{0};
    for (int i = 0; i < n; ++i) {
        auto t0 = std::chrono::steady_clock::now();
        r.state.newest_raw = frame;
        r.last_out = r.pipeline.process(frame, r.seq++);
        std::chrono::duration<double> d = std::chrono::steady_clock::now() - t0;
        worst = std::max(worst, d);
        total += d;
    }
    const double fps = n / total.count();
    std::cout << "live pipeline: " << fps << " frames/s, slowest frame " << worst.count() * 1000 << " ms\n";
    EXPECT_GE(fps, 15.0);
    EXPECT_LT(total.count() / n, 0.100);
}

// IP-PRF-02
TEST(Performance, IP_PRF_02_Photo) {
    targetscope::testkit::TempDataDir dirs;
    PhotoJobs jobs([&] {
        Runtime rt;
        rt.staging_dir = dirs.staging_dir;
        return rt;
    });
    Scene s;
    s.image = cv::Size(4000, 3000);
    s.pose.fill = 0.8;
    s.holes_mm = {{80, 100}, {120, 105}, {100, 70},  {60, 60}, {140, 140},
                  {70, 130}, {130, 70},  {100, 130}, {90, 85}, {150, 100}};
    cv::imwrite((dirs.staging_dir / "upload_big.jpg").string(), render(s));
    auto t0 = std::chrono::steady_clock::now();
    auto r = jobs.detect_shots_in_photo({{"staged_file", "upload_big.jpg"},
                                         {"target_type", nlohmann::json(s.target)},
                                         {"bullet_diameter_mm", 4.0}})
                 .get();
    std::chrono::duration<double> d = std::chrono::steady_clock::now() - t0;
    std::cout << "12 megapixel photo: " << d.count() << " s\n";
    EXPECT_EQ(r["shots"].size(), 10u);
    EXPECT_LT(d.count(), 10.0);
}

// IP-PRF-03
TEST(Performance, IP_PRF_03_LensCalibration) {
    LensCalibrationSession session;
    session.camera.camera_key = "cam";
    session.camera.width_px = 1280;
    session.camera.height_px = 720;
    std::vector<cv::Point2f> corners;
    for (int r = 0; r < BOARD_ROWS; ++r)
        for (int c = 0; c < BOARD_COLS; ++c) corners.emplace_back(300.f + 60.f * c, 200.f + 60.f * r);
    for (int i = 0; i < 20; ++i) {
        auto view = corners;
        for (auto& p : view) {
            p.x += 8.f * std::sin(i * 0.7f + p.y * 0.01f);
            p.y += 6.f * std::cos(i * 1.3f + p.x * 0.01f) + i * 4.f;
        }
        session.views.push_back(view);
    }
    auto t0 = std::chrono::steady_clock::now();
    finish(session);
    std::chrono::duration<double> d = std::chrono::steady_clock::now() - t0;
    std::cout << "lens calibration with 20 views: " << d.count() << " s\n";
    EXPECT_LT(d.count(), 30.0);
}
