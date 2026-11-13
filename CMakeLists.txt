# doc/software_description.md#code-organization
# doc/testing/overview.md#running-the-tests
cmake_minimum_required(VERSION 3.24)
project(targetscope VERSION 0.0.2 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# doc/development.md#environment
if(DEFINED ENV{TARGETSCOPE_DEPS})
    list(APPEND CMAKE_PREFIX_PATH "$ENV{TARGETSCOPE_DEPS}")
    set(CMAKE_BUILD_RPATH "$ENV{TARGETSCOPE_DEPS}/lib")
endif()

option(TARGETSCOPE_BUILD_TESTS "Build the C++ tests" ON)
# doc/testing/overview.md#failpoints
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
