# camera_reader (C++): reads frames from a USB or network camera and publishes them
# on targetscope/frames/raw; camera controls (doc/programs/camera_reader.md).
find_package(OpenCV 4 REQUIRED COMPONENTS core imgproc videoio)

# Everything except main(), so the tests can link it.
add_library(camera_reader_core STATIC
    src/camera_identity.cpp
    src/camera_controls.cpp
    src/capture_loop.cpp
    src/request_handler.cpp
    src/rtsp_source.cpp
    src/v4l2_devices.cpp
    src/v4l2_source.cpp
)
target_include_directories(camera_reader_core PUBLIC src)
target_link_libraries(camera_reader_core PUBLIC targetscope_common_cpp ${OpenCV_LIBS})

add_executable(camera_reader src/main.cpp)
target_link_libraries(camera_reader PRIVATE camera_reader_core)
install(TARGETS camera_reader DESTINATION lib/targetscope)

if(TARGETSCOPE_BUILD_TESTS)
    add_executable(camera_reader_tests
        tests/source_selection_test.cpp
        tests/capture_loop_test.cpp
        tests/reconnect_test.cpp
        tests/setting_changes_test.cpp
        tests/controls_test.cpp
        tests/devices_test.cpp
        tests/errors_test.cpp
    )
    target_include_directories(camera_reader_tests PRIVATE tests)
    target_link_libraries(camera_reader_tests PRIVATE camera_reader_core targetscope_testkit_cpp GTest::gtest_main)
    gtest_discover_tests(camera_reader_tests)
    # Hardware tests (CR-HW) are manual checklists: doc/testing/camera_reader.md#hardware-tests-manual
endif()
