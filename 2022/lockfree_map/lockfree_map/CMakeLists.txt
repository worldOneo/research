file(GLOB lockfree_map_src "*.hpp" "*.cpp")
add_library(lockfree_map ${lockfree_map_src})
target_link_libraries(lockfree_map haz_ptrs)
target_include_directories(lockfree_map PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
