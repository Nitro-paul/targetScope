# doc/shared_libraries.md#c-library-targetscope_common_cpp
include(FetchContent)
find_package(Threads REQUIRED)

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


# doc/shared_libraries.md#libraries
find_package(redis++ QUIET)
find_package(iceoryx2-cxx QUIET)
if(NOT redis++_FOUND OR NOT iceoryx2-cxx_FOUND)
    message(FATAL_ERROR "redis-plus-plus and the iceoryx2 C++ bindings are needed (found: redis++ ${redis++_FOUND}, iceoryx2-cxx ${iceoryx2-cxx_FOUND}). "
        "Install them with tools/install_deps.sh (into ~/.local/opt/targetscope-deps) or set TARGETSCOPE_DEPS / CMAKE_PREFIX_PATH to their prefix; see doc/development.md.")
endif()

add_library(targetscope_common_cpp STATIC
    src/bus.cpp
    src/clock.cpp
    src/config.cpp
    src/failpoint.cpp
    src/frames.cpp
    src/geometry.cpp
    src/logging.cpp
    src/messages.cpp
    src/model.cpp
    src/program.cpp
    src/settings.cpp
    src/targets.cpp
)
target_include_directories(targetscope_common_cpp PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${PROJECT_SOURCE_DIR}/schemas/frames
)
target_link_libraries(targetscope_common_cpp PUBLIC nlohmann_json::nlohmann_json spdlog::spdlog
    redis++::redis++ iceoryx2-cxx::static-lib-cxx Threads::Threads)
target_compile_definitions(targetscope_common_cpp PUBLIC TARGETSCOPE_VERSION="${PROJECT_VERSION}")
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
        tests/bus_test.cpp
        tests/settings_test.cpp
        tests/frame_header_test.cpp
    )
    target_link_libraries(targetscope_common_cpp_tests PRIVATE targetscope_common_cpp targetscope_testkit_cpp GTest::gtest_main)
    target_compile_definitions(targetscope_common_cpp_tests PRIVATE
        TARGETSCOPE_SCHEMAS_DIR="${PROJECT_SOURCE_DIR}/schemas"
        TARGETSCOPE_TESTDATA_DIR="${PROJECT_SOURCE_DIR}/testdata")
    include(GoogleTest)
    gtest_discover_tests(targetscope_common_cpp_tests)
endif()
