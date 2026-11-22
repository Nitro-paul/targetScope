// doc/programs/image_processor.md#lens-calibration
#include "lens_calibration.hpp"

#include <opencv2/imgproc.hpp>

#include "targetscope/messages.hpp"

#if __has_include(<opencv2/calib.hpp>)
#include <opencv2/calib.hpp>
#endif
#if __has_include(<opencv2/objdetect.hpp>)
#include <opencv2/objdetect.hpp>
#endif
#if __has_include(<opencv2/geometry.hpp>)
#include <opencv2/geometry.hpp>
#endif
#if __has_include(<opencv2/calib3d.hpp>)
#include <opencv2/calib3d.hpp>
#endif

namespace image_processor {

using nlohmann::json;
using targetscope::ErrorCode;
using targetscope::ServiceError;

// doc/programs/image_processor.md#lens-calibration
bool capture_view(LensCalibrationSession& session, const cv::Mat& raw) {
    cv::Mat gray;
    cv::cvtColor(raw, gray, cv::COLOR_BGR2GRAY);
    std::vector<cv::Point2f> corners;
    if (!cv::findChessboardCornersSB(gray, cv::Size(BOARD_COLS, BOARD_ROWS), corners,
                                     cv::CALIB_CB_EXHAUSTIVE | cv::CALIB_CB_ACCURACY))
        return false;
    session.views.push_back(std::move(corners));
    return true;
}

// doc/programs/image_processor.md#lens-calibration
LensCalibration finish(const LensCalibrationSession& session) {
    if (static_cast<int>(session.views.size()) < MIN_VIEWS)
        throw ServiceError(
            ErrorCode::ValidationFailed,
            "at least " + std::to_string(MIN_VIEWS) + " views are needed, got " + std::to_string(session.views.size()));
    std::vector<cv::Point3f> board;
    for (int r = 0; r < BOARD_ROWS; ++r)
        for (int c = 0; c < BOARD_COLS; ++c) board.emplace_back(static_cast<float>(c), static_cast<float>(r), 0.f);
    std::vector<std::vector<cv::Point3f>> objects(session.views.size(), board);
    cv::Size size(session.camera.width_px, session.camera.height_px);
    LensCalibration out;
    out.camera_key = session.camera.camera_key.value_or("");
    out.width_px = size.width;
    out.height_px = size.height;
    std::vector<cv::Mat> rvecs, tvecs;
    cv::Mat K = cv::Mat::eye(3, 3, CV_64F), D = cv::Mat::zeros(5, 1, CV_64F);
    K.at<double>(0, 0) = K.at<double>(1, 1) = size.width;
    K.at<double>(0, 2) = size.width / 2.0;
    K.at<double>(1, 2) = size.height / 2.0;
    out.rms_px = cv::calibrateCamera(objects, session.views, size, K, D, rvecs, tvecs,
                                     cv::CALIB_USE_INTRINSIC_GUESS | cv::CALIB_ZERO_TANGENT_DIST | cv::CALIB_FIX_K3);
    out.camera_matrix = K;
    out.dist_coeffs = D;
    return out;
}

// doc/database.md#known-settings
json to_setting(const LensCalibration& c) {
    json k = json::array(), d = json::array();
    for (int r = 0; r < 3; ++r)
        for (int col = 0; col < 3; ++col) k.push_back(c.camera_matrix.at<double>(r, col));
    for (int i = 0; i < 5; ++i) d.push_back(c.dist_coeffs.at<double>(i));
    return json{{"width_px", c.width_px},
                {"height_px", c.height_px},
                {"camera_matrix", k},
                {"dist_coeffs", d},
                {"rms_px", c.rms_px}};
}

std::optional<LensCalibration> from_setting(const json& all, const CameraInfo& camera) {
    if (!camera.camera_key || !all.is_object() || !all.contains(*camera.camera_key)) return std::nullopt;
    try {
        const json& e = all.at(*camera.camera_key);
        LensCalibration c;
        c.camera_key = *camera.camera_key;
        c.width_px = e.at("width_px").get<int>();
        c.height_px = e.at("height_px").get<int>();
        c.rms_px = e.at("rms_px").get<double>();
        auto k = e.at("camera_matrix").get<std::vector<double>>();
        auto d = e.at("dist_coeffs").get<std::vector<double>>();
        if (k.size() != 9 || d.size() != 5) return std::nullopt;
        c.camera_matrix = cv::Mat(3, 3, CV_64F);
        for (int i = 0; i < 9; ++i) c.camera_matrix.at<double>(i / 3, i % 3) = k[i];
        c.dist_coeffs = cv::Mat(5, 1, CV_64F);
        for (int i = 0; i < 5; ++i) c.dist_coeffs.at<double>(i) = d[i];
        return c;
    } catch (const json::exception&) {
        return std::nullopt;
    }
}

LensCorrector::LensCorrector(const LensCalibration& c, cv::Size size)
    : camera_matrix_(c.camera_matrix.clone()), dist_coeffs_(c.dist_coeffs.clone()) {
    cv::initUndistortRectifyMap(camera_matrix_, dist_coeffs_, cv::Mat(), camera_matrix_, size, CV_16SC2, map1_, map2_);
}

void LensCorrector::correct(const cv::Mat& raw, cv::Mat& out) const {
    cv::remap(raw, out, map1_, map2_, cv::INTER_LINEAR, cv::BORDER_CONSTANT, cv::Scalar());
}

// doc/programs/image_processor.md#start-and-calibration
std::vector<cv::Point2f> LensCorrector::to_corrected(const std::vector<cv::Point2f>& raw_points) const {
    std::vector<cv::Point2f> out;
    cv::undistortPoints(raw_points, out, camera_matrix_, dist_coeffs_, cv::noArray(), camera_matrix_,
                        cv::TermCriteria(cv::TermCriteria::COUNT | cv::TermCriteria::EPS, 40, 1e-10));
    return out;
}

std::vector<cv::Point2f> LensCorrector::to_raw(const std::vector<cv::Point2f>& corrected_points) const {
    const double fx = camera_matrix_.at<double>(0, 0), fy = camera_matrix_.at<double>(1, 1);
    const double cx = camera_matrix_.at<double>(0, 2), cy = camera_matrix_.at<double>(1, 2);
    const double k1 = dist_coeffs_.at<double>(0), k2 = dist_coeffs_.at<double>(1), p1 = dist_coeffs_.at<double>(2),
                 p2 = dist_coeffs_.at<double>(3), k3 = dist_coeffs_.at<double>(4);
    std::vector<cv::Point2f> out;
    for (auto& p : corrected_points) {
        const double x = (p.x - cx) / fx, y = (p.y - cy) / fy;
        const double r2 = x * x + y * y;
        const double radial = 1 + k1 * r2 + k2 * r2 * r2 + k3 * r2 * r2 * r2;
        const double xd = x * radial + 2 * p1 * x * y + p2 * (r2 + 2 * x * x);
        const double yd = y * radial + p1 * (r2 + 2 * y * y) + 2 * p2 * x * y;
        out.emplace_back(static_cast<float>(xd * fx + cx), static_cast<float>(yd * fy + cy));
    }
    return out;
}

// doc/programs/image_processor.md#lens-calibration
cv::Mat LensCorrector::combined_map(const cv::Mat& homography, cv::Size out) const {
    cv::Mat inv = homography.inv();
    const double fx = camera_matrix_.at<double>(0, 0), fy = camera_matrix_.at<double>(1, 1);
    const double cx = camera_matrix_.at<double>(0, 2), cy = camera_matrix_.at<double>(1, 2);
    const double k1 = dist_coeffs_.at<double>(0), k2 = dist_coeffs_.at<double>(1), p1 = dist_coeffs_.at<double>(2),
                 p2 = dist_coeffs_.at<double>(3), k3 = dist_coeffs_.at<double>(4);
    cv::Mat map(out, CV_32FC2);
    const double* h = inv.ptr<double>();
    for (int v = 0; v < out.height; ++v) {
        auto* row = map.ptr<cv::Vec2f>(v);
        for (int u = 0; u < out.width; ++u) {
            const double w = h[6] * u + h[7] * v + h[8];
            const double px = (h[0] * u + h[1] * v + h[2]) / w;
            const double py = (h[3] * u + h[4] * v + h[5]) / w;
            const double x = (px - cx) / fx, y = (py - cy) / fy;
            const double r2 = x * x + y * y;
            const double radial = 1 + k1 * r2 + k2 * r2 * r2 + k3 * r2 * r2 * r2;
            const double xd = x * radial + 2 * p1 * x * y + p2 * (r2 + 2 * x * x);
            const double yd = y * radial + p1 * (r2 + 2 * y * y) + 2 * p2 * x * y;
            row[u] = cv::Vec2f(static_cast<float>(xd * fx + cx), static_cast<float>(yd * fy + cy));
        }
    }
    return map;
}

}
