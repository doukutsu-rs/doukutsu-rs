cmake_minimum_required(VERSION 3.18)

project(doukutsu_rs CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED true)

add_executable(doukutsu_rs 
    src/framework/backend.cpp
    src/framework/backend_sdl2.cpp
    src/framework/context.cpp  
    src/framework/filesystem.cpp
    src/framework/keyboard.cpp
    src/framework/gamepad.cpp
    src/common.cpp 
    src/game.cpp
    src/main.cpp
    src/map.cpp
    src/physics.cpp
    src/texture_set.cpp
    src/rng.cpp)