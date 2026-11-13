# doc/shared_libraries.md#c-library-targetscope_common_cpp
include(FetchContent)

# doc/shared_libraries.md#libraries
find_package(nlohmann_json 3.11 QUIET)
if(NOT nlohmann_json_FOUND)
    FetchContent_Declare(nlohmann_json URL https://github.com/nlohmann/json/releases/download/v3.11.3/json.tar.xz)
    FetchContent_MakeAvailable(nlohmann_json)
endif()

find_package(spdlog QUIET)
if(NOT spdlog_FOUND)
    FetchContent_Declare(spdlog GIT_REPOSITORY https://github.com/gabime/spdlog.git GIT_TAG v1.14.1)
    FetchContent_MakeAvailable(spdlog)
endif()


add_library(targetscope_common_cpp STATIC
    src/clock.cpp
    src/config.cpp
    src/failpoint.cpp
    src/geometry.cpp
    src/logging.cpp
    src/messages.cpp
    src/model.cpp
    src/targets.cpp
)
target_include_directories(targetscope_common_cpp PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${PROJECT_SOURCE_DIR}/schemas/frames   # frame_header.h, the contract
)
target_link_libraries(targetscope_common_cpp PUBLIC nlohmann_json::nlohmann_json spdlog::spdlog)
if(TARGETSCOPE_FAILPOINTS)
    target_compile_definitions(targetscope_common_cpp PUBLIC TARGETSCOPE_FAILPOINTS=1)
endif()

if(TARGETSCOPE_BUILD_TESTS)
    find_package(GTest QUIET)
    if(NOT GTest_FOUND)
        FetchContent_Declare(googletest URL https://github.com/google/googletest/archive/refs/tags/v1.15.2.tar.gz)
        FetchContent_MakeAvailable(googletest)
    endif()

    add_executable(targetscope_common_cpp_tests
        tests/envelopes_test.cpp
        tests/geometry_test.cpp
        tests/targets_test.cpp
        tests/config_test.cpp
        tests/clock_test.cpp
    )
    target_link_libraries(targetscope_common_cpp_tests PRIVATE targetscope_common_cpp GTest::gtest_main)
    target_compile_definitions(targetscope_common_cpp_tests PRIVATE
        TARGETSCOPE_SCHEMAS_DIR="${PROJECT_SOURCE_DIR}/schemas"
        TARGETSCOPE_TESTDATA_DIR="${PROJECT_SOURCE_DIR}/testdata")
    include(GoogleTest)
    gtest_discover_tests(targetscope_common_cpp_tests)
endif()
