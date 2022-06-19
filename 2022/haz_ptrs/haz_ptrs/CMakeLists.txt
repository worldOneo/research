file(GLOB hazptr_src "*.hpp" "*.cpp")
add_library(haz_ptrs ${hazptr_src})

target_include_directories(haz_ptrs PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")