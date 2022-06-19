cmake_minimum_required(VERSION 3.0.0)
project(research/reclamation VERSION 0.1.0)
set(CMAKE_CXX_STANDARD 20)

if(MSVC)
    message(WARNING "MSVC Doesnt support double/wide CAS which is required for VBR")
    add_compile_options(/W4)
    if(CONFIG STREQUAL "Debug")
        message("Using debug")
        add_compile_options(/DEBUG)
    else()
        message("Using release")
        add_compile_options(/O2)
    endif()
else()
    add_compile_options(-Wall)
    if(CONFIG STREQUAL "Debug")
        message("Using debug")
        add_compile_options(-O0)
        add_compile_options(-g3)
    else()
        message("Using release")
        add_compile_options(-O3)
        add_compile_options(-DNDEBUG)
    endif()
endif()

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