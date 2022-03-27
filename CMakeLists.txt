project(doukutsu_rs CXX)

cmake_minimum_required(VERSION 3.18)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED true)

add_executable(doukutsu_rs 
    src/engine_constants/npcs.cpp
    src/npc/ai/balcony.cpp
    src/sound/sound_manager.cpp
    src/common.cpp 
    src/main.cpp
    src/map.cpp
    src/physics.cpp
    src/shared_game_state.cpp
    src/texture_set.cpp)