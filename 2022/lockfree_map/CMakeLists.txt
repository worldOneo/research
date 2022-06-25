cmake_minimum_required(VERSION 3.0.0)
project(research/lockfree_map VERSION 0.1.0)
set(CMAKE_CXX_STANDARD 20)

add_subdirectory(lockfree_map)
add_subdirectory(benchmark)
add_subdirectory(demo)