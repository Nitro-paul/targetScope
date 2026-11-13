// image_processor: Redis thread (requests, events), processing thread (live mode),
// photo worker pool (capture-mode jobs) (doc/programs/image_processor.md#architecture).
#include "processing_thread.hpp"
#include "request_handler.hpp"

int main() {
    // TODO: BootstrapConfig::from_env, logging::init("image_processor"), Bus, SettingsCache,
    //       camera info from camera_status + camera_reader:events, start ProcessingThread and
    //       PhotoJobs, Bus::serve(RequestHandler), stop on SIGTERM/SIGINT.
    return 0;
}
