#include "physics.h"
#include "common.h"
#include "caret.h"
#include "npc.h"
#include "stage.h"
#include "shared_game_state.h"

#include <algorithm>

using namespace doukutsu_rs;
using namespace doukutsu_rs::caret;
using namespace doukutsu_rs::common;
using namespace doukutsu_rs::physics;
using namespace doukutsu_rs::npc;
using namespace doukutsu_rs::shared_game_state;
using namespace doukutsu_rs::stage;

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
const std::array<std::pair<int32_t, int32_t>, 64> physics::OFFSETS = {
    std::make_pair(0, 0),
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

void PhysicalEntity::test_platform_hit(SharedGameState &state, int32_t x, int32_t y)
{
    auto half_tile_size = state.tile_size.as_int() * 0x100;

    if (((this->x() - this->hit_bounds().right) < (x * 2 + 1) * half_tile_size) &&
        ((this->x() + this->hit_bounds().right) > (x * 2 - 1) * half_tile_size) &&
        ((this->y() + this->hit_bounds().bottom) > ((y * 2 - 1) * half_tile_size)) &&
        ((this->y() + this->hit_bounds().bottom) < (y * 2 - 1) * half_tile_size + 0x400))
    {
        this->set_y(((y * 2 - 1) * half_tile_size) - this->hit_bounds().bottom);

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

void PhysicalEntity::test_hit_upper_left_slope_high(SharedGameState &state, int32_t x, int32_t y)
{
    auto tile_size = state.tile_size.as_int() * 0x200;
    auto half_tile_size = tile_size / 2;
    auto quarter_tile_size = half_tile_size / 2;

    if ((this->x() < (x * 2 + 1) * half_tile_size) &&
        (this->x() > (x * 2 - 1) * half_tile_size) &&
        ((this->y() - this->hit_bounds().top) < (y * tile_size) - ((this->x() - x * tile_size) / 2) + quarter_tile_size) &&
        ((this->y() + this->hit_bounds().bottom) > (y * 2 - 1) * half_tile_size))
    {
        this->set_y(
            (y * tile_size) - ((this->x() - x * tile_size) / 2) + quarter_tile_size + this->hit_bounds().top);

        if (is_player() && !this->cond().hidden() && this->vel_y() < -0x200)
        {
            state.sound_manager.play_sfx(3);
            state.create_caret(
                this->x(),
                this->y() - this->hit_bounds().top,
                CaretType::LittleParticles,
                Direction::Left);
            state.create_caret(
                this->x(),
                this->y() - this->hit_bounds().top,
                CaretType::LittleParticles,
                Direction::Left);
        }

        if (this->vel_y() < 0)
        {
            this->set_vel_y(0);
        }

        this->flags().set_hit_top_wall(true);
        this->flags().set_hit_upper_left_slope(true);
    }
}

void PhysicalEntity::test_hit_upper_left_slope_low(SharedGameState &state, int32_t x, int32_t y)
{
    auto tile_size = state.tile_size.as_int() * 0x200;
    auto half_tile_size = tile_size / 2;
    auto quarter_tile_size = half_tile_size / 2;

    if ((this->x() < (x * 2 + 1) * half_tile_size) &&
        (this->x() > (x * 2 - 1) * half_tile_size) &&
        ((this->y() - this->hit_bounds().top) < (y * tile_size) - ((this->x() - x * tile_size) / 2) - quarter_tile_size) &&
        ((this->y() + this->hit_bounds().bottom) > (y * 2 - 1) * half_tile_size))
    {
        this->set_y(
            (y * tile_size) - ((this->x() - x * tile_size) / 2) - quarter_tile_size + this->hit_bounds().top);

        if (is_player() && !this->cond().hidden() && this->vel_y() < -0x200)
        {
            state.sound_manager.play_sfx(3);
            state.create_caret(
                this->x(),
                this->y() - this->hit_bounds().top,
                CaretType::LittleParticles,
                Direction::Left);
            state.create_caret(
                this->x(),
                this->y() - this->hit_bounds().top,
                CaretType::LittleParticles,
                Direction::Left);
        }

        if (this->vel_y() < 0)
        {
            this->set_vel_y(0);
        }

        this->flags().set_hit_top_wall(true);
        this->flags().set_hit_upper_left_slope(true);
    }
}

void PhysicalEntity::test_hit_upper_right_slope_low(SharedGameState &state, int32_t x, int32_t y)
{
    auto tile_size = state.tile_size.as_int() * 0x200;
    auto half_tile_size = tile_size / 2;
    auto quarter_tile_size = half_tile_size / 2;

    if ((this->x() < (x * 2 + 1) * half_tile_size) &&
        (this->x() > (x * 2 - 1) * half_tile_size) &&
        ((this->y() - this->hit_bounds().top) < (y * tile_size) + ((this->x() - x * tile_size) / 2) - quarter_tile_size) &&
        ((this->y() + this->hit_bounds().bottom) > (y * 2 - 1) * half_tile_size))
    {
        this->set_y(
            (y * tile_size) + ((this->x() - x * tile_size) / 2) - quarter_tile_size + this->hit_bounds().top);

        if (is_player() && !this->cond().hidden() && this->vel_y() < -0x200)
        {
            state.sound_manager.play_sfx(3);
            state.create_caret(
                this->x(),
                this->y() - this->hit_bounds().top,
                CaretType::LittleParticles,
                Direction::Left);
            state.create_caret(
                this->x(),
                this->y() - this->hit_bounds().top,
                CaretType::LittleParticles,
                Direction::Left);
        }

        if (this->vel_y() < 0)
        {
            this->set_vel_y(0);
        }

        this->flags().set_hit_top_wall(true);
        this->flags().set_hit_upper_right_slope(true);
    }
}

void PhysicalEntity::test_hit_upper_right_slope_high(SharedGameState &state, int32_t x, int32_t y)
{
    auto tile_size = state.tile_size.as_int() * 0x200;
    auto half_tile_size = tile_size / 2;
    auto quarter_tile_size = half_tile_size / 2;

    if ((this->x() < (x * 2 + 1) * half_tile_size) &&
        (this->x() > (x * 2 - 1) * half_tile_size) &&
        ((this->y() - this->hit_bounds().top) < (y * tile_size) + ((this->x() - x * tile_size) / 2) + quarter_tile_size) &&
        ((this->y() + this->hit_bounds().bottom) > (y * 2 - 1) * half_tile_size))
    {
        this->set_y(
            (y * tile_size) + ((this->x() - x * tile_size) / 2) + quarter_tile_size + this->hit_bounds().top);

        if (is_player() && !this->cond().hidden() && this->vel_y() < -0x200)
        {
            state.sound_manager.play_sfx(3);
            state.create_caret(
                this->x(),
                this->y() - this->hit_bounds().top,
                CaretType::LittleParticles,
                Direction::Left);
            state.create_caret(
                this->x(),
                this->y() - this->hit_bounds().top,
                CaretType::LittleParticles,
                Direction::Left);
        }

        if (this->vel_y() < 0)
        {
            this->set_vel_y(0);
        }

        this->flags().set_hit_top_wall(true);
        this->flags().set_hit_upper_right_slope(true);
    }
}

void PhysicalEntity::test_hit_lower_left_slope_high(SharedGameState &state, int32_t x, int32_t y)
{
    auto tile_size = state.tile_size.as_int() * 0x200;
    auto half_tile_size = tile_size / 2;
    auto quarter_tile_size = half_tile_size / 2;

    this->flags().set_hit_left_higher_half(true);

    if ((this->x() < (x * 2 + 1) * half_tile_size) &&
        (this->x() > (x * 2 - 1) * half_tile_size) &&
        ((this->y() + this->hit_bounds().bottom) > (y * tile_size) + ((this->x() - x * tile_size) / 2) - quarter_tile_size) &&
        ((this->y() - this->hit_bounds().top) < (y * 2 + 1) * half_tile_size))
    {
        this->set_y(
            (y * tile_size) + ((this->x() - x * tile_size) / 2) - quarter_tile_size - this->hit_bounds().bottom);

        if (is_player() && this->vel_y() > 0x400)
        {
            state.sound_manager.play_sfx(23);
        }

        if (this->vel_y() > 0)
        {
            this->set_vel_y(0);
        }

        this->flags().set_hit_left_slope(true);
        this->flags().set_hit_bottom_wall(true);
    }
}

void PhysicalEntity::test_hit_lower_left_slope_low(SharedGameState &state, int32_t x, int32_t y)
{
    auto tile_size = state.tile_size.as_int() * 0x200;
    auto half_tile_size = tile_size / 2;
    auto quarter_tile_size = half_tile_size / 2;

    this->flags().set_hit_left_lower_half(true);

    if ((this->x() < (x * 2 + 1) * half_tile_size) &&
        (this->x() > (x * 2 - 1) * half_tile_size) &&
        ((this->y() + this->hit_bounds().bottom) > (y * tile_size) + ((this->x() - x * tile_size) / 2) - quarter_tile_size) &&
        ((this->y() - this->hit_bounds().top) < (y * 2 + 1) * half_tile_size))
    {
        this->set_y(
            (y * tile_size) + ((this->x() - x * tile_size) / 2) - quarter_tile_size - this->hit_bounds().bottom);

        if (is_player() && this->vel_y() > 0x400)
        {
            state.sound_manager.play_sfx(23);
        }

        if (this->vel_y() > 0)
        {
            this->set_vel_y(0);
        }

        this->flags().set_hit_left_slope(true);
        this->flags().set_hit_bottom_wall(true);
    }
}

// // lower right half (smaller)
// fn test_hit_lower_right_slope_low(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
//     let tile_size = state.tile_size.as_int() * 0x200;
//     let half_tile_size = tile_size / 2;
//     let quarter_tile_size = half_tile_size / 2;

//     self.flags().set_hit_right_lower_half(true);

//     if (self.x() < (x * 2 + 1) * half_tile_size)
//         && (self.x() > (x * 2 - 1) * half_tile_size)
//         && (self.y() + self.hit_bounds().bottom as i32)
//             > (y * tile_size) - (self.x() - x * tile_size) / 2 + quarter_tile_size
//         && (self.y() - self.hit_bounds().top as i32) < (y * 2 + 1) * half_tile_size
//     {
//         self.set_y(
//             (y * tile_size) - ((self.x() - x * tile_size) / 2) + quarter_tile_size
//                 - self.hit_bounds().bottom as i32,
//         );

//         if self.is_player() && self.vel_y() > 0x400 {
//             state.sound_manager.play_sfx(23);
//         }

//         if self.vel_y() > 0 {
//             self.set_vel_y(0);
//         }

//         self.flags().set_hit_right_slope(true);
//         self.flags().set_hit_bottom_wall(true);
//     }
// }
void PhysicalEntity::test_hit_lower_right_slope_low(SharedGameState &state, int32_t x, int32_t y)
{
    auto tile_size = state.tile_size.as_int() * 0x200;
    auto half_tile_size = tile_size / 2;
    auto quarter_tile_size = half_tile_size / 2;

    this->flags().set_hit_right_lower_half(true);

    if ((this->x() < (x * 2 + 1) * half_tile_size) &&
        (this->x() > (x * 2 - 1) * half_tile_size) &&
        ((this->y() + this->hit_bounds().bottom) > (y * tile_size) - (this->x() - x * tile_size) / 2 + quarter_tile_size) &&
        ((this->y() - this->hit_bounds().top) < (y * 2 + 1) * half_tile_size))
    {
        this->set_y(
            (y * tile_size) - ((this->x() - x * tile_size) / 2) + quarter_tile_size - this->hit_bounds().bottom);

        if (is_player() && this->vel_y() > 0x400)
        {
            state.sound_manager.play_sfx(23);
        }

        if (this->vel_y() > 0)
        {
            this->set_vel_y(0);
        }

        this->flags().set_hit_right_slope(true);
        this->flags().set_hit_bottom_wall(true);
    }
}

// // lower right half (bigger)
// fn test_hit_lower_right_slope_high(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
//     let tile_size = state.tile_size.as_int() * 0x200;
//     let half_tile_size = tile_size / 2;
//     let quarter_tile_size = half_tile_size / 2;

//     self.flags().set_hit_right_higher_half(true);

//     if (self.x() < (x * 2 + 1) * half_tile_size)
//         && (self.x() > (x * 2 - 1) * half_tile_size)
//         && (self.y() + self.hit_bounds().bottom as i32)
//             > (y * tile_size) - (self.x() - x * tile_size) / 2 - quarter_tile_size
//         && (self.y() - self.hit_bounds().top as i32) < (y * 2 + 1) * half_tile_size
//     {
//         self.set_y(
//             (y * tile_size)
//                 - ((self.x() - x * tile_size) / 2)
//                 - quarter_tile_size
//                 - self.hit_bounds().bottom as i32,
//         );

//         if self.is_player() && self.vel_y() > 0x400 {
//             state.sound_manager.play_sfx(23);
//         }

//         if self.vel_y() > 0 {
//             self.set_vel_y(0);
//         }

//         self.flags().set_hit_right_slope(true);
//         self.flags().set_hit_bottom_wall(true);
//     }
// }

void PhysicalEntity::test_hit_lower_right_slope_high(SharedGameState &state, int32_t x, int32_t y)
{
    auto tile_size = state.tile_size.as_int() * 0x200;
    auto half_tile_size = tile_size / 2;
    auto quarter_tile_size = half_tile_size / 2;

    this->flags().set_hit_right_higher_half(true);

    if ((this->x() < (x * 2 + 1) * half_tile_size) &&
        (this->x() > (x * 2 - 1) * half_tile_size) &&
        ((this->y() + this->hit_bounds().bottom) > (y * tile_size) + ((this->x() - x * tile_size) / 2) - quarter_tile_size) &&
        ((this->y() - this->hit_bounds().top) < (y * 2 + 1) * half_tile_size))
    {
        this->set_y(
            (y * tile_size) + ((this->x() - x * tile_size) / 2) - quarter_tile_size - this->hit_bounds().bottom);

        if (is_player() && this->vel_y() > 0x400)
        {
            state.sound_manager.play_sfx(23);
        }

        if (this->vel_y() > 0)
        {
            this->set_vel_y(0);
        }

        this->flags().set_hit_right_slope(true);
        this->flags().set_hit_bottom_wall(true);
    }
}

// // upper left slope
// fn test_hit_upper_left_slope(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
//     let tile_size = state.tile_size.as_int() * 0x200;
//     let half_tile_size = tile_size / 2;

//     if self.x() < (x * 2 + 1) * half_tile_size
//         && self.x() > (x * 2 - 1) * half_tile_size
//         && (self.y() - self.hit_bounds().top as i32) < (y * tile_size) - (self.x() - x * tile_size)
//         && (self.y() + self.hit_bounds().bottom as i32) > (y * 2 - 1) * half_tile_size
//     {
//         self.set_y((y * tile_size) - (self.x() - x * tile_size) + self.hit_bounds().top as i32);

//         if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
//             state.sound_manager.play_sfx(3);
//             state.create_caret(
//                 self.x(),
//                 self.y() - self.hit_bounds().top as i32,
//                 CaretType::LittleParticles,
//                 Direction::Left,
//             );
//             state.create_caret(
//                 self.x(),
//                 self.y() - self.hit_bounds().top as i32,
//                 CaretType::LittleParticles,
//                 Direction::Left,
//             );
//         }

//         if self.vel_y() < 0 {
//             self.set_vel_y(0);
//         }

//         self.flags().set_hit_top_wall(true);
//     }
// }
void PhysicalEntity::test_hit_upper_left_slope(SharedGameState &state, int32_t x, int32_t y)
{
    auto tile_size = state.tile_size.as_int() * 0x200;
    auto half_tile_size = tile_size / 2;

    if ((this->x() < (x * 2 + 1) * half_tile_size) &&
        (this->x() > (x * 2 - 1) * half_tile_size) &&
        ((this->y() - this->hit_bounds().top) < (y * tile_size) + ((this->x() - x * tile_size) / 2)) &&
        ((this->y() + this->hit_bounds().bottom) > (y * 2 - 1) * half_tile_size))
    {
        this->set_y((y * tile_size) - (this->x() - x * tile_size) + this->hit_bounds().top);

        if (is_player() && !this->cond().hidden() && this->vel_y() < -0x200)
        {
            state.sound_manager.play_sfx(3);
            state.create_caret(
                this->x(),
                this->y() - this->hit_bounds().top,
                CaretType::LittleParticles,
                Direction::Left);
            state.create_caret(
                this->x(),
                this->y() - this->hit_bounds().top,
                CaretType::LittleParticles,
                Direction::Left);
        }

        if (this->vel_y() < 0)
        {
            this->set_vel_y(0);
        }

        this->flags().set_hit_top_wall(true);
    }
}

// // upper right slope
// fn test_hit_upper_right_slope(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
//     let tile_size = state.tile_size.as_int() * 0x200;
//     let half_tile_size = tile_size / 2;

//     if self.x() < (x * 2 + 1) * half_tile_size
//         && self.x() > (x * 2 - 1) * half_tile_size
//         && (self.y() - self.hit_bounds().top as i32) < (y * tile_size) + (self.x() - x * tile_size)
//         && (self.y() + self.hit_bounds().bottom as i32) > (y * 2 - 1) * half_tile_size
//     {
//         self.set_y((y * tile_size) + (self.x() - x * tile_size) + self.hit_bounds().top as i32);

//         if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
//             state.sound_manager.play_sfx(3);
//             state.create_caret(
//                 self.x(),
//                 self.y() - self.hit_bounds().top as i32,
//                 CaretType::LittleParticles,
//                 Direction::Left,
//             );
//             state.create_caret(
//                 self.x(),
//                 self.y() - self.hit_bounds().top as i32,
//                 CaretType::LittleParticles,
//                 Direction::Left,
//             );
//         }

//         if self.vel_y() < 0 {
//             self.set_vel_y(0);
//         }

//         self.flags().set_hit_top_wall(true);
//     }
// }
void PhysicalEntity::test_hit_upper_right_slope(SharedGameState &state, int32_t x, int32_t y)
{
    auto tile_size = state.tile_size.as_int() * 0x200;
    auto half_tile_size = tile_size / 2;

    if ((this->x() < (x * 2 + 1) * half_tile_size) &&
        (this->x() > (x * 2 - 1) * half_tile_size) &&
        ((this->y() - this->hit_bounds().top) < (y * tile_size) - ((this->x() - x * tile_size) / 2)) &&
        ((this->y() + this->hit_bounds().bottom) > (y * 2 - 1) * half_tile_size))
    {
        this->set_y((y * tile_size) - (this->x() - x * tile_size) + this->hit_bounds().top);

        if (is_player() && !this->cond().hidden() && this->vel_y() < -0x200)
        {
            state.sound_manager.play_sfx(3);
            state.create_caret(
                this->x(),
                this->y() - this->hit_bounds().top,
                CaretType::LittleParticles,
                Direction::Left);
            state.create_caret(
                this->x(),
                this->y() - this->hit_bounds().top,
                CaretType::LittleParticles,
                Direction::Left);
        }

        if (this->vel_y() < 0)
        {
            this->set_vel_y(0);
        }

        this->flags().set_hit_top_wall(true);
    }
}

// // lower left slope
// fn test_hit_lower_left_slope(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
//     let tile_size = state.tile_size.as_int() * 0x200;
//     let half_tile_size = tile_size / 2;
//     let quarter_tile_size = half_tile_size / 2;

//     self.flags().set_hit_left_higher_half(true);

//     if self.x() < (x * 2 + 1) * half_tile_size
//         && self.x() > (x * 2 - 1) * half_tile_size
//         && (self.y() + self.hit_bounds().bottom as i32)
//             > (y * tile_size) + (self.x() - x * tile_size) - quarter_tile_size
//         && (self.y() - self.hit_bounds().top as i32) < (y * 2 + 1) * half_tile_size
//     {
//         self.set_y(
//             (y * tile_size) + (self.x() - x * tile_size) - quarter_tile_size - self.hit_bounds().bottom as i32,
//         );

//         if self.is_player() && self.vel_y() > 0x400 {
//             state.sound_manager.play_sfx(23);
//         }

//         if self.vel_y() > 0 {
//             self.set_vel_y(0);
//         }

//         self.flags().set_hit_left_slope(true);
//         self.flags().set_hit_bottom_wall(true);
//     }
// }
void PhysicalEntity::test_hit_lower_left_slope(SharedGameState &state, int32_t x, int32_t y)
{
    auto tile_size = state.tile_size.as_int() * 0x200;
    auto half_tile_size = tile_size / 2;
    auto quarter_tile_size = half_tile_size / 2;

    this->flags().set_hit_left_higher_half(true);

    if ((this->x() < (x * 2 + 1) * half_tile_size) &&
        (this->x() > (x * 2 - 1) * half_tile_size) &&
        ((this->y() + this->hit_bounds().bottom) > (y * tile_size) - ((this->x() - x * tile_size) / 2)) &&
        ((this->y() - this->hit_bounds().top) < (y * 2 + 1) * half_tile_size))
    {
        this->set_y(
            (y * tile_size) - (this->x() - x * tile_size) + quarter_tile_size + this->hit_bounds().bottom);

        if (is_player() && this->vel_y() > 0x400)
        {
            state.sound_manager.play_sfx(23);
        }

        if (this->vel_y() > 0)
        {
            this->set_vel_y(0);
        }

        this->flags().set_hit_left_slope(true);
        this->flags().set_hit_bottom_wall(true);
    }
}

// // lower right slope
// fn test_hit_lower_right_slope(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
//     let tile_size = state.tile_size.as_int() * 0x200;
//     let half_tile_size = tile_size / 2;
//     let quarter_tile_size = half_tile_size / 2;

//     self.flags().set_hit_right_higher_half(true);

//     if (self.x() < (x * 2 + 1) * half_tile_size)
//         && (self.x() > (x * 2 - 1) * half_tile_size)
//         && (self.y() + self.hit_bounds().bottom as i32)
//             > (y * tile_size) - (self.x() - x * tile_size) - quarter_tile_size
//         && (self.y() - self.hit_bounds().top as i32) < (y * 2 + 1) * half_tile_size
//     {
//         self.set_y(
//             (y * tile_size) - (self.x() - x * tile_size) - quarter_tile_size - self.hit_bounds().bottom as i32,
//         );

//         if self.is_player() && self.vel_y() > 0x400 {
//             state.sound_manager.play_sfx(23);
//         }

//         if self.vel_y() > 0 {
//             self.set_vel_y(0);
//         }

//         self.flags().set_hit_right_slope(true);
//         self.flags().set_hit_bottom_wall(true);
//     }
// }
void PhysicalEntity::test_hit_lower_right_slope(SharedGameState &state, int32_t x, int32_t y)
{
    auto tile_size = state.tile_size.as_int() * 0x200;
    auto half_tile_size = tile_size / 2;
    auto quarter_tile_size = half_tile_size / 2;

    this->flags().set_hit_right_higher_half(true);

    if ((this->x() < (x * 2 + 1) * half_tile_size) &&
        (this->x() > (x * 2 - 1) * half_tile_size) &&
        ((this->y() + this->hit_bounds().bottom) > (y * tile_size) + ((this->x() - x * tile_size) / 2)) &&
        ((this->y() - this->hit_bounds().top) < (y * 2 + 1) * half_tile_size))
    {
        this->set_y(
            (y * tile_size) + (this->x() - x * tile_size) - quarter_tile_size + this->hit_bounds().bottom);

        if (is_player() && this->vel_y() > 0x400)
        {
            state.sound_manager.play_sfx(23);
        }

        if (this->vel_y() > 0)
        {
            this->set_vel_y(0);
        }

        this->flags().set_hit_right_slope(true);
        this->flags().set_hit_bottom_wall(true);
    }
}

// fn test_hit_water(&mut self, state: &SharedGameState, x: i32, y: i32) {
//     let tile_size = state.tile_size.as_int() * 0x200;
//     let mult = tile_size / 16;
//     let bounds_x = if self.is_player() { 5 } else { 6 } * mult;
//     let bounds_up = if self.is_player() { 5 } else { 6 } * mult;
//     let bounds_down = if self.is_player() { 0 } else { 6 } * mult;

//     if (self.x() - self.hit_bounds().right as i32) < x * tile_size + bounds_x
//         && (self.x() + self.hit_bounds().right as i32) > x * tile_size - bounds_x
//         && (self.y() - self.hit_bounds().top as i32) < y * tile_size + bounds_up
//         && (self.y() + self.hit_bounds().bottom as i32) > y * tile_size - bounds_down
//     {
//         self.flags().set_in_water(true);
//     }
// }
void PhysicalEntity::test_hit_water(SharedGameState &state, int32_t x, int32_t y)
{
    auto tile_size = state.tile_size.as_int() * 0x200;
    auto mult = tile_size / 16;
    auto bounds_x = (is_player() ? 5 : 6) * mult;
    auto bounds_up = (is_player() ? 5 : 6) * mult;
    auto bounds_down = (is_player() ? 0 : 6) * mult;

    if ((this->x() - this->hit_bounds().right) < (x * tile_size) + bounds_x &&
        (this->x() + this->hit_bounds().right) > (x * tile_size) - bounds_x &&
        (this->y() - this->hit_bounds().top) < (y * tile_size) + bounds_up &&
        (this->y() + this->hit_bounds().bottom) > (y * tile_size) - bounds_down)
    {
        this->flags().set_in_water(true);
    }
}

// fn test_hit_spike(&mut self, state: &SharedGameState, x: i32, y: i32, water: bool) {
//     let mult = state.tile_size.as_int() * 0x200 / 16;

//     if (self.x() - 0x800) < (x * 16 + 4) * mult
//         && (self.x() + 0x800) > (x * 16 - 4) * mult
//         && (self.y() - 0x800) < (y * 16 + 3) * mult
//         && (self.y() + 0x800) > (y * 16 - 3) * mult
//     {
//         self.flags().set_hit_by_spike(true);
//         if water {
//             self.flags().set_in_water(true);
//         }
//     }
// }
void PhysicalEntity::test_hit_spike(SharedGameState &state, int32_t x, int32_t y, bool water)
{
    auto mult = state.tile_size.as_int() * 0x200 / 16;

    if ((this->x() - 0x800) < (x * 16 + 4) * mult &&
        (this->x() + 0x800) > (x * 16 - 4) * mult &&
        (this->y() - 0x800) < (y * 16 + 3) * mult &&
        (this->y() + 0x800) > (y * 16 - 3) * mult)
    {
        this->flags().set_hit_by_spike(true);
        if (water)
        {
            this->flags().set_in_water(true);
        }
    }
}

// fn test_hit_force(&mut self, state: &SharedGameState, x: i32, y: i32, direction: Direction, water: bool) {
//     let mult = state.tile_size.as_int() * 0x200 / 16;

//     if (self.x() - self.hit_bounds().left as i32) < (x * 16 + 6) * mult
//         && (self.x() + self.hit_bounds().right as i32) > (x * 16 - 6) * mult
//         && (self.y() - self.hit_bounds().top as i32) < (y * 16 + 6) * mult
//         && (self.y() + self.hit_bounds().bottom as i32) > (y * 16 - 6) * mult
//     {
//         match direction {
//             Direction::Left => self.flags().set_force_left(true),
//             Direction::Up => self.flags().set_force_up(true),
//             Direction::Right => self.flags().set_force_right(true),
//             Direction::Bottom => self.flags().set_force_down(true),
//             Direction::FacingPlayer => unreachable!(),
//         }

//         if water {
//             self.flags().set_in_water(true);
//         }
//     }
// }
void PhysicalEntity::test_hit_force(SharedGameState &state, int32_t x, int32_t y, Direction direction, bool water)
{
    auto mult = state.tile_size.as_int() * 0x200 / 16;

    if ((this->x() - this->hit_bounds().left) < (x * 16 + 6) * mult &&
        (this->x() + this->hit_bounds().right) > (x * 16 - 6) * mult &&
        (this->y() - this->hit_bounds().top) < (y * 16 + 6) * mult &&
        (this->y() + this->hit_bounds().bottom) > (y * 16 - 6) * mult)
    {
        switch (direction)
        {
        case Direction::Left:
            this->flags().set_force_left(true);
            break;
        case Direction::Up:
            this->flags().set_force_up(true);
            break;
        case Direction::Right:
            this->flags().set_force_right(true);
            break;
        case Direction::Bottom:
            this->flags().set_force_down(true);
            break;
        case Direction::FacingPlayer:
            common::unreachable();
            break;
        }

        if (water)
        {
            this->flags().set_in_water(true);
        }
    }
}

void PhysicalEntity::tick_map_collisions(SharedGameState &state, NPCList &npc_list, Stage &stage)
{
    auto hit_rect_size = std::clamp(this->hit_rect_size(), (uintptr_t)1, (uintptr_t)4);
    hit_rect_size = state.tile_size == TileSize::Tile8x8
                        ? (hit_rect_size * hit_rect_size * 4)
                        : (hit_rect_size * hit_rect_size);

    auto tile_size = state.tile_size == TileSize::Tile8x8 ? 8 : 16;
    auto x = (this->x() + this->offset_x()) / tile_size;
    auto y = (this->y() + this->offset_y()) / tile_size;

    this->flags().value = 0;
    for (auto idx = 0; idx < hit_rect_size; idx++)
    {
        auto [ox, oy] = OFFSETS[idx];
        auto attrib = stage.map.get_attribute((x + ox), (y + oy));

        if ((attrib == 0x62 || attrib == 0x42) && this->is_player())
        {
            this->test_hit_spike(state, x + ox, y + oy, attrib & 0x20 != 0);
        }
        else if (attrib == 0x02 || attrib == 0x60)
        {
            this->test_hit_water(state, x + ox, y + oy);
        }
        else if (attrib == 0x62 && !this->is_player())
        {
            this->test_hit_water(state, x + ox, y + oy);
        }
        else if (attrib == 0x61)
        {
            this->test_block_hit(state, x + ox, y + oy);
            this->test_hit_water(state, x + ox, y + oy);
        }
        else if ((attrib == 0x04 || attrib == 0x64) && !this->is_player())
        {
            this->test_block_hit(state, x + ox, y + oy);
            this->test_hit_water(state, x + ox, y + oy);
        }
        else if ((attrib == 0x05 || attrib == 0x41 || attrib == 0x43 || attrib == 0x46) && this->is_player())
        {
            this->test_block_hit(state, x + ox, y + oy);
        }
        else if ((attrib == 0x03 || attrib == 0x05 || attrib == 0x41 || attrib == 0x43) && !this->is_player())
        {
            this->test_block_hit(state, x + ox, y + oy);
        }
        else if (attrib == 0x44)
        {
            if (!this->ignore_tile_44())
            {
                this->test_block_hit(state, x + ox, y + oy);
            }
        }
        else if (attrib == 0x4a)
        {
            this->test_platform_hit(state, x + ox, y + oy);
        }
        else if (attrib == 0x50 || attrib == 0x70)
        {
            this->test_hit_upper_left_slope_high(state, x + ox, y + oy);
            if (attrib & 0x20 != 0)
            {
                this->test_hit_water(state, x + ox, y + oy);
            }
        }
        else if (attrib == 0x51 || attrib == 0x71)
        {
            this->test_hit_upper_left_slope_low(state, x + ox, y + oy);
            if (attrib & 0x20 != 0)
            {
                this->test_hit_water(state, x + ox, y + oy);
            }
        }
        else if (attrib == 0x52 || attrib == 0x72)
        {
            this->test_hit_upper_right_slope_low(state, x + ox, y + oy);
            if (attrib & 0x20 != 0)
            {
                this->test_hit_water(state, x + ox, y + oy);
            }
        }
        else if (attrib == 0x53 || attrib == 0x73)
        {
            this->test_hit_upper_right_slope_high(state, x + ox, y + oy);
            if (attrib & 0x20 != 0)
            {
                this->test_hit_water(state, x + ox, y + oy);
            }
        }
        else if (attrib == 0x54 || attrib == 0x74)
        {
            this->test_hit_lower_left_slope_high(state, x + ox, y + oy);
            if (attrib & 0x20 != 0)
            {
                this->test_hit_water(state, x + ox, y + oy);
            }
        }
        else if (attrib == 0x55 || attrib == 0x75)
        {
            this->test_hit_lower_left_slope_low(state, x + ox, y + oy);
            if (attrib & 0x20 != 0)
            {
                this->test_hit_water(state, x + ox, y + oy);
            }
        }
        else if (attrib == 0x56 || attrib == 0x76)
        {
            this->test_hit_lower_right_slope_low(state, x + ox, y + oy);
            if (attrib & 0x20 != 0)
            {
                this->test_hit_water(state, x + ox, y + oy);
            }
        }
        else if (attrib == 0x57 || attrib == 0x77)
        {
            this->test_hit_lower_right_slope_high(state, x + ox, y + oy);
            if (attrib & 0x20 != 0)
            {
                this->test_hit_water(state, x + ox, y + oy);
            }
        }
        else if (attrib == 0x5a || attrib == 0x7a)
        {
            this->test_hit_upper_left_slope(state, x + ox, y + oy);
            if (attrib & 0x20 != 0)
            {
                this->test_hit_water(state, x + ox, y + oy);
            }
        }
        else if (attrib == 0x5b || attrib == 0x7b)
        {
            this->test_hit_upper_right_slope(state, x + ox, y + oy);
            if (attrib & 0x20 != 0)
            {
                this->test_hit_water(state, x + ox, y + oy);
            }
        }
        else if (attrib == 0x5c || attrib == 0x7c)
        {
            this->test_hit_lower_left_slope(state, x + ox, y + oy);
            if (attrib & 0x20 != 0)
            {
                this->test_hit_water(state, x + ox, y + oy);
            }
        }
        else if (attrib == 0x5d || attrib == 0x7d)
        {
            this->test_hit_lower_right_slope(state, x + ox, y + oy);
            if (attrib & 0x20 != 0)
            {
                this->test_hit_water(state, x + ox, y + oy);
            }
        }
        else if ((attrib == 0x80 || attrib == 0xa0) && this->is_player())
        {
            this->test_hit_force(state, x + ox, y + oy, Direction::Left, attrib & 0x20 != 0);
        }
        else if ((attrib == 0x81 || attrib == 0xa1) && this->is_player())
        {
            this->test_hit_force(state, x + ox, y + oy, Direction::Up, attrib & 0x20 != 0);
        }
        else if ((attrib == 0x82 || attrib == 0xa2) && this->is_player())
        {
            this->test_hit_force(state, x + ox, y + oy, Direction::Right, attrib & 0x20 != 0);
        }
        else if ((attrib == 0x83 || attrib == 0xa3) && this->is_player())
        {
            this->test_hit_force(state, x + ox, y + oy, Direction::Bottom, attrib & 0x20 != 0);
        }
        else if ((attrib == 0x80 || attrib == 0xa0) && !this->is_player())
        {
            this->flags().set_force_left(true);
            if (attrib & 0x20 != 0)
            {
                this->flags().set_in_water(true);
            }
        }
        else if ((attrib == 0x81 || attrib == 0xa1) && !this->is_player())
        {
            this->flags().set_force_up(true);
            if (attrib & 0x20 != 0)
            {
                this->flags().set_in_water(true);
            }
        }
        else if ((attrib == 0x82 || attrib == 0xa2) && !this->is_player())
        {
            this->flags().set_force_right(true);
            if (attrib & 0x20 != 0)
            {
                this->flags().set_in_water(true);
            }
        }
        else if ((attrib == 0x83 || attrib == 0xa3) && !this->is_player())
        {
            this->flags().set_force_down(true);
            if (attrib & 0x20 != 0)
            {
                this->flags().set_in_water(true);
            }
        }
    }

    if ((this->y() - 0x800) > state.water_level)
    {
        this->flags().set_in_water(true);
    }
}