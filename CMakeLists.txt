# doc/software_description.md#code-organization
# doc/testing/overview.md#running-the-tests
cmake_minimum_required(VERSION 3.24)
project(targetscope VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# doc/development.md#environment
if(DEFINED ENV{TARGETSCOPE_DEPS})
    set(TARGETSCOPE_DEPS_DIR "$ENV{TARGETSCOPE_DEPS}")
elseif(EXISTS "$ENV{HOME}/.local/opt/targetscope-deps")
    set(TARGETSCOPE_DEPS_DIR "$ENV{HOME}/.local/opt/targetscope-deps")
endif()
if(DEFINED TARGETSCOPE_DEPS_DIR)
    message(STATUS "Dependencies in ${TARGETSCOPE_DEPS_DIR}")
    list(APPEND CMAKE_PREFIX_PATH "${TARGETSCOPE_DEPS_DIR}")
    set(CMAKE_BUILD_RPATH "${TARGETSCOPE_DEPS_DIR}/lib")
endif()

option(TARGETSCOPE_BUILD_TESTS "Build the C++ tests" ON)
# doc/testing/overview.md#failpoints
option(TARGETSCOPE_FAILPOINTS "Compile failpoints into the programs" OFF)

if(TARGETSCOPE_BUILD_TESTS)
    enable_testing()
    # doc/testing/overview.md#running-the-tests
    find_package(GTest REQUIRED)
    include(GoogleTest)
endif()

add_subdirectory(cpp/targetscope_common)
if(TARGETSCOPE_BUILD_TESTS)
    add_subdirectory(cpp/targetscope_testkit)
endif()
add_subdirectory(programs/camera_reader)
add_subdirectory(programs/image_processor)
