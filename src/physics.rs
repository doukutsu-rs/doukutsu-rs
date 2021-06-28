use num_traits::clamp;

use crate::caret::CaretType;
use crate::common::{Condition, Direction, Flag, Rect};
use crate::npc::list::NPCList;
use crate::shared_game_state::{SharedGameState, TileSize};
use crate::stage::Stage;

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

pub const OFFSETS: [(i32, i32); 64] = [
    (0, 0),
    (1, 0),
    (0, 1),
    (1, 1),
    (2, 0),
    (2, 1),
    (2, 2),
    (0, 2),
    (1, 2),
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (-1, 2),
    (0, -1),
    (1, -1),
    (2, -1),
    (3, 3),
    (3, -1),
    (3, 0),
    (3, 1),
    (3, 2),
    (-1, 3),
    (0, 3),
    (1, 3),
    (2, 3),
    (-2, -2),
    (-2, -1),
    (-2, 0),
    (-2, 1),
    (-2, 2),
    (-2, 3),
    (-1, -2),
    (0, -2),
    (1, -2),
    (2, -2),
    (3, -2),
    (-3, -3),
    (-3, -2),
    (-3, -1),
    (-3, 0),
    (-3, 1),
    (-3, 2),
    (-3, 3),
    (-2, -3),
    (-1, -3),
    (0, -3),
    (1, -3),
    (2, -3),
    (3, -3),
    (4, -3),
    (4, -2),
    (4, -1),
    (4, 0),
    (4, 1),
    (4, 2),
    (4, 3),
    (-3, 4),
    (-2, 4),
    (-1, 4),
    (0, 4),
    (1, 4),
    (2, 4),
    (3, 4),
    (4, 4),
];

pub trait PhysicalEntity {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn vel_x(&self) -> i32;
    fn vel_y(&self) -> i32;

    fn hit_rect_size(&self) -> usize;
    fn offset_x(&self) -> i32 {
        0
    }
    fn offset_y(&self) -> i32 {
        0
    }

    fn hit_bounds(&self) -> &Rect<u32>;
    fn display_bounds(&self) -> &Rect<u32>;

    fn set_x(&mut self, x: i32);
    fn set_y(&mut self, y: i32);
    fn set_vel_x(&mut self, x: i32);
    fn set_vel_y(&mut self, y: i32);

    fn cond(&mut self) -> &mut Condition;
    fn flags(&mut self) -> &mut Flag;

    fn direction(&self) -> Direction;
    fn is_player(&self) -> bool;
    fn ignore_tile_44(&self) -> bool {
        true
    }
    fn player_left_pressed(&self) -> bool {
        false
    }
    fn player_right_pressed(&self) -> bool {
        false
    }

    fn test_block_hit(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let bounds_x = if self.is_player() { 0x600 } else { 0x600 };
        let bounds_top = if self.is_player() { 0x800 } else { 0x600 };
        let bounds_bottom = if self.is_player() { 0x800 } else { 0x600 };
        let half_tile_size = state.tile_size.as_int() * 0x100;

        // left wall
        if (self.y() - self.hit_bounds().top as i32) < ((y * 2 + 1) * half_tile_size - bounds_top)
            && (self.y() + self.hit_bounds().bottom as i32) > ((y * 2 - 1) * half_tile_size + bounds_bottom)
            && (self.x() - self.hit_bounds().right as i32) < (x * 2 + 1) * half_tile_size
            && (self.x() - self.hit_bounds().right as i32) > (x * 2) * half_tile_size
        {
            self.set_x(((x * 2 + 1) * half_tile_size) + self.hit_bounds().right as i32);

            if self.is_player() {
                if self.vel_x() < -0x180 {
                    self.set_vel_x(-0x180);
                }

                if !self.player_left_pressed() && self.vel_x() < 0 {
                    self.set_vel_x(0);
                }
            }

            self.flags().set_hit_left_wall(true);
        }

        // right wall
        if (self.y() - self.hit_bounds().top as i32) < ((y * 2 + 1) * half_tile_size - bounds_top)
            && (self.y() + self.hit_bounds().bottom as i32) > ((y * 2 - 1) * half_tile_size + bounds_bottom)
            && (self.x() + self.hit_bounds().right as i32) > (x * 2 - 1) * half_tile_size
            && (self.x() + self.hit_bounds().right as i32) < (x * 2) * half_tile_size
        {
            self.set_x(((x * 2 - 1) * half_tile_size) - self.hit_bounds().right as i32);

            if self.is_player() {
                if self.vel_x() > 0x180 {
                    self.set_vel_x(0x180);
                }

                if !self.player_right_pressed() && self.vel_x() > 0 {
                    self.set_vel_x(0);
                }
            }

            self.flags().set_hit_right_wall(true);
        }

        // ceiling
        if ((self.x() - self.hit_bounds().right as i32) < (x * 2 + 1) * half_tile_size - bounds_x)
            && ((self.x() + self.hit_bounds().right as i32) > (x * 2 - 1) * half_tile_size + bounds_x)
            && (self.y() - self.hit_bounds().top as i32) < (y * 2 + 1) * half_tile_size
            && (self.y() - self.hit_bounds().top as i32) > (y * 2) * half_tile_size
        {
            self.set_y(((y * 2 + 1) * half_tile_size) + self.hit_bounds().top as i32);

            if self.is_player() {
                if !self.cond().hidden() && self.vel_y() < -0x200 {
                    state.sound_manager.play_sfx(3);
                    state.create_caret(
                        self.x(),
                        self.y() - self.hit_bounds().top as i32,
                        CaretType::LittleParticles,
                        Direction::Left,
                    );
                    state.create_caret(
                        self.x(),
                        self.y() - self.hit_bounds().top as i32,
                        CaretType::LittleParticles,
                        Direction::Left,
                    );
                }

                if self.vel_y() < 0 {
                    self.set_vel_y(0);
                }
            } else {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }

        // floor
        if ((self.x() - self.hit_bounds().right as i32) < (x * 2 + 1) * half_tile_size - bounds_x)
            && ((self.x() + self.hit_bounds().right as i32) > (x * 2 - 1) * half_tile_size + bounds_x)
            && ((self.y() + self.hit_bounds().bottom as i32) > ((y * 2 - 1) * half_tile_size))
            && ((self.y() + self.hit_bounds().bottom as i32) < (y * 2) * half_tile_size)
        {
            self.set_y(((y * 2 - 1) * half_tile_size) - self.hit_bounds().bottom as i32);

            if self.is_player() {
                if self.vel_y() > 0x400 {
                    state.sound_manager.play_sfx(23);
                }

                if self.vel_y() > 0 {
                    self.set_vel_y(0);
                }
            } else {
                self.set_vel_y(0);
            }

            self.flags().set_hit_bottom_wall(true);
        }
    }

    fn test_platform_hit(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let half_tile_size = state.tile_size.as_int() * 0x100;

        if ((self.x() - self.hit_bounds().right as i32) < (x * 2 + 1) * half_tile_size)
            && ((self.x() + self.hit_bounds().right as i32) > (x * 2 - 1) * half_tile_size)
            && ((self.y() + self.hit_bounds().bottom as i32) > ((y * 2 - 1) * half_tile_size))
            && ((self.y() + self.hit_bounds().bottom as i32) < (y * 2 - 1) * half_tile_size + 0x400)
        {
            self.set_y(((y * 2 - 1) * half_tile_size) - self.hit_bounds().bottom as i32);

            if self.is_player() {
                if self.vel_y() > 0x400 {
                    state.sound_manager.play_sfx(23);
                }

                if self.vel_y() > 0 {
                    self.set_vel_y(0);
                }
            } else {
                self.set_vel_y(0);
            }

            self.flags().set_hit_bottom_wall(true);
        }
    }

    // upper left slope (bigger half)
    fn test_hit_upper_left_slope_high(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let tile_size = state.tile_size.as_int() * 0x200;
        let half_tile_size = tile_size / 2;
        let quarter_tile_size = half_tile_size / 2;

        if self.x() < (x * 2 + 1) * half_tile_size
            && self.x() > (x * 2 - 1) * half_tile_size
            && (self.y() - self.hit_bounds().top as i32)
                < (y * tile_size) - (self.x() - x * tile_size) / 2 + quarter_tile_size
            && (self.y() + self.hit_bounds().bottom as i32) > (y * 2 - 1) * half_tile_size
        {
            self.set_y(
                (y * tile_size) - ((self.x() - x * tile_size) / 2) + quarter_tile_size + self.hit_bounds().top as i32,
            );

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                state.sound_manager.play_sfx(3);
                state.create_caret(
                    self.x(),
                    self.y() - self.hit_bounds().top as i32,
                    CaretType::LittleParticles,
                    Direction::Left,
                );
                state.create_caret(
                    self.x(),
                    self.y() - self.hit_bounds().top as i32,
                    CaretType::LittleParticles,
                    Direction::Left,
                );
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // upper left slope (smaller half)
    fn test_hit_upper_left_slope_low(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let tile_size = state.tile_size.as_int() * 0x200;
        let half_tile_size = tile_size / 2;
        let quarter_tile_size = half_tile_size / 2;

        if self.x() < (x * 2 + 1) * half_tile_size
            && self.x() > (x * 2 - 1) * half_tile_size
            && (self.y() - self.hit_bounds().top as i32)
                < (y * tile_size) - (self.x() - x * tile_size) / 2 - quarter_tile_size
            && (self.y() + self.hit_bounds().bottom as i32) > (y * 2 - 1) * half_tile_size
        {
            self.set_y(
                (y * tile_size) - ((self.x() - x * tile_size) / 2) - quarter_tile_size + self.hit_bounds().top as i32,
            );

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                state.sound_manager.play_sfx(3);
                state.create_caret(
                    self.x(),
                    self.y() - self.hit_bounds().top as i32,
                    CaretType::LittleParticles,
                    Direction::Left,
                );
                state.create_caret(
                    self.x(),
                    self.y() - self.hit_bounds().top as i32,
                    CaretType::LittleParticles,
                    Direction::Left,
                );
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // upper right slope (smaller half)
    fn test_hit_upper_right_slope_low(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let tile_size = state.tile_size.as_int() * 0x200;
        let half_tile_size = tile_size / 2;
        let quarter_tile_size = half_tile_size / 2;

        if self.x() < (x * 2 + 1) * half_tile_size
            && self.x() > (x * 2 - 1) * half_tile_size
            && (self.y() - self.hit_bounds().top as i32)
                < (y * tile_size) + (self.x() - x * tile_size) / 2 - quarter_tile_size
            && (self.y() + self.hit_bounds().bottom as i32) > (y * 2 - 1) * half_tile_size
        {
            self.set_y(
                (y * tile_size) + ((self.x() - x * tile_size) / 2) - quarter_tile_size + self.hit_bounds().top as i32,
            );

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                state.sound_manager.play_sfx(3);
                state.create_caret(
                    self.x(),
                    self.y() - self.hit_bounds().top as i32,
                    CaretType::LittleParticles,
                    Direction::Left,
                );
                state.create_caret(
                    self.x(),
                    self.y() - self.hit_bounds().top as i32,
                    CaretType::LittleParticles,
                    Direction::Left,
                );
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // upper right slope (bigger half)
    fn test_hit_upper_right_slope_high(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let tile_size = state.tile_size.as_int() * 0x200;
        let half_tile_size = tile_size / 2;
        let quarter_tile_size = half_tile_size / 2;

        if self.x() < (x * 2 + 1) * half_tile_size
            && self.x() > (x * 2 - 1) * half_tile_size
            && (self.y() - self.hit_bounds().top as i32)
                < (y * tile_size) + (self.x() - x * tile_size) / 2 + quarter_tile_size
            && (self.y() + self.hit_bounds().bottom as i32) > (y * 2 - 1) * half_tile_size
        {
            self.set_y(
                (y * tile_size) + ((self.x() - x * tile_size) / 2) + quarter_tile_size + self.hit_bounds().top as i32,
            );

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                state.sound_manager.play_sfx(3);
                state.create_caret(
                    self.x(),
                    self.y() - self.hit_bounds().top as i32,
                    CaretType::LittleParticles,
                    Direction::Left,
                );
                state.create_caret(
                    self.x(),
                    self.y() - self.hit_bounds().top as i32,
                    CaretType::LittleParticles,
                    Direction::Left,
                );
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // lower left half (bigger)
    fn test_hit_lower_left_slope_high(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let tile_size = state.tile_size.as_int() * 0x200;
        let half_tile_size = tile_size / 2;
        let quarter_tile_size = half_tile_size / 2;

        self.flags().set_hit_left_higher_half(true);

        if self.x() < (x * 2 + 1) * half_tile_size
            && self.x() > (x * 2 - 1) * half_tile_size
            && (self.y() + self.hit_bounds().bottom as i32)
                > (y * tile_size) + (self.x() - x * tile_size) / 2 - quarter_tile_size
            && (self.y() - self.hit_bounds().top as i32) < (y * 2 + 1) * half_tile_size
        {
            self.set_y(
                (y * tile_size) + ((self.x() - x * tile_size) / 2)
                    - quarter_tile_size
                    - self.hit_bounds().bottom as i32,
            );

            if self.is_player() && self.vel_y() > 0x400 {
                state.sound_manager.play_sfx(23);
            }

            if self.vel_y() > 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_left_slope(true);
            self.flags().set_hit_bottom_wall(true);
        }
    }

    // lower left half (smaller)
    fn test_hit_lower_left_slope_low(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let tile_size = state.tile_size.as_int() * 0x200;
        let half_tile_size = tile_size / 2;
        let quarter_tile_size = half_tile_size / 2;

        self.flags().set_hit_left_lower_half(true);

        if (self.x() < (x * 2 + 1) * half_tile_size)
            && (self.x() > (x * 2 - 1) * half_tile_size)
            && (self.y() + self.hit_bounds().bottom as i32)
                > (y * tile_size) + (self.x() - x * tile_size) / 2 + quarter_tile_size
            && (self.y() - self.hit_bounds().top as i32) < (y * 2 + 1) * half_tile_size
        {
            self.set_y(
                (y * tile_size) + ((self.x() - x * tile_size) / 2) + quarter_tile_size
                    - self.hit_bounds().bottom as i32,
            );

            if self.is_player() && self.vel_y() > 0x400 {
                state.sound_manager.play_sfx(23);
            }

            if self.vel_y() > 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_left_slope(true);
            self.flags().set_hit_bottom_wall(true);
        }
    }

    // lower right half (smaller)
    fn test_hit_lower_right_slope_low(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let tile_size = state.tile_size.as_int() * 0x200;
        let half_tile_size = tile_size / 2;
        let quarter_tile_size = half_tile_size / 2;

        self.flags().set_hit_right_lower_half(true);

        if (self.x() < (x * 2 + 1) * half_tile_size)
            && (self.x() > (x * 2 - 1) * half_tile_size)
            && (self.y() + self.hit_bounds().bottom as i32)
                > (y * tile_size) - (self.x() - x * tile_size) / 2 + quarter_tile_size
            && (self.y() - self.hit_bounds().top as i32) < (y * 2 + 1) * half_tile_size
        {
            self.set_y(
                (y * tile_size) - ((self.x() - x * tile_size) / 2) + quarter_tile_size
                    - self.hit_bounds().bottom as i32,
            );

            if self.is_player() && self.vel_y() > 0x400 {
                state.sound_manager.play_sfx(23);
            }

            if self.vel_y() > 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_right_slope(true);
            self.flags().set_hit_bottom_wall(true);
        }
    }

    // lower right half (bigger)
    fn test_hit_lower_right_slope_high(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let tile_size = state.tile_size.as_int() * 0x200;
        let half_tile_size = tile_size / 2;
        let quarter_tile_size = half_tile_size / 2;

        self.flags().set_hit_right_higher_half(true);

        if (self.x() < (x * 2 + 1) * half_tile_size)
            && (self.x() > (x * 2 - 1) * half_tile_size)
            && (self.y() + self.hit_bounds().bottom as i32)
                > (y * tile_size) - (self.x() - x * tile_size) / 2 - quarter_tile_size
            && (self.y() - self.hit_bounds().top as i32) < (y * 2 + 1) * half_tile_size
        {
            self.set_y(
                (y * tile_size)
                    - ((self.x() - x * tile_size) / 2)
                    - quarter_tile_size
                    - self.hit_bounds().bottom as i32,
            );

            if self.is_player() && self.vel_y() > 0x400 {
                state.sound_manager.play_sfx(23);
            }

            if self.vel_y() > 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_right_slope(true);
            self.flags().set_hit_bottom_wall(true);
        }
    }

    // upper left slope
    fn test_hit_upper_left_slope(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let tile_size = state.tile_size.as_int() * 0x200;
        let half_tile_size = tile_size / 2;

        if self.x() < (x * 2 + 1) * half_tile_size
            && self.x() > (x * 2 - 1) * half_tile_size
            && (self.y() - self.hit_bounds().top as i32) < (y * tile_size) - (self.x() - x * tile_size)
            && (self.y() + self.hit_bounds().bottom as i32) > (y * 2 - 1) * half_tile_size
        {
            self.set_y(
                (y * tile_size) - (self.x() - x * tile_size) + self.hit_bounds().top as i32
            );

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                state.sound_manager.play_sfx(3);
                state.create_caret(
                    self.x(),
                    self.y() - self.hit_bounds().top as i32,
                    CaretType::LittleParticles,
                    Direction::Left,
                );
                state.create_caret(
                    self.x(),
                    self.y() - self.hit_bounds().top as i32,
                    CaretType::LittleParticles,
                    Direction::Left,
                );
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // upper right slope
    fn test_hit_upper_right_slope(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let tile_size = state.tile_size.as_int() * 0x200;
        let half_tile_size = tile_size / 2;

        if self.x() < (x * 2 + 1) * half_tile_size
            && self.x() > (x * 2 - 1) * half_tile_size
            && (self.y() - self.hit_bounds().top as i32) < (y * tile_size) + (self.x() - x * tile_size)
            && (self.y() + self.hit_bounds().bottom as i32) > (y * 2 - 1) * half_tile_size
        {
            self.set_y(
                (y * tile_size) + (self.x() - x * tile_size) + self.hit_bounds().top as i32
            );

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                state.sound_manager.play_sfx(3);
                state.create_caret(
                    self.x(),
                    self.y() - self.hit_bounds().top as i32,
                    CaretType::LittleParticles,
                    Direction::Left,
                );
                state.create_caret(
                    self.x(),
                    self.y() - self.hit_bounds().top as i32,
                    CaretType::LittleParticles,
                    Direction::Left,
                );
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // lower left slope
    fn test_hit_lower_left_slope(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let tile_size = state.tile_size.as_int() * 0x200;
        let half_tile_size = tile_size / 2;
        let quarter_tile_size = half_tile_size / 2;

        self.flags().set_hit_left_higher_half(true);

        if self.x() < (x * 2 + 1) * half_tile_size
            && self.x() > (x * 2 - 1) * half_tile_size
            && (self.y() + self.hit_bounds().bottom as i32) > (y * tile_size) + (self.x() - x * tile_size) - quarter_tile_size
            && (self.y() - self.hit_bounds().top as i32) < (y * 2 + 1) * half_tile_size
        {
            self.set_y((y * tile_size) + (self.x() - x * tile_size) - quarter_tile_size - self.hit_bounds().bottom as i32);

            if self.is_player() && self.vel_y() > 0x400 {
                state.sound_manager.play_sfx(23);
            }

            if self.vel_y() > 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_left_slope(true);
            self.flags().set_hit_bottom_wall(true);
        }
    }

    // lower right slope
    fn test_hit_lower_right_slope(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        let tile_size = state.tile_size.as_int() * 0x200;
        let half_tile_size = tile_size / 2;
        let quarter_tile_size = half_tile_size / 2;

        self.flags().set_hit_right_higher_half(true);

        if (self.x() < (x * 2 + 1) * half_tile_size)
            && (self.x() > (x * 2 - 1) * half_tile_size)
            && (self.y() + self.hit_bounds().bottom as i32) > (y * tile_size) - (self.x() - x * tile_size) - quarter_tile_size
            && (self.y() - self.hit_bounds().top as i32) < (y * 2 + 1) * half_tile_size
        {
            self.set_y((y * tile_size) - (self.x() - x * tile_size) - quarter_tile_size - self.hit_bounds().bottom as i32);

            if self.is_player() && self.vel_y() > 0x400 {
                state.sound_manager.play_sfx(23);
            }

            if self.vel_y() > 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_right_slope(true);
            self.flags().set_hit_bottom_wall(true);
        }
    }

    fn test_hit_water(&mut self, state: &SharedGameState, x: i32, y: i32) {
        let tile_size = state.tile_size.as_int() * 0x200;
        let mult = tile_size / 16;
        let bounds_x = if self.is_player() { 5 } else { 6 } * mult;
        let bounds_up = if self.is_player() { 5 } else { 6 } * mult;
        let bounds_down = if self.is_player() { 0 } else { 6 } * mult;

        if (self.x() - self.hit_bounds().right as i32) < x * tile_size + bounds_x
            && (self.x() + self.hit_bounds().right as i32) > x * tile_size - bounds_x
            && (self.y() - self.hit_bounds().top as i32) < y * tile_size + bounds_up
            && (self.y() + self.hit_bounds().bottom as i32) > y * tile_size - bounds_down
        {
            self.flags().set_in_water(true);
        }
    }

    fn test_hit_spike(&mut self, state: &SharedGameState, x: i32, y: i32, water: bool) {
        let mult = state.tile_size.as_int() * 0x200 / 16;

        if (self.x() - 0x800) < (x * 16 + 4) * mult
            && (self.x() + 0x800) > (x * 16 - 4) * mult
            && (self.y() - 0x800) < (y * 16 + 3) * mult
            && (self.y() + 0x800) > (y * 16 - 3) * mult
        {
            self.flags().set_hit_by_spike(true);
            if water {
                self.flags().set_in_water(true);
            }
        }
    }

    fn test_hit_force(&mut self, state: &SharedGameState, x: i32, y: i32, direction: Direction, water: bool) {
        let mult = state.tile_size.as_int() * 0x200 / 16;

        if (self.x() - self.hit_bounds().left as i32) < (x * 16 + 6) * mult
            && (self.x() + self.hit_bounds().right as i32) > (x * 16 - 6) * mult
            && (self.y() - self.hit_bounds().top as i32) < (y * 16 + 6) * mult
            && (self.y() + self.hit_bounds().bottom as i32) > (y * 16 - 6) * mult
        {
            match direction {
                Direction::Left => self.flags().set_force_left(true),
                Direction::Up => self.flags().set_force_up(true),
                Direction::Right => self.flags().set_force_right(true),
                Direction::Bottom => self.flags().set_force_down(true),
                Direction::FacingPlayer => unreachable!(),
            }

            if water {
                self.flags().set_in_water(true);
            }
        }
    }

    fn tick_map_collisions(&mut self, state: &mut SharedGameState, _npc_list: &NPCList, stage: &mut Stage) {
        let hit_rect_size = self.hit_rect_size().clamp(1, 4);
        let hit_rect_size = if state.tile_size == TileSize::Tile8x8 {
            4 * hit_rect_size * hit_rect_size
        } else {
            hit_rect_size * hit_rect_size
        };

        let tile_size = state.tile_size.as_int() * 0x200;
        let x = (self.x() + self.offset_x()) / tile_size;
        let y = (self.y() + self.offset_y()) / tile_size;

        self.flags().0 = 0;
        for (idx, &(ox, oy)) in OFFSETS.iter().enumerate() {
            if idx == hit_rect_size {
                break;
            }

            let attrib = stage.map.get_attribute((x + ox) as usize, (y + oy) as usize);
            match attrib {
                // Spikes
                0x62 | 0x42 if self.is_player() => {
                    self.test_hit_spike(state, x + ox, y + oy, attrib & 0x20 != 0);
                }

                // Blocks
                0x02 | 0x60 => {
                    self.test_hit_water(state, x + ox, y + oy);
                }
                0x62 if !self.is_player() => {
                    self.test_hit_water(state, x + ox, y + oy);
                }
                0x61 => {
                    self.test_block_hit(state, x + ox, y + oy);
                    self.test_hit_water(state, x + ox, y + oy);
                }
                0x04 | 0x64 if !self.is_player() => {
                    self.test_block_hit(state, x + ox, y + oy);
                    self.test_hit_water(state, x + ox, y + oy);
                }
                0x05 | 0x41 | 0x43 | 0x46 if self.is_player() => {
                    self.test_block_hit(state, x + ox, y + oy);
                }
                0x03 | 0x05 | 0x41 | 0x43 if !self.is_player() => {
                    self.test_block_hit(state, x + ox, y + oy);
                }
                0x44 => {
                    if !self.ignore_tile_44() {
                        self.test_block_hit(state, x + ox, y + oy);
                    }
                }
                0x4a => {
                    self.test_platform_hit(state, x + ox, y + oy);
                }

                // Slopes
                0x50 | 0x70 => {
                    self.test_hit_upper_left_slope_high(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 {
                        self.test_hit_water(state, x + ox, y + oy);
                    }
                }
                0x51 | 0x71 => {
                    self.test_hit_upper_left_slope_low(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 {
                        self.test_hit_water(state, x + ox, y + oy);
                    }
                }
                0x52 | 0x72 => {
                    self.test_hit_upper_right_slope_low(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 {
                        self.test_hit_water(state, x + ox, y + oy);
                    }
                }
                0x53 | 0x73 => {
                    self.test_hit_upper_right_slope_high(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 {
                        self.test_hit_water(state, x + ox, y + oy);
                    }
                }
                0x54 | 0x74 => {
                    self.test_hit_lower_left_slope_high(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 {
                        self.test_hit_water(state, x + ox, y + oy);
                    }
                }
                0x55 | 0x75 => {
                    self.test_hit_lower_left_slope_low(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 {
                        self.test_hit_water(state, x + ox, y + oy);
                    }
                }
                0x56 | 0x76 => {
                    self.test_hit_lower_right_slope_low(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 {
                        self.test_hit_water(state, x + ox, y + oy);
                    }
                }
                0x57 | 0x77 => {
                    self.test_hit_lower_right_slope_high(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 {
                        self.test_hit_water(state, x + ox, y + oy);
                    }
                }
                0x5a | 0x7a => {
                    self.test_hit_upper_left_slope(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 {
                        self.test_hit_water(state, x + ox, y + oy);
                    }
                }
                0x5b | 0x7b => {
                    self.test_hit_upper_right_slope(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 {
                        self.test_hit_water(state, x + ox, y + oy);
                    }
                }
                0x5c | 0x7c => {
                    self.test_hit_lower_left_slope(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 {
                        self.test_hit_water(state, x + ox, y + oy);
                    }
                }
                0x5d | 0x7d => {
                    self.test_hit_lower_right_slope(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 {
                        self.test_hit_water(state, x + ox, y + oy);
                    }
                }

                // Forces
                0x80 | 0xa0 if self.is_player() => {
                    self.test_hit_force(state, x + ox, y + oy, Direction::Left, attrib & 0x20 != 0);
                }
                0x81 | 0xa1 if self.is_player() => {
                    self.test_hit_force(state, x + ox, y + oy, Direction::Up, attrib & 0x20 != 0);
                }
                0x82 | 0xa2 if self.is_player() => {
                    self.test_hit_force(state, x + ox, y + oy, Direction::Right, attrib & 0x20 != 0);
                }
                0x83 | 0xa3 if self.is_player() => {
                    self.test_hit_force(state, x + ox, y + oy, Direction::Bottom, attrib & 0x20 != 0);
                }
                0x80 | 0xa0 if !self.is_player() => {
                    self.flags().set_force_left(true);
                    if attrib & 0x20 != 0 {
                        self.flags().set_in_water(true);
                    }
                }
                0x81 | 0xa1 if !self.is_player() => {
                    self.flags().set_force_up(true);
                    if attrib & 0x20 != 0 {
                        self.flags().set_in_water(true);
                    }
                }
                0x82 | 0xa2 if !self.is_player() => {
                    self.flags().set_force_right(true);
                    if attrib & 0x20 != 0 {
                        self.flags().set_in_water(true);
                    }
                }
                0x83 | 0xa3 if !self.is_player() => {
                    self.flags().set_force_down(true);
                    if attrib & 0x20 != 0 {
                        self.flags().set_in_water(true);
                    }
                }
                _ => {}
            }
        }
    }
}
