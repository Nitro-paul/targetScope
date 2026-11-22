# doc/testing/overview.md#test-kit
add_library(targetscope_testkit_cpp STATIC
    src/redis_test_server.cpp
    src/test_bus.cpp
    src/fake_program.cpp
    src/event_probe.cpp
    src/temp_data_dir.cpp
    src/fake_clock.cpp
    src/frame_feeder.cpp
    src/failpoints.cpp
    src/fixtures.cpp
    src/process.cpp
)
target_include_directories(targetscope_testkit_cpp PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(targetscope_testkit_cpp PUBLIC targetscope_common_cpp)
target_compile_definitions(targetscope_testkit_cpp PUBLIC
    TARGETSCOPE_TESTDATA_DIR="${PROJECT_SOURCE_DIR}/testdata")
