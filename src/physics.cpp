#include "physics.h"
#include <algorithm>

using namespace doukutsu;

namespace doukutsu::physics {

/// Physics collision check offsets - from the Rust physics.rs
const std::array<std::pair<int32_t, int32_t>, 64> OFFSETS = {{
    {0, 0},
    {1, 0}, {0, 1}, {1, 1},
    {2, 0}, {2, 1}, {2, 2}, {0, 2}, {1, 2},
    {-1, -1}, {-1, 0}, {-1, 1}, {-1, 2},
    {0, -1}, {1, -1}, {2, -1},
    {3, 3},
    {3, -1}, {3, 0}, {3, 1}, {3, 2},
    {-1, 3}, {0, 3}, {1, 3}, {2, 3},
    {-2, -2}, {-2, -1}, {-2, 0}, {-2, 1}, {-2, 2}, {-2, 3},
    {-1, -2}, {0, -2}, {1, -2}, {2, -2}, {3, -2},
    {-3, -3}, {-3, -2}, {-3, -1}, {-3, 0}, {-3, 1}, {-3, 2}, {-3, 3},
    {-2, -3}, {-1, -3}, {0, -3}, {1, -3}, {2, -3}, {3, -3},
    {4, -3}, {4, -2}, {4, -1}, {4, 0}, {4, 1}, {4, 2}, {4, 3},
    {-3, 4}, {-2, 4}, {-1, 4}, {0, 4}, {1, 4}, {2, 4}, {3, 4}, {4, 4}
}};

void PhysicalEntity::tick_map_collisions(SharedGameState& state, const Map& map) {
    // Basic collision system stub
    // TODO: Implement full collision system based on Rust implementation
    
    const size_t hit_rect_size = std::clamp(this->hit_rect_size(), 1ul, 4ul);
    const int32_t tile_size = (map.tile_size == TileSize::Tile16x16) ? 0x2000 : 0x1000;
    const int32_t x = (this->x() + this->offset_x()) / tile_size;
    const int32_t y = (this->y() + this->offset_y()) / tile_size;

    // Clear previous collision flags
    this->flags().value = 0;
    
    // Check collision for each offset based on hit rect size
    for (size_t idx = 0; idx < hit_rect_size * hit_rect_size && idx < OFFSETS.size(); ++idx) {
        const auto [ox, oy] = OFFSETS[idx];
        const uint8_t attrib = map.get_attribute(static_cast<size_t>(x + ox), static_cast<size_t>(y + oy));
        
        // Basic collision types (stub implementation)
        switch (attrib) {
            case 0x05: case 0x41: case 0x43: case 0x46:
                // Solid blocks
                if (this->is_player()) {
                    // TODO: Implement block collision
                    this->flags().set_hit_bottom_wall(true);
                }
                break;
            case 0x02: case 0x60:
                // Water tiles
                this->flags().set_in_water(true);
                break;
            case 0x62: case 0x42:
                // Spike tiles
                if (this->is_player()) {
                    this->flags().set_hit_by_spike(true);
                    if (attrib & 0x20) {
                        this->flags().set_in_water(true);
                    }
                }
                break;
            // TODO: Add more collision types as needed
            default:
                break;
        }
    }
}

} // namespace doukutsu::physics