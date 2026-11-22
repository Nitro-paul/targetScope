// doc/programs/image_processor.md#new-holes
#pragma once

#include <opencv2/core.hpp>
#include <string>
#include <vector>

#include "state.hpp"

namespace image_processor {

struct Blob {
    cv::Point2d paper_mm;
    double diameter_mm = 0;
    double roundness = 0;
    double area_mm2 = 0;
    double major_mm = 0;
    double minor_mm = 0;
    double orientation_rad = 0;
    bool merged = false;
};

struct DetectionResult {
    std::vector<Blob> holes;
    std::vector<std::string> warnings;
};

bool is_occluded(const cv::Mat& gray, const cv::Mat& reference);

std::vector<Blob> find_new_blobs(const cv::Mat& gray, const cv::Mat& reference, double px_per_mm,
                                 double bullet_diameter_mm, const std::vector<cv::Point2d>& known_holes_mm = {});

std::vector<Blob> update_candidates(std::vector<Candidate>& candidates, const std::vector<Blob>& blobs,
                                    double bullet_diameter_mm, int confirm_frames);

std::vector<Blob> split_overlapping(const Blob& blob, double bullet_diameter_mm);

float confidence(const Blob& blob, double bullet_diameter_mm);

DetectionResult detect_in_single_image(const cv::Mat& transformed_bgr, double px_per_mm,
                                       const targetscope::model::TargetType& t, double bullet_diameter_mm);

}
