if(MSVC)
  set(CMAKE_CXX_FLAGS "/DEBUG /W4")
else()
  set(CMAKE_CXX_FLAGS "-Wall -g -O0")
endif()

file(GLOB main_src "*.hpp" "*.cpp")
add_executable(reclamation_demo ${main_src})
target_link_libraries(reclamation_demo haz_ptrs)