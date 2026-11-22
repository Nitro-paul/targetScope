// doc/programs/image_processor.md#live-mode
#include "live_pipeline.hpp"

#include <opencv2/imgproc.hpp>

#include "capture.hpp"
#include "coordinates.hpp"
#include "drawing.hpp"
#include "hole_detector.hpp"
#include "lens_calibration.hpp"
#include "paper_finder.hpp"
#include "staging.hpp"
#include "targetscope/clock.hpp"
#include "targetscope/messages.hpp"

namespace image_processor {

using namespace std::chrono_literals;
using nlohmann::json;
using targetscope::ErrorCode;
using targetscope::ServiceError;
using targetscope::model::CalibrationStatus;
using targetscope::model::Corners;

namespace {

constexpr int SUPERVISION_PERIOD = 30;
constexpr double MOVE_LIMIT_MM = 2.0;
constexpr double NEW_PAPER_VISIBLE_FRACTION = 0.20;
constexpr int SUPERVISION_FAILURES = 2;
constexpr int TARGET_IMAGE_QUALITY = 92;

std::int64_t now_ms() { return targetscope::unix_ms(std::chrono::system_clock::now()); }

}

LivePipeline::LivePipeline(ProcessorState& state, LiveEvents events) : state_(state), events_(std::move(events)) {}

cv::Size LivePipeline::live_size() const {
    const auto& live = *state_.live;
    const double ppm = px_per_mm();
    return {static_cast<int>(std::lround(live.target_type.paper_width_mm * ppm)),
            static_cast<int>(std::lround(live.target_type.paper_height_mm * ppm))};
}

// doc/programs/image_processor.md#coordinate-systems
double LivePipeline::px_per_mm() const {
    if (!state_.live) return 0;
    const auto& t = state_.live->target_type;
    return state_.runtime.live_long_side_px / std::max(t.paper_width_mm, t.paper_height_mm);
}

const LensCorrector* LivePipeline::active_lens(cv::Size frame) const {
    return state_.corrector && state_.lens_size == frame ? state_.corrector.get() : nullptr;
}

cv::Mat LivePipeline::corrected(const cv::Mat& raw) const {
    if (auto* lens = active_lens(raw.size())) {
        cv::Mat out;
        lens->correct(raw, out);
        return out;
    }
    return raw;
}

// doc/programs/image_processor.md#live-frame
LivePipeline::Warped LivePipeline::warp(const cv::Mat& raw) const {
    const auto& live = *state_.live;
    Warped w;
    if (live.lens_applied && !live.warp_map.empty())
        cv::remap(raw, w.bgr, live.warp_map, cv::noArray(), cv::INTER_LINEAR);
    else
        cv::warpPerspective(raw, w.bgr, live.calibration->homography, live_size(), cv::INTER_LINEAR);
    cv::cvtColor(w.bgr, w.gray, cv::COLOR_BGR2GRAY);
    return w;
}

CalibrationStatus LivePipeline::status() const {
    CalibrationStatus s;
    s.updated_ms = state_.live ? state_.live->updated_ms : now_ms();
    if (!state_.live) {
        s.message = "live mode not active";
        return s;
    }
    const auto& live = *state_.live;
    s.target_type_id = live.target_type.id;
    if (live.frames_gone) {
        s.message = "no camera frames";
        return s;
    }
    if (!live.calibration) {
        s.message = live.message.empty() ? "searching for the paper" : live.message;
        return s;
    }
    const auto& c = *live.calibration;
    s.state = c.state == CalState::Calibrated ? "calibrated" : "lost";
    s.method = c.method == Method::Auto ? "auto" : "manual";
    s.corners = c.corners;
    s.px_per_mm = c.px_per_mm;
    if (c.state == CalState::Lost) s.message = "paper lost";
    return s;
}

void LivePipeline::emit_status() {
    state_.live->updated_ms = now_ms();
    if (events_.publish) events_.publish("calibration_changed", status());
}

void LivePipeline::set_message(const std::string& message) {
    if (state_.live->message == message) return;
    state_.live->message = message;
    emit_status();
}

// doc/programs/image_processor.md#start-and-calibration
CalibrationStatus LivePipeline::start(const targetscope::model::TargetType& t, double bullet_diameter_mm) {
    LiveState live;
    live.target_type = t;
    live.bullet_diameter_mm = bullet_diameter_mm;
    live.updated_ms = now_ms();
    const bool fresh = !state_.newest_raw.empty() && std::chrono::steady_clock::now() - state_.last_frame_time < 2s;
    live.message = fresh ? "searching for the paper" : "no camera frames";
    state_.live = std::move(live);
    return status();
}

void LivePipeline::stop() { state_.live.reset(); }

// doc/programs/image_processor.md#start-and-calibration
void LivePipeline::install_calibration(const std::vector<cv::Point2f>& frame_corners, Method method) {
    auto& live = *state_.live;
    Calibration c;
    c.method = method;
    c.frame_corners = frame_corners;
    const cv::Size frame = state_.newest_raw.empty() ? cv::Size() : state_.newest_raw.size();
    const LensCorrector* lens = active_lens(frame);
    c.corners = to_corners(lens ? lens->to_raw(frame_corners) : frame_corners);
    c.px_per_mm = px_per_mm();
    c.homography =
        paper_homography(frame_corners, live.target_type.paper_width_mm, live.target_type.paper_height_mm, c.px_per_mm);
    c.state = CalState::Calibrated;
    live.lens_applied = lens != nullptr;
    live.warp_map = lens ? lens->combined_map(c.homography, live_size()) : cv::Mat();
    live.calibration = std::move(c);
    live.message.clear();
    live.search_failures = 0;
}

// doc/programs/image_processor.md#start-and-calibration
CalibrationStatus LivePipeline::calibrate(const Corners& corners) {
    if (!state_.live) throw ServiceError(ErrorCode::InvalidState, "live mode is not active");
    auto& live = *state_.live;
    const cv::Size frame = state_.newest_raw.empty() ? cv::Size() : state_.newest_raw.size();
    const LensCorrector* lens = active_lens(frame);
    auto raw_points = to_points(corners);
    install_calibration(lens ? lens->to_corrected(raw_points) : raw_points, Method::Manual);
    live.calibration->corners = corners;
    live.frames_gone = false;
    if (!state_.newest_raw.empty()) {
        Warped w = warp(state_.newest_raw);
        live.holes.clear();
        live.candidates.clear();
        initial_scan(w);
        live.last_good_raw = state_.newest_raw.clone();
        live.last_good_homography = live.calibration->homography.clone();
    }
    emit_status();
    return status();
}

// doc/programs/image_processor.md#initial-hole-scan
void LivePipeline::initial_scan(const Warped& w) {
    auto& live = *state_.live;
    live.reference = w.gray.clone();
    auto result = detect_in_single_image(w.bgr, live.calibration->px_per_mm, live.target_type, live.bullet_diameter_mm);
    for (auto& b : result.holes) {
        Hole h;
        h.id = live.next_hole_id++;
        h.paper_mm = b.paper_mm;
        h.counted = false;
        h.confidence = confidence(b, live.bullet_diameter_mm);
        live.holes.push_back(h);
    }
}

// doc/programs/image_processor.md#start-and-calibration
void LivePipeline::merge_scan(const Warped& w) {
    auto& live = *state_.live;
    live.reference = w.gray.clone();
    auto result = detect_in_single_image(w.bgr, live.calibration->px_per_mm, live.target_type, live.bullet_diameter_mm);
    for (auto& b : result.holes) {
        bool known = false;
        for (auto& h : live.holes)
            if (std::hypot(h.paper_mm.x - b.paper_mm.x, h.paper_mm.y - b.paper_mm.y) < 0.6 * live.bullet_diameter_mm)
                known = true;
        if (known) continue;
        Hole h;
        h.id = live.next_hole_id++;
        h.paper_mm = b.paper_mm;
        h.counted = false;
        h.confidence = confidence(b, live.bullet_diameter_mm);
        live.holes.push_back(h);
    }
    live.candidates.clear();
}

// doc/programs/image_processor.md#paper-change
bool LivePipeline::is_new_paper(const Warped& w) const {
    const auto& live = *state_.live;
    if (live.holes.empty() || live.reference.empty() || live.reference.size() != w.gray.size()) return false;
    const double ppm = live.calibration->px_per_mm;
    const int r = std::max(2, static_cast<int>(0.4 * live.bullet_diameter_mm * ppm));
    int visible = 0;
    for (auto& h : live.holes) {
        const cv::Point p(static_cast<int>(std::lround(h.paper_mm.x * ppm)),
                          static_cast<int>(std::lround(h.paper_mm.y * ppm)));
        const cv::Rect box =
            cv::Rect(p.x - r, p.y - r, 2 * r + 1, 2 * r + 1) & cv::Rect(0, 0, w.gray.cols, w.gray.rows);
        if (box.area() == 0) continue;
        cv::Mat disc = cv::Mat::zeros(box.size(), CV_8U);
        cv::circle(disc, cv::Point(p.x - box.x, p.y - box.y), r, 255, cv::FILLED);
        cv::Mat diff;
        cv::absdiff(w.gray(box), live.reference(box), diff);
        const int differing = cv::countNonZero((diff > 40) & disc);
        if (differing < 0.35 * cv::countNonZero(disc)) ++visible;
    }
    return static_cast<double>(visible) / live.holes.size() <= NEW_PAPER_VISIBLE_FRACTION;
}

// doc/programs/image_processor.md#paper-change
std::string LivePipeline::save_previous_paper() {
    auto& live = *state_.live;
    if (live.last_good_raw.empty() || live.last_good_homography.empty()) return {};
    try {
        cv::Mat img = warp_at(state_, live, state_.runtime.capture_long_side_px, nullptr);
        return write_staged_jpeg(state_.runtime.staging_dir, "target", img, TARGET_IMAGE_QUALITY);
    } catch (const std::exception&) {
        return {};
    }
}

// doc/programs/image_processor.md#paper-change
void LivePipeline::replace_paper(const Warped& w) {
    auto& live = *state_.live;
    const std::string previous = save_previous_paper();
    live.live_paper += 1;
    live.holes.clear();
    live.candidates.clear();
    live.last_good_raw.release();
    live.last_good_homography.release();
    initial_scan(w);
    if (events_.publish)
        events_.publish("paper_replaced",
                        targetscope::messages::PaperReplaced{
                            live.live_paper, previous.empty() ? std::nullopt : std::optional<std::string>(previous)});
}

// doc/programs/image_processor.md#paper-change
json LivePipeline::new_paper(int from_live_paper) {
    if (!state_.live) throw ServiceError(ErrorCode::InvalidState, "live mode is not active");
    auto& live = *state_.live;
    if (from_live_paper != live.live_paper)
        return {{"changed", false}, {"live_paper", live.live_paper}, {"previous_paper_staged_file", nullptr}};
    const std::string previous = save_previous_paper();
    live.live_paper += 1;
    live.holes.clear();
    live.candidates.clear();
    live.last_good_raw.release();
    live.last_good_homography.release();
    live.reference.release();
    if (live.calibration && live.calibration->method == Method::Auto) {
        live.calibration.reset();
        live.warp_map.release();
        live.last_search = {};
        live.message = "searching for the paper";
        emit_status();
    } else if (live.calibration && !state_.newest_raw.empty() && live.calibration->state == CalState::Calibrated) {
        initial_scan(warp(state_.newest_raw));
    }
    return {{"changed", true},
            {"live_paper", live.live_paper},
            {"previous_paper_staged_file", previous.empty() ? json(nullptr) : json(previous)}};
}

void LivePipeline::set_counting(bool counting) {
    if (!state_.live) throw ServiceError(ErrorCode::Unavailable, "live mode is not active");
    auto& live = *state_.live;
    if (counting) {
        const bool fresh = !live.frames_gone;
        if (!live.calibration || live.calibration->state != CalState::Calibrated || !fresh)
            throw ServiceError(ErrorCode::Unavailable, "the target is not calibrated");
    }
    live.counting = counting;
}

// doc/programs/image_processor.md#paper-change
void LivePipeline::uncount_hole(int hole_id) {
    if (!state_.live) return;
    for (auto& h : state_.live->holes)
        if (h.id == hole_id) h.counted = false;
}

// doc/programs/image_processor.md#error-handling
void LivePipeline::frames_stopped() {
    if (!state_.live || state_.live->frames_gone) return;
    state_.live->frames_gone = true;
    emit_status();
}

// doc/programs/image_processor.md#start-and-calibration
void LivePipeline::search(const cv::Mat& raw) {
    auto& live = *state_.live;
    live.last_search = std::chrono::steady_clock::now();
    cv::Mat frame = corrected(raw);
    auto found = find_paper(frame, &live.target_type);
    if (!found) {
        set_message("paper not found");
        return;
    }
    const bool was_lost = live.calibration && live.calibration->state == CalState::Lost;
    install_calibration(to_points(*found), Method::Auto);
    Warped w = warp(raw);
    if (was_lost && is_new_paper(w)) {
        replace_paper(w);
    } else if (was_lost) {
        merge_scan(w);
    } else {
        live.holes.clear();
        live.candidates.clear();
        initial_scan(w);
    }
    live.last_good_raw = raw.clone();
    live.last_good_homography = live.calibration->homography.clone();
    emit_status();
}

// doc/programs/image_processor.md#start-and-calibration
void LivePipeline::supervise(const cv::Mat& raw, const Warped& now) {
    auto& live = *state_.live;
    auto& cal = *live.calibration;
    (void)now;
    cv::Mat frame = corrected(raw);
    auto found = find_paper_near(frame, to_corners(cal.frame_corners), live.target_type);
    if (!found) {
        if (cal.method == Method::Auto && ++live.search_failures >= SUPERVISION_FAILURES) {
            cal.state = CalState::Lost;
            live.search_failures = 0;
            emit_status();
        }
        return;
    }
    live.search_failures = 0;
    auto pts = to_points(*found);
    std::vector<cv::Point2f> mapped;
    cv::perspectiveTransform(pts, mapped, cal.homography);
    const double w_px = live.target_type.paper_width_mm * cal.px_per_mm,
                 h_px = live.target_type.paper_height_mm * cal.px_per_mm;
    const std::vector<cv::Point2f> expected{{0, 0},
                                            {static_cast<float>(w_px), 0},
                                            {static_cast<float>(w_px), static_cast<float>(h_px)},
                                            {0, static_cast<float>(h_px)}};
    double worst = 0;
    for (int i = 0; i < 4; ++i) worst = std::max(worst, cv::norm(mapped[i] - expected[i]) / cal.px_per_mm);
    if (worst <= MOVE_LIMIT_MM) return;
    if (cal.method == Method::Manual) {
        cal.state = CalState::Lost;
        emit_status();
        return;
    }
    install_calibration(pts, Method::Auto);
    Warped w = warp(raw);
    live.reference = w.gray.clone();
    live.candidates.clear();
    live.last_good_raw = raw.clone();
    live.last_good_homography = live.calibration->homography.clone();
    emit_status();
}

// doc/programs/image_processor.md#new-holes
void LivePipeline::detect_new(const Warped& w) {
    auto& live = *state_.live;
    const double ppm = live.calibration->px_per_mm;
    std::vector<cv::Point2d> known;
    for (auto& h : live.holes) known.push_back(h.paper_mm);
    auto blobs = find_new_blobs(w.gray, live.reference, ppm, live.bullet_diameter_mm, known);
    auto confirmed = update_candidates(live.candidates, blobs, live.bullet_diameter_mm, state_.runtime.confirm_frames);
    for (auto& b : confirmed) {
        bool duplicate = false;
        for (auto& h : live.holes)
            if (std::hypot(h.paper_mm.x - b.paper_mm.x, h.paper_mm.y - b.paper_mm.y) < 0.4 * live.bullet_diameter_mm)
                duplicate = true;
        if (duplicate) continue;
        Hole h;
        h.id = live.next_hole_id++;
        h.paper_mm = b.paper_mm;
        h.counted = live.counting;
        h.confidence = confidence(b, live.bullet_diameter_mm);
        live.holes.push_back(h);
        if (live.counting && events_.publish) {
            auto shot = paper_to_shot(h.paper_mm, live.target_type);
            targetscope::messages::ShotDetected e;
            e.hole_id = h.id;
            e.live_paper = live.live_paper;
            e.angle_turns = shot.angle_turns;
            e.radius_mm = shot.radius_mm;
            e.confidence = h.confidence;
            e.frame_seq = static_cast<std::int64_t>(live.frame_seq);
            events_.publish("shot_detected", e);
        }
        cv::Mat mask = cv::Mat::zeros(w.gray.size(), CV_8U);
        cv::circle(mask, cv::Point(static_cast<int>(h.paper_mm.x * ppm), static_cast<int>(h.paper_mm.y * ppm)),
                   static_cast<int>(1.4 * live.bullet_diameter_mm * ppm), 255, cv::FILLED);
        w.gray.copyTo(live.reference, mask);
    }
}

// doc/programs/image_processor.md#live-mode
std::optional<cv::Mat> LivePipeline::process(const cv::Mat& raw, std::uint64_t seq) {
    if (!state_.live) return std::nullopt;
    auto& live = *state_.live;
    live.frame_counter++;
    live.frame_seq = seq;
    const auto now = std::chrono::steady_clock::now();

    if (live.frames_gone) {
        live.frames_gone = false;
        live.last_search = {};
        if (live.calibration && live.calibration->state == CalState::Calibrated) {
            const LensCorrector* lens = active_lens(raw.size());
            if ((lens != nullptr) == live.lens_applied) {
                Warped w = warp(raw);
                merge_scan(w);
                emit_status();
            } else {
                live.calibration->state = CalState::Lost;
                emit_status();
            }
        } else {
            emit_status();
        }
    }

    const bool searching =
        !live.calibration || (live.calibration->state == CalState::Lost && live.calibration->method == Method::Auto);
    if (searching) {
        if (now - live.last_search >= state_.runtime.search_interval) search(raw);
        if (!live.calibration || live.calibration->state != CalState::Calibrated) return std::nullopt;
    }
    if (live.calibration->state == CalState::Lost) return std::nullopt;

    Warped w = warp(raw);
    if (live.reference.empty()) {
        live.holes.clear();
        initial_scan(w);
    }
    if (live.frame_counter % SUPERVISION_PERIOD == 0) {
        supervise(raw, w);
        if (live.calibration->state != CalState::Calibrated) return std::nullopt;
        if (live.reference.empty()) return std::nullopt;
        w = warp(raw);
    }
    if (!is_occluded(w.gray, live.reference)) {
        live.last_good_raw = raw.clone();
        live.last_good_homography = live.calibration->homography.clone();
        detect_new(w);
    }
    cv::Mat out = w.bgr.clone();
    draw_live_frame(out, live, live.calibration->px_per_mm);
    return out;
}

}
