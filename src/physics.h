#pragma once

#include <array>
#include <tuple>

#include "common.h"

namespace doukutsu_rs::shared_game_state {
    class SharedGameState;
}

namespace doukutsu_rs::physics
{
    extern const std::array<std::pair<int32_t, int32_t>, 64> OFFSETS;

    class PhysicalEntity
    {
    public:
        virtual int32_t x() = 0;
        virtual int32_t y() = 0;
        virtual int32_t vel_x() = 0;
        virtual int32_t vel_y() = 0;

        virtual uintptr_t hit_rect_size() = 0;
        virtual int32_t offset_x() { return 0; }
        virtual int32_t offset_y() { return 0; }

        virtual const common::Rect<uint32_t> &hit_bounds() = 0;
        virtual const common::Rect<uint32_t> &display_bounds() = 0;

        virtual void set_x(int32_t x) = 0;
        virtual void set_y(int32_t y) = 0;
        virtual void set_vel_x(int32_t x) = 0;
        virtual void set_vel_y(int32_t y) = 0;

        virtual common::Condition &cond() = 0;
        virtual common::Flag &flags() = 0;

        virtual common::Direction direction() = 0;
        virtual bool is_player() = 0;
        virtual bool ignore_tile_44() { return true; }
        virtual bool player_left_pressed() { return false; }
        virtual bool player_right_pressed() { return false; }

        virtual void test_block_hit(shared_game_state::SharedGameState &state, int32_t x, int32_t y);
        virtual void test_platform_hit(shared_game_state::SharedGameState &state, int32_t x, int32_t y);

        // upper left slope (bigger half)
        virtual void test_hit_upper_left_slope_high(shared_game_state::SharedGameState &state, int32_t x, int32_t y);

        // upper left slope (smaller half)
        virtual void test_hit_upper_left_slope_low(shared_game_state::SharedGameState &state, int32_t x, int32_t y);

        // upper right slope (smaller half)
        virtual void test_hit_upper_right_slope_low(shared_game_state::SharedGameState &state, int32_t x, int32_t y);

        // upper right slope (bigger half)
        virtual void test_hit_upper_right_slope_high(shared_game_state::SharedGameState &state, int32_t x, int32_t y);

        // lower left half (bigger)
        virtual void test_hit_lower_left_slope_high(shared_game_state::SharedGameState &state, int32_t x, int32_t y);

        // lower left half (smaller)
        virtual void test_hit_lower_left_slope_low(shared_game_state::SharedGameState &state, int32_t x, int32_t y);

        // lower right half (smaller)
        virtual void test_hit_lower_right_slope_low(shared_game_state::SharedGameState &state, int32_t x, int32_t y);

        // lower right half (bigger)
        virtual void test_hit_lower_right_slope_high(shared_game_state::SharedGameState &state, int32_t x, int32_t y);

        // upper left slope
        virtual void test_hit_upper_left_slope(shared_game_state::SharedGameState &state, int32_t x, int32_t y);

        // upper right slope
        virtual void test_hit_upper_right_slope(shared_game_state::SharedGameState &state, int32_t x, int32_t y);

        // lower left slope
        virtual void test_hit_lower_left_slope(shared_game_state::SharedGameState &state, int32_t x, int32_t y);

        // lower right slope
        virtual void test_hit_lower_right_slope(shared_game_state::SharedGameState &state, int32_t x, int32_t y);

        virtual void test_hit_water(shared_game_state::SharedGameState &state, int32_t x, int32_t y);
        virtual void test_hit_spike(shared_game_state::SharedGameState &state, int32_t x, int32_t y, bool water);
        virtual void test_hit_force(shared_game_state::SharedGameState &state, int32_t x, int32_t y, common::Direction direction, bool water);
        // virtual void tick_map_collisions(shared_game_state::SharedGameState& state, common::NPCList& npc_list, common::Stage& stage);
    };
}