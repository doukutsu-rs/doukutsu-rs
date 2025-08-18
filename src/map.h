#pragma once

#include "common.h"
#include <vector>
#include <string>
#include <array>
#include <iostream>

namespace doukutsu {

/// Tile size enumeration
enum class TileSize {
    Tile16x16,
    Tile8x8
};

/// Map data structure - based on Rust game/map.rs
class Map {
public:
    Map() = default;
    ~Map() = default;
    
    // Move semantics
    Map(Map&&) = default;
    Map& operator=(Map&&) = default;

    // Map properties
    uint16_t width = 0;
    uint16_t height = 0;
    std::vector<uint8_t> tiles;
    std::array<uint8_t, 0x100> attrib{};
    TileSize tile_size = TileSize::Tile16x16;

    /// Get tile attribute at position
    [[nodiscard]] uint8_t get_attribute(size_t x, size_t y) const;

    /// Load PXM format map
    static Map load_pxm(std::istream& map_data, std::istream& attrib_data);

    /// Check if position is valid
    [[nodiscard]] bool is_valid_position(size_t x, size_t y) const {
        return x < width && y < height;
    }

private:
    // Disable copy semantics
    Map(const Map&) = delete;
    Map& operator=(const Map&) = delete;
};

/// NPC data structure
struct NPCData {
    uint16_t id;
    int16_t x, y;
    uint16_t flag_num;
    uint16_t event_num;
    uint16_t npc_type;
    uint16_t flags;
    uint8_t layer;
};

/// Load NPCs from PXE format
std::vector<NPCData> load_npc_data(std::istream& data);

} // namespace doukutsu