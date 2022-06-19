file(GLOB main_src "*.hpp" "*.cpp")
add_executable(reclamation_demo ${main_src})
target_link_libraries(reclamation_demo haz_ptrs)