# C++ projects of TargetScope: targetscope_common_cpp, its test kit, camera_reader, image_processor.
# Build: cmake -S . -B build && cmake --build build
# Tests: ctest --test-dir build -LE "performance|accuracy"   (see doc/testing/overview.md#running-the-tests)
cmake_minimum_required(VERSION 3.24)
project(targetscope VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

option(TARGETSCOPE_BUILD_TESTS "Build the C++ tests" ON)
# Failpoints (TS_FAILPOINT) are compiled into test builds only, never into release builds.
option(TARGETSCOPE_FAILPOINTS "Compile failpoints into the programs" OFF)

if(TARGETSCOPE_BUILD_TESTS)
    enable_testing()
endif()

add_subdirectory(cpp/targetscope_common)
if(TARGETSCOPE_BUILD_TESTS)
    add_subdirectory(cpp/targetscope_testkit)
endif()
add_subdirectory(programs/camera_reader)
add_subdirectory(programs/image_processor)
