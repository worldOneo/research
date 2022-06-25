file(GLOB main_src "*.hpp" "*.cpp")
add_executable(lockfree_map_demo ${main_src})
target_link_libraries(lockfree_map_demo lockfree_map haz_ptrs)