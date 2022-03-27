#include "map.h"

using namespace doukutsu_rs::map;

uint8_t Map::get_attribute(size_t x, size_t y) const
{
    if (x >= width || y >= height)
    {
        return 0;
    }

    size_t pos = width * y + x;
    if (pos > tiles.size())
    {
        return attrib[0];
    }

    return attrib[tiles[pos]];
}