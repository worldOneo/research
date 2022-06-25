file(GLOB hazptr_src "*.hpp" "*.cpp")
add_library(haz_ptrs ${hazptr_src})

if(MSVC)
    message(WARNING "MSVC Doesnt support double/wide CAS which is required for VBR")
endif()

target_include_directories(haz_ptrs PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
