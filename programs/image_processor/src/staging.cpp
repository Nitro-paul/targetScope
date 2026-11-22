// doc/interfaces.md#staging-folder
#include "staging.hpp"

#include <opencv2/imgcodecs.hpp>

#include "targetscope/messages.hpp"

namespace image_processor {

using targetscope::ErrorCode;
using targetscope::ServiceError;

// doc/interfaces.md#staging-folder
std::string write_staged_jpeg(const std::filesystem::path& staging_dir, const std::string& prefix, const cv::Mat& bgr,
                              int quality) {
    const std::string name = prefix + "_" + targetscope::new_uuid() + ".jpg";
    const auto part = staging_dir / ("." + name + ".part");
    const auto final_path = staging_dir / name;
    std::error_code ec;
    std::filesystem::create_directories(staging_dir, ec);
    std::vector<uchar> bytes;
    if (!cv::imencode(".jpg", bgr, bytes, {cv::IMWRITE_JPEG_QUALITY, quality}))
        throw ServiceError(ErrorCode::ProcessingFailed, "encoding the image failed");
    {
        FILE* f = std::fopen(part.c_str(), "wb");
        if (!f) throw ServiceError(ErrorCode::IoError, "cannot write " + part.string());
        const bool ok = std::fwrite(bytes.data(), 1, bytes.size(), f) == bytes.size();
        std::fclose(f);
        if (!ok) {
            std::filesystem::remove(part, ec);
            throw ServiceError(ErrorCode::IoError, "writing " + part.string() + " failed");
        }
    }
    std::filesystem::rename(part, final_path, ec);
    if (ec) throw ServiceError(ErrorCode::IoError, "renaming " + part.string() + " failed: " + ec.message());
    return name;
}

// doc/interfaces.md#staging-folder
cv::Mat read_staged_image(const std::filesystem::path& staging_dir, const std::string& staged_file) {
    if (staged_file.empty() || staged_file.find('/') != std::string::npos ||
        staged_file.find("..") != std::string::npos)
        throw ServiceError(ErrorCode::ValidationFailed, "staged_file must be a plain file name");
    const auto path = staging_dir / staged_file;
    if (!std::filesystem::exists(path)) throw ServiceError(ErrorCode::NotFound, "no such file " + staged_file);
    cv::Mat img = cv::imread(path.string(), cv::IMREAD_COLOR);
    if (img.empty()) throw ServiceError(ErrorCode::IoError, "cannot read the image " + staged_file);
    return img;
}

}
