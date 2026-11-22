// doc/programs/image_processor.md#capture-mode
#include "photo_jobs.hpp"

#include <opencv2/imgproc.hpp>

#include "coordinates.hpp"
#include "hole_detector.hpp"
#include "paper_finder.hpp"
#include "staging.hpp"
#include "target_proposal.hpp"
#include "targetscope/messages.hpp"

namespace image_processor {

using nlohmann::json;
using targetscope::ErrorCode;
using targetscope::ServiceError;
using targetscope::model::Corners;
using targetscope::model::TargetType;

namespace {

constexpr int JPEG_QUALITY = 92;

double positive(const json& p, const char* key) {
    if (!p.contains(key) || !p[key].is_number() || p[key].get<double>() <= 0)
        throw ServiceError(ErrorCode::ValidationFailed, std::string(key) + " must be a positive number");
    return p[key].get<double>();
}

std::string staged_file_of(const json& p) {
    if (!p.contains("staged_file") || !p["staged_file"].is_string())
        throw ServiceError(ErrorCode::ValidationFailed, "staged_file is required");
    return p["staged_file"].get<std::string>();
}

Corners corners_of(const json& j) {
    try {
        return j.get<Corners>();
    } catch (const json::exception& e) {
        throw ServiceError(ErrorCode::ValidationFailed, std::string("corners: ") + e.what());
    }
}

}

PhotoJobs::PhotoJobs(RuntimeProvider runtime, int threads) : runtime_(std::move(runtime)) {
    for (int i = 0; i < threads; ++i) workers_.emplace_back([this] { work(); });
}

PhotoJobs::~PhotoJobs() {
    {
        std::lock_guard lock(mutex_);
        stop_ = true;
    }
    cv_.notify_all();
    for (auto& w : workers_) w.join();
}

void PhotoJobs::work() {
    for (;;) {
        std::packaged_task<json()> task;
        {
            std::unique_lock lock(mutex_);
            cv_.wait(lock, [&] { return stop_ || !queue_.empty(); });
            if (queue_.empty()) return;
            task = std::move(queue_.front());
            queue_.pop_front();
        }
        task();
    }
}

std::future<json> PhotoJobs::submit(std::function<json()> job) {
    std::packaged_task<json()> task(std::move(job));
    auto future = task.get_future();
    {
        std::lock_guard lock(mutex_);
        queue_.push_back(std::move(task));
    }
    cv_.notify_one();
    return future;
}

// doc/interfaces.md#find_paper
std::future<json> PhotoJobs::find_paper(const json& payload) {
    return submit([this, payload]() -> json {
        const auto rt = runtime_();
        cv::Mat img = read_staged_image(rt.staging_dir, staged_file_of(payload));
        auto corners = image_processor::find_paper(img, nullptr);
        return {{"corners", corners ? json(*corners) : json(nullptr)}, {"width_px", img.cols}, {"height_px", img.rows}};
    });
}

// doc/interfaces.md#prepare_target_photo
std::future<json> PhotoJobs::prepare_target_photo(const json& payload) {
    return submit([this, payload]() -> json {
        const auto rt = runtime_();
        const std::string file = staged_file_of(payload);
        const double w_mm = positive(payload, "paper_width_mm"), h_mm = positive(payload, "paper_height_mm");
        if (!payload.contains("corners")) throw ServiceError(ErrorCode::ValidationFailed, "corners are required");
        Corners corners = corners_of(payload["corners"]);
        cv::Mat img = read_staged_image(rt.staging_dir, file);
        const double ppm = rt.live_long_side_px / std::max(w_mm, h_mm);
        cv::Mat H = paper_homography(corners, w_mm, h_mm, ppm);
        cv::Mat out;
        cv::warpPerspective(
            img, out, H, cv::Size(static_cast<int>(std::lround(w_mm * ppm)), static_cast<int>(std::lround(h_mm * ppm))),
            cv::INTER_LINEAR);
        const std::string name = write_staged_jpeg(rt.staging_dir, "paper", out, JPEG_QUALITY);
        TargetProposal p = propose_rings(out, ppm);
        auto opt = [](const std::optional<double>& v) { return v ? json(*v) : json(nullptr); };
        return {{"staged_file", name},
                {"px_per_mm", ppm},
                {"proposal",
                 {{"center_x_mm", opt(p.center_x_mm)},
                  {"center_y_mm", opt(p.center_y_mm)},
                  {"inner_white_radius_mm", opt(p.inner_white_radius_mm)},
                  {"black_radius_mm", opt(p.black_radius_mm)},
                  {"outer_radius_mm", opt(p.outer_radius_mm)}}}};
    });
}

// doc/interfaces.md#detect_shots_in_photo
std::future<json> PhotoJobs::detect_shots_in_photo(const json& payload) {
    return submit([this, payload]() -> json {
        const auto rt = runtime_();
        const std::string file = staged_file_of(payload);
        if (!payload.contains("target_type"))
            throw ServiceError(ErrorCode::ValidationFailed, "target_type is required");
        TargetType t;
        try {
            t = payload["target_type"].get<TargetType>();
        } catch (const json::exception& e) {
            throw ServiceError(ErrorCode::ValidationFailed, std::string("target_type: ") + e.what());
        }
        const double bullet = positive(payload, "bullet_diameter_mm");
        cv::Mat img = read_staged_image(rt.staging_dir, file);
        std::optional<Corners> corners;
        if (payload.contains("corners") && !payload["corners"].is_null())
            corners = corners_of(payload["corners"]);
        else
            corners = image_processor::find_paper(img, &t);
        if (!corners) throw ServiceError(ErrorCode::ProcessingFailed, "no target paper found in the photo");
        const double ppm = rt.capture_long_side_px / std::max(t.paper_width_mm, t.paper_height_mm);
        cv::Mat H = paper_homography(*corners, t.paper_width_mm, t.paper_height_mm, ppm);
        cv::Mat out;
        cv::warpPerspective(img, out, H,
                            cv::Size(static_cast<int>(std::lround(t.paper_width_mm * ppm)),
                                     static_cast<int>(std::lround(t.paper_height_mm * ppm))),
                            cv::INTER_LINEAR);
        const std::string name = write_staged_jpeg(rt.staging_dir, "photo", out, JPEG_QUALITY);
        DetectionResult result = detect_in_single_image(out, ppm, t, bullet);
        json shots = json::array();
        for (auto& b : result.holes) {
            auto shot = paper_to_shot(b.paper_mm, t);
            shots.push_back({{"angle_turns", shot.angle_turns},
                             {"radius_mm", shot.radius_mm},
                             {"confidence", confidence(b, bullet)}});
        }
        return {{"shots", shots}, {"staged_file", name}, {"warnings", result.warnings}};
    });
}

}
