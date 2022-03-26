#include "physics.h"
#include "common.h"
#include "shared_game_state.h"

using namespace doukutsu_rs;
using namespace doukutsu_rs::common;
using namespace doukutsu_rs::physics;

//      -3 -2 -1  0  1  2  3  4
//    +------------------------
// -3 | 37 44 45 46 47 48 49 50
// -2 | 38 26 32 33 34 35 36 51
// -1 | 39 27 10 14 15 16 18 52
//  0 | 40 28 11  1  2  5 19 53
//  1 | 41 29 12  3  4  6 20 54
//  2 | 42 30 13  8  9  7 21 55
//  3 | 43 31 22 23 24 25 17 56
//  4 | 57 58 59 60 61 62 63 64
const std::pair<int32_t, int32_t> physics::OFFSETS[64] = {
    {0, 0},
    {1, 0},
    {0, 1},
    {1, 1},
    {2, 0},
    {2, 1},
    {2, 2},
    {0, 2},
    {1, 2},
    {-1, -1},
    {-1, 0},
    {-1, 1},
    {-1, 2},
    {0, -1},
    {1, -1},
    {2, -1},
    {3, 3},
    {3, -1},
    {3, 0},
    {3, 1},
    {3, 2},
    {-1, 3},
    {0, 3},
    {1, 3},
    {2, 3},
    {-2, -2},
    {-2, -1},
    {-2, 0},
    {-2, 1},
    {-2, 2},
    {-2, 3},
    {-1, -2},
    {0, -2},
    {1, -2},
    {2, -2},
    {3, -2},
    {-3, -3},
    {-3, -2},
    {-3, -1},
    {-3, 0},
    {-3, 1},
    {-3, 2},
    {-3, 3},
    {-2, -3},
    {-1, -3},
    {0, -3},
    {1, -3},
    {2, -3},
    {3, -3},
    {4, -3},
    {4, -2},
    {4, -1},
    {4, 0},
    {4, 1},
    {4, 2},
    {4, 3},
    {-3, 4},
    {-2, 4},
    {-1, 4},
    {0, 4},
    {1, 4},
    {2, 4},
    {3, 4},
    {4, 4},
};

void PhysicalEntity::test_block_hit(shared_game_state::SharedGameState &state, int32_t x, int32_t y)
{
    int32_t bounds_x = is_player() ? 0x600 : 0x600;
    int32_t bounds_top = is_player() ? 0x800 : 0x600;
    int32_t bounds_bottom = is_player() ? 0x800 : 0x600;
    int32_t half_tile_size = state.tile_size.as_int() * 0x100;

    if ((this->y() - (int32_t)this->hit_bounds().top) < ((y * 2 + 1) * half_tile_size - bounds_top) && (this->y() + (int32_t)this->hit_bounds().bottom) > ((y * 2 - 1) * half_tile_size + bounds_bottom))
    {
        // left wall
        if ((this->x() - (int32_t)this->hit_bounds().right) < (x * 2 + 1) * half_tile_size && (this->x() - (int32_t)this->hit_bounds().right) > (x * 2) * half_tile_size)
        {
            this->set_x(((x * 2 + 1) * half_tile_size) + (int32_t)this->hit_bounds().right);

            if (is_player())
            {
                if (this->vel_x() < -0x180)
                {
                    this->set_vel_x(-0x180);
                }

                if (!this->player_left_pressed() && this->vel_x() < 0)
                {
                    this->set_vel_x(0);
                }
            }

            this->flags().set_hit_left_wall(true);
        }

        // right wall
        if ((this->x() + (int32_t)this->hit_bounds().right) > (x * 2 - 1) * half_tile_size && (this->x() + (int32_t)this->hit_bounds().right) < (x * 2) * half_tile_size)
        {
            this->set_x(((x * 2 - 1) * half_tile_size) - (int32_t)this->hit_bounds().right);

            if (is_player())
            {
                if (this->vel_x() > 0x180)
                {
                    this->set_vel_x(0x180);
                }

                if (!this->player_right_pressed() && this->vel_x() > 0)
                {
                    this->set_vel_x(0);
                }
            }

            this->flags().set_hit_right_wall(true);
        }
    }

    if (((this->x() - (int32_t)this->hit_bounds().right) < (x * 2 + 1) * half_tile_size - bounds_x) && ((this->x() + (int32_t)this->hit_bounds().right) > (x * 2 - 1) * half_tile_size + bounds_x))
    {
        // ceiling
        if ((this->y() - (int32_t)this->hit_bounds().top) < (y * 2 + 1) * half_tile_size && (this->y() - (int32_t)this->hit_bounds().top) > (y * 2) * half_tile_size)
        {
            this->set_y(((y * 2 + 1) * half_tile_size) + (int32_t)this->hit_bounds().top);

            if (is_player())
            {
                if (!this->cond().hidden() && this->vel_y() < -0x200)
                {
                    state.sound_manager.play_sfx(3);
                    state.create_caret(
                        this->x(),
                        this->y() - (int32_t)this->hit_bounds().top,
                        CaretType::LittleParticles,
                        Direction::Left);
                    state.create_caret(
                        this->x(),
                        this->y() - (int32_t)this->hit_bounds().top,
                        CaretType::LittleParticles,
                        Direction::Left);
                }

                if (this->vel_y() < 0)
                {
                    this->set_vel_y(0);
                }
            }
            else
            {
                this->set_vel_y(0);
            }

            this->flags().set_hit_top_wall(true);
        }

        // floor
        if (((this->y() + (int32_t)this->hit_bounds().bottom) > ((y * 2 - 1) * half_tile_size)) &&
            ((this->y() + (int32_t)this->hit_bounds().bottom) < (y * 2) * half_tile_size))
        {
            this->set_y(((y * 2 - 1) * half_tile_size) - (int32_t)this->hit_bounds().bottom);
            if (is_player())
            {
                if (this->vel_y() > 0x400)
                {
                    state.sound_manager.play_sfx(23);
                }

                if (this->vel_y() > 0)
                {
                    this->set_vel_y(0);
                }
            }
            else
            {
                this->set_vel_y(0);
            }

            this->flags().set_hit_bottom_wall(true);
        }
    }
}