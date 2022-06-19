if(MSVC)
  set(CMAKE_CXX_FLAGS "/O2")
else()
  set(CMAKE_CXX_FLAGS "-O3")
endif()

add_executable(reclamation_benchmark "benchmark.cpp")
target_link_libraries(reclamation_benchmark benchmark::benchmark haz_ptrs)
