// Hole detection: live (difference to the reference) and in a single image
// (doc/programs/image_processor.md#new-holes, #holes-in-a-single-image).
#pragma once

#include <string>
#include <vector>

#include <opencv2/core.hpp>

#include "state.hpp"

namespace image_processor {

struct Blob {
    cv::Point2d paper_mm;
    double diameter_mm = 0;
    double roundness = 0;
    double area_mm2 = 0;
};

struct DetectionResult {
    std::vector<Blob> holes;
    std::vector<std::string> warnings;  // blobs rejected only because of their size
};

// True if more than 5 % of the paper's pixels differ strongly from the reference.
bool is_occluded(const cv::Mat& gray, const cv::Mat& reference);

// absdiff, threshold, opening, connected components; size 0.6×..2× bullet, roundness > 0.6.
std::vector<Blob> find_new_blobs(const cv::Mat& gray, const cv::Mat& reference, double px_per_mm,
                                 double bullet_diameter_mm);

// Matches blobs to candidates (within half a bullet diameter); returns confirmed ones.
std::vector<Blob> update_candidates(std::vector<Candidate>& candidates, const std::vector<Blob>& blobs,
                                    double bullet_diameter_mm, int confirm_frames);

// Overlapping holes: a blob > 1.6× a single hole's area becomes two holes (confidence < 0.5).
std::vector<Blob> split_overlapping(const Blob& blob, double bullet_diameter_mm);

float confidence(const Blob& blob, double bullet_diameter_mm);

// Single image: adaptive threshold inside (lighter) and outside (darker) the black area.
DetectionResult detect_in_single_image(const cv::Mat& transformed_bgr, double px_per_mm,
                                       const targetscope::model::TargetType& t, double bullet_diameter_mm);

}  // namespace image_processor
