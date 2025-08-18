#include "map.h"
#include "framework/error.h"
#include <cstring>

using namespace doukutsu;

uint8_t Map::get_attribute(size_t x, size_t y) const {
    if (x >= width || y >= height) {
        return 0;
    }

    size_t pos = width * y + x;
    if (pos >= tiles.size()) {
        return attrib[0];
    }

    return attrib[tiles[pos]];
}

Map Map::load_pxm(std::istream& map_data, std::istream& attrib_data) {
    // Read magic
    char magic[3];
    if (!map_data.read(magic, 3) || std::memcmp(magic, "PXM", 3) != 0) {
        throw framework::GameError::io_error("Invalid PXM magic");
    }

    // Read version
    uint8_t version;
    if (!map_data.read(reinterpret_cast<char*>(&version), 1)) {
        throw framework::GameError::io_error("Failed to read PXM version");
    }

    if (version != 0x10) {
        throw framework::GameError::io_error("Unsupported PXM version");
    }

    Map map;

    // Read dimensions (little-endian)
    if (!map_data.read(reinterpret_cast<char*>(&map.width), 2) ||
        !map_data.read(reinterpret_cast<char*>(&map.height), 2)) {
        throw framework::GameError::io_error("Failed to read map dimensions");
    }

    // Resize tiles vector
    size_t tile_count = static_cast<size_t>(map.width) * map.height;
    map.tiles.resize(tile_count);

    // Read tile data
    if (!map_data.read(reinterpret_cast<char*>(map.tiles.data()), tile_count)) {
        throw framework::GameError::io_error("Failed to read tile data");
    }

    // Read attribute data
    if (!attrib_data.read(reinterpret_cast<char*>(map.attrib.data()), 0x100)) {
        // Attribute data might be shorter, that's OK
    }

    map.tile_size = TileSize::Tile16x16;
    return map;
}

std::vector<NPCData> load_npc_data(std::istream& data) {
    // Read magic
    char magic[3];
    if (!data.read(magic, 3) || std::memcmp(magic, "PXE", 3) != 0) {
        throw framework::GameError::io_error("Invalid PXE magic");
    }

    // Read version
    uint8_t version;
    if (!data.read(reinterpret_cast<char*>(&version), 1)) {
        throw framework::GameError::io_error("Failed to read PXE version");
    }

    if (version != 0x00 && version != 0x10) {
        throw framework::GameError::io_error("Unsupported PXE version");
    }

    // Read NPC count
    uint32_t count;
    if (!data.read(reinterpret_cast<char*>(&count), 4)) {
        throw framework::GameError::io_error("Failed to read NPC count");
    }

    std::vector<NPCData> npcs;
    npcs.reserve(count);

    for (uint32_t i = 0; i < count; ++i) {
        NPCData npc;
        npc.id = 170 + i; // Base NPC ID

        if (!data.read(reinterpret_cast<char*>(&npc.x), 2) ||
            !data.read(reinterpret_cast<char*>(&npc.y), 2) ||
            !data.read(reinterpret_cast<char*>(&npc.flag_num), 2) ||
            !data.read(reinterpret_cast<char*>(&npc.event_num), 2) ||
            !data.read(reinterpret_cast<char*>(&npc.npc_type), 2) ||
            !data.read(reinterpret_cast<char*>(&npc.flags), 2)) {
            throw framework::GameError::io_error("Failed to read NPC data");
        }

        // Layer field only in version 0x10
        if (version == 0x10) {
            if (!data.read(reinterpret_cast<char*>(&npc.layer), 1)) {
                throw framework::GameError::io_error("Failed to read NPC layer");
            }
        } else {
            npc.layer = 0;
        }

        npcs.push_back(npc);
    }

    return npcs;
}