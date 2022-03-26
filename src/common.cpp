#include "common.h"

#include <chrono>

using namespace doukutsu_rs;

float common::interpolate_fix9_scale(int old_val, int val, float frame_delta)
{
    if (std::abs(old_val - val) > 0x1800)
    {
        return fix9_scale(val);
    }

    return fix9_scale(old_val) * (1.0f - frame_delta) + fix9_scale(val) * frame_delta;
}

uint64_t common::get_timestamp()
{
    std::chrono::system_clock::time_point now = std::chrono::system_clock::now();
    return std::chrono::duration_cast<std::chrono::seconds>(now.time_since_epoch()).count();
}