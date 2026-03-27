cmake_minimum_required(VERSION 3.10)
project(TOS_Traditional VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Source files
file(GLOB_RECURSE SOURCES "src/*.cpp")

# Executable
add_executable(tos_traditional ${SOURCES})

# Include directories
target_include_directories(tos_traditional PRIVATE src)

# Optional: Add compiler flags for warnings (good practice)
if(MSVC)
    target_compile_options(tos_traditional PRIVATE /W4)
else()
    target_compile_options(tos_traditional PRIVATE -Wall -Wextra -Wpedantic)
endif()
