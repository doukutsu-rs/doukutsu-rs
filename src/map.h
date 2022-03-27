#pragma once

#include <array>
#include <vector>
#include <cstdint>

#include "shared_game_state.h"

namespace doukutsu_rs::map
{
    class Map
    {
    public:
        uint16_t width;
        uint16_t height;
        std::vector<uint8_t> tiles;
        std::array<uint8_t, 0x100> attrib;
        shared_game_state::TileSize tile_size;

        uint8_t get_attribute(size_t x, size_t y) const;
    };
}