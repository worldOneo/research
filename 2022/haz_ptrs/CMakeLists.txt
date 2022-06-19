cmake_minimum_required(VERSION 3.0.0)
project(research/reclamation VERSION 0.1.0)
set(CMAKE_CXX_STANDARD 20)

set(BENCHMARK_ENABLE_TESTING NO)

include(FetchContent)

FetchContent_Declare(
    googlebenchmark
    GIT_REPOSITORY https://github.com/google/benchmark.git
    GIT_TAG v1.6.1
)

FetchContent_MakeAvailable(googlebenchmark)
add_subdirectory(haz_ptrs)
add_subdirectory(benchmark)
add_subdirectory(demo)