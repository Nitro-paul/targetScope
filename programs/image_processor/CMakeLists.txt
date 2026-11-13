# doc/programs/image_processor.md#technology
find_package(OpenCV REQUIRED COMPONENTS core)
if(OpenCV_VERSION VERSION_LESS 4 OR NOT OpenCV_VERSION VERSION_LESS 6)
    message(FATAL_ERROR "OpenCV 4 or 5 is needed, found ${OpenCV_VERSION}")
endif()
# doc/development.md#what-is-needed
if(OpenCV_VERSION VERSION_LESS 5)
    set(TARGETSCOPE_OPENCV_CALIB calib3d)
else()
    set(TARGETSCOPE_OPENCV_CALIB calib geometry)
endif()
find_package(OpenCV REQUIRED COMPONENTS core imgproc imgcodecs ${TARGETSCOPE_OPENCV_CALIB})

add_library(image_processor_core STATIC
    src/capture.cpp
    src/coordinates.cpp
    src/drawing.cpp
    src/hole_detector.cpp
    src/lens_calibration.cpp
    src/live_pipeline.cpp
    src/paper_finder.cpp
    src/photo_jobs.cpp
    src/processing_thread.cpp
    src/request_handler.cpp
    src/staging.cpp
    src/target_proposal.cpp
)
target_include_directories(image_processor_core PUBLIC src)
target_link_libraries(image_processor_core PUBLIC targetscope_common_cpp ${OpenCV_LIBS})

add_executable(image_processor src/main.cpp)
target_link_libraries(image_processor PRIVATE image_processor_core)
install(TARGETS image_processor DESTINATION lib/targetscope)

if(TARGETSCOPE_BUILD_TESTS)
    add_executable(image_processor_tests
        tests/coordinates_test.cpp
        tests/find_paper_test.cpp
        tests/live_start_test.cpp
        tests/hole_detection_test.cpp
        tests/paper_change_test.cpp
        tests/live_frames_test.cpp
        tests/capture_test.cpp
        tests/photos_test.cpp
        tests/lens_calibration_test.cpp
        tests/errors_test.cpp
    )
    target_link_libraries(image_processor_tests PRIVATE image_processor_core targetscope_testkit_cpp GTest::gtest_main)
    gtest_discover_tests(image_processor_tests)

    # Release build, nightly: ctest --test-dir build -L performance
    add_executable(image_processor_performance_tests tests/performance_test.cpp)
    target_link_libraries(image_processor_performance_tests PRIVATE image_processor_core targetscope_testkit_cpp GTest::gtest_main)
    gtest_discover_tests(image_processor_performance_tests PROPERTIES LABELS performance)

    # Accuracy report over testdata/photos and testdata/videos: ctest --test-dir build -L accuracy
    add_executable(image_processor_accuracy_tests tests/accuracy_test.cpp)
    target_link_libraries(image_processor_accuracy_tests PRIVATE image_processor_core targetscope_testkit_cpp GTest::gtest_main)
    gtest_discover_tests(image_processor_accuracy_tests PROPERTIES LABELS accuracy)
    # Hardware tests (IP-HW) are manual checklists: doc/testing/image_processor.md#hardware-tests-manual
endif()
