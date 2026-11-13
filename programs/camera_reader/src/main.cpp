// camera_reader: Redis thread (Bus::serve, settings watch, events) + capture thread
// (doc/programs/camera_reader.md#architecture).
#include "capture_loop.hpp"
#include "request_handler.hpp"

int main() {
    // TODO: BootstrapConfig::from_env, logging::init("camera_reader"), Bus, SettingsCache,
    //       start CaptureLoop thread, Bus::serve(RequestHandler), stop on SIGTERM/SIGINT.
    return 0;
}
