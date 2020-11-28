use num_traits::clamp;

use crate::caret::CaretType;
use crate::common::{Condition, Direction, Flag, Rect};
use crate::shared_game_state::SharedGameState;
use crate::stage::Stage;

//      -1  0  1  2
//    +------------
// -1 | 10 14 15 16
//  0 | 11  1  2  5
//  1 | 12  3  4  6
//  2 | 13  8  9  7
pub const OFF_X: [isize; 16] = [0, 1, 0, 1, 2, 2, 2, 0, 1, -1, -1, -1, -1, 0, 1, 2];
pub const OFF_Y: [isize; 16] = [0, 0, 1, 1, 0, 1, 2, 2, 2, -1, 0, 1, 2, -1, -1, -1];

pub trait PhysicalEntity {
    fn x(&self) -> isize;
    fn y(&self) -> isize;
    fn vel_x(&self) -> isize;
    fn vel_y(&self) -> isize;

    fn hit_rect_size(&self) -> usize;
    fn offset_x(&self) -> isize { 0 }
    fn offset_y(&self) -> isize { 0 }

    fn hit_bounds(&self) -> &Rect<usize>;

    fn set_x(&mut self, x: isize);
    fn set_y(&mut self, y: isize);
    fn set_vel_x(&mut self, x: isize);
    fn set_vel_y(&mut self, y: isize);

    fn cond(&mut self) -> &mut Condition;
    fn flags(&mut self) -> &mut Flag;

    fn direction(&self) -> Direction;
    fn is_player(&self) -> bool;
    fn ignore_tile_44(&self) -> bool { true }
    fn player_left_pressed(&self) -> bool { false }
    fn player_right_pressed(&self) -> bool { false }

    fn judge_hit_block(&mut self, state: &mut SharedGameState, x: isize, y: isize) {
        let bounds_x = if self.is_player() { 5 } else { 8 };
        let bounds_y = if self.is_player() { 4 } else { 5 };
        // left wall
        if (self.y() - self.hit_bounds().top as isize) < (y * 16 + bounds_y) * 0x200
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 - bounds_y) * 0x200
            && (self.x() - self.hit_bounds().right as isize) < (x * 16 + 8) * 0x200
            && (self.x() - self.hit_bounds().right as isize) > x * 16 * 0x200 {
            self.set_x(((x * 16 + 8) * 0x200) + self.hit_bounds().right as isize);

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
        if (self.y() - self.hit_bounds().top as isize) < (y * 16 + bounds_y) * 0x200
            && self.y() + self.hit_bounds().bottom as isize > (y * 16 - bounds_y) * 0x200
            && (self.x() + self.hit_bounds().right as isize) > (x * 16 - 8) * 0x200
            && (self.x() + self.hit_bounds().right as isize) < x * 16 * 0x200 {
            self.set_x(((x * 16 - 8) * 0x200) - self.hit_bounds().right as isize);

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
        if (self.x() - self.hit_bounds().right as isize) < (x * 16 + bounds_x) * 0x200
            && (self.x() + self.hit_bounds().right as isize) > (x * 16 - bounds_x) * 0x200
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 + 8) * 0x200
            && (self.y() - self.hit_bounds().top as isize) > y * 16 * 0x200 {
            self.set_y(((y * 16 + 8) * 0x200) + self.hit_bounds().top as isize);

            if self.is_player() {
                if !self.cond().hidden() && self.vel_y() < -0x200 {
                    state.sound_manager.play_sfx(3);
                    state.create_caret(self.x(), self.y() - self.hit_bounds().top as isize, CaretType::LittleParticles, Direction::Left);
                    state.create_caret(self.x(), self.y() - self.hit_bounds().top as isize, CaretType::LittleParticles, Direction::Left);
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
        if ((self.x() - self.hit_bounds().right as isize) < (x * 16 + bounds_x) * 0x200)
            && ((self.x() + self.hit_bounds().right as isize) > (x * 16 - bounds_x) * 0x200)
            && ((self.y() + self.hit_bounds().bottom as isize) > ((y * 16 - 8) * 0x200))
            && ((self.y() + self.hit_bounds().bottom as isize) < (y * 16 * 0x200)) {
            self.set_y(((y * 16 - 8) * 0x200) - self.hit_bounds().bottom as isize);

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
    fn judge_hit_triangle_a(&mut self, state: &mut SharedGameState, x: isize, y: isize) {
        if self.x() < (x * 16 + 8) * 0x200
            && self.x() > (x * 16 - 8) * 0x200
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 * 0x200) - (self.x() - x * 16 * 0x200) / 2 + 0x800
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 - 8) * 0x200 {
            self.set_y((y * 16 * 0x200) - ((self.x() - x * 16 * 0x200) / 2) + 0x800 + self.hit_bounds().top as isize);

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                state.sound_manager.play_sfx(3);
                state.create_caret(self.x(), self.y() - self.hit_bounds().top as isize, CaretType::LittleParticles, Direction::Left);
                state.create_caret(self.x(), self.y() - self.hit_bounds().top as isize, CaretType::LittleParticles, Direction::Left);
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // upper left slope (smaller half)
    fn judge_hit_triangle_b(&mut self, state: &mut SharedGameState, x: isize, y: isize) {
        if self.x() < (x * 16 + 8) * 0x200
            && self.x() > (x * 16 - 8) * 0x200
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 * 0x200) - (self.x() - x * 16 * 0x200) / 2 - 0x800
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 - 8) * 0x200 {
            self.set_y((y * 16 * 0x200) - ((self.x() - x * 16 * 0x200) / 2) - 0x800 + self.hit_bounds().top as isize);

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                state.sound_manager.play_sfx(3);
                state.create_caret(self.x(), self.y() - self.hit_bounds().top as isize, CaretType::LittleParticles, Direction::Left);
                state.create_caret(self.x(), self.y() - self.hit_bounds().top as isize, CaretType::LittleParticles, Direction::Left);
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // upper right slope (smaller half)
    fn judge_hit_triangle_c(&mut self, state: &mut SharedGameState, x: isize, y: isize) {
        if self.x() < (x * 16 + 8) * 0x200
            && self.x() > (x * 16 - 8) * 0x200
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 * 0x200) + (self.x() - x * 16 * 0x200) / 2 - 0x800
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 - 8) * 0x200 {
            self.set_y((y * 16 * 0x200) + ((self.x() - x * 16 * 0x200) / 2) - 0x800 + self.hit_bounds().top as isize);

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                state.sound_manager.play_sfx(3);
                state.create_caret(self.x(), self.y() - self.hit_bounds().top as isize, CaretType::LittleParticles, Direction::Left);
                state.create_caret(self.x(), self.y() - self.hit_bounds().top as isize, CaretType::LittleParticles, Direction::Left);
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // upper right slope (bigger half)
    fn judge_hit_triangle_d(&mut self, state: &mut SharedGameState, x: isize, y: isize) {
        if (self.x() < (x * 16 + 8) * 0x200)
            && (self.x() > (x * 16 - 8) * 0x200)
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 * 0x200) + (self.x() - x * 16 * 0x200) / 2 + 0x800
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 - 8) * 0x200 {
            self.set_y((y * 16 * 0x200) + ((self.x() - x * 16 * 0x200) / 2) + 0x800 + self.hit_bounds().top as isize);

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                state.sound_manager.play_sfx(3);
                state.create_caret(self.x(), self.y() - self.hit_bounds().top as isize, CaretType::LittleParticles, Direction::Left);
                state.create_caret(self.x(), self.y() - self.hit_bounds().top as isize, CaretType::LittleParticles, Direction::Left);
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // lower left half (bigger)
    fn judge_hit_triangle_e(&mut self, state: &mut SharedGameState, x: isize, y: isize) {
        self.flags().set_hit_left_bigger_half(true);

        if (self.x() < (x * 16 + 8) * 0x200)
            && (self.x() > (x * 16 - 8) * 0x200)
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 * 0x200) + (self.x() - x * 16 * 0x200) / 2 - 0x800
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 + 8) * 0x200 {
            self.set_y((y * 16 * 0x200) + ((self.x() - x * 16 * 0x200) / 2) - 0x800 - self.hit_bounds().bottom as isize);

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
    fn judge_hit_triangle_f(&mut self, state: &mut SharedGameState, x: isize, y: isize) {
        self.flags().set_hit_left_smaller_half(true);

        if (self.x() < (x * 16 + 8) * 0x200)
            && (self.x() > (x * 16 - 8) * 0x200)
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 * 0x200) + (self.x() - x * 16 * 0x200) / 2 + 0x800
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 + 8) * 0x200 {
            self.set_y((y * 16 * 0x200) + ((self.x() - x * 16 * 0x200) / 2) + 0x800 - self.hit_bounds().bottom as isize);

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
    fn judge_hit_triangle_g(&mut self, state: &mut SharedGameState, x: isize, y: isize) {
        self.flags().set_hit_right_smaller_half(true);

        if (self.x() < (x * 16 + 8) * 0x200)
            && (self.x() > (x * 16 - 8) * 0x200)
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 * 0x200) - (self.x() - x * 16 * 0x200) / 2 + 0x800
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 + 8) * 0x200 {
            self.set_y((y * 16 * 0x200) - ((self.x() - x * 16 * 0x200) / 2) + 0x800 - self.hit_bounds().bottom as isize);

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
    fn judge_hit_triangle_h(&mut self, state: &mut SharedGameState, x: isize, y: isize) {
        self.flags().set_hit_right_bigger_half(true);

        if (self.x() < (x * 16 + 8) * 0x200)
            && (self.x() > (x * 16 - 8) * 0x200)
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 * 0x200) - (self.x() - x * 16 * 0x200) / 2 - 0x800
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 + 8) * 0x200 {
            self.set_y((y * 16 * 0x200) - ((self.x() - x * 16 * 0x200) / 2) - 0x800 - self.hit_bounds().bottom as isize);

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

    fn judge_hit_water(&mut self, x: isize, y: isize) {
        let bounds_x = if self.is_player() { 5 } else { 6 };
        let bounds_up = if self.is_player() { 5 } else { 6 };
        let bounds_down = if self.is_player() { 0 } else { 6 };
        if (self.x() - self.hit_bounds().right as isize) < (x * 16 + bounds_x) * 0x200
            && (self.x() + self.hit_bounds().right as isize) > (x * 16 - bounds_x) * 0x200
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 + bounds_up) * 0x200
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 - bounds_down) * 0x200 {
            self.flags().set_in_water(true);
        }
    }

    fn judge_hit_spike(&mut self, x: isize, y: isize, water: bool) {
        if (self.x() - 0x800) < (x * 16 + 4) * 0x200
            && (self.x() + 0x800) > (x * 16 - 4) * 0x200
            && (self.y() - 0x800) < (y * 16 + 3) * 0x200
            && (self.y() + 0x800) > (y * 16 - 3) * 0x200 {
            self.flags().set_hit_by_spike(true);
            if water {
                self.flags().set_in_water(true);
            }
        }
    }

    fn judge_hit_force(&mut self, x: isize, y: isize, direction: Direction, water: bool) {
        if (self.x() - self.hit_bounds().left as isize) < (x * 16 + 6) * 0x200
            && (self.x() + self.hit_bounds().right as isize) > (x * 16 - 6) * 0x200
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 + 6) * 0x200
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 - 6) * 0x200 {
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

    fn tick_map_collisions(&mut self, state: &mut SharedGameState, stage: &mut Stage) {
        let hit_rect_size = clamp(self.hit_rect_size(), 1, 4);
        let hit_rect_size = hit_rect_size * hit_rect_size;

        let x = (self.x() + self.offset_x()) / (16 * 0x200);
        let y = (self.y() + self.offset_y()) / (16 * 0x200);

        self.flags().0 = 0;
        for (idx, (&ox, &oy)) in OFF_X.iter().zip(OFF_Y.iter()).enumerate() {
            if idx == hit_rect_size {
                break;
            }

            let attrib = stage.map.get_attribute((x + ox) as usize, (y + oy) as usize);
            match attrib {
                // Spikes
                0x62 | 0x42 if self.is_player() => {
                    self.judge_hit_spike(x + ox, y + oy, attrib & 0x20 != 0);
                }

                // Blocks
                0x02 | 0x60 => {
                    self.judge_hit_water(x + ox, y + oy);
                }
                0x62 if !self.is_player() => {
                    self.judge_hit_water(x + ox, y + oy);
                }
                0x61 => {
                    self.judge_hit_block(state, x + ox, y + oy);
                    self.judge_hit_water(x + ox, y + oy);
                }
                0x04 | 0x64 if !self.is_player() => {
                    self.judge_hit_block(state, x + ox, y + oy);
                    self.judge_hit_water(x + ox, y + oy);
                }
                0x05 | 0x41 | 0x43 | 0x46 if self.is_player() => {
                    self.judge_hit_block(state, x + ox, y + oy);
                }
                0x03 | 0x05 | 0x41 | 0x43 if !self.is_player() => {
                    self.judge_hit_block(state, x + ox, y + oy);
                }
                0x44 => {
                    if !self.ignore_tile_44() {
                        self.judge_hit_block(state, x + ox, y + oy);
                    }
                }

                // Slopes
                0x50 | 0x70 => {
                    self.judge_hit_triangle_a(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x51 | 0x71 => {
                    self.judge_hit_triangle_b(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x52 | 0x72 => {
                    self.judge_hit_triangle_c(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x53 | 0x73 => {
                    self.judge_hit_triangle_d(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x54 | 0x74 => {
                    self.judge_hit_triangle_e(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x55 | 0x75 => {
                    self.judge_hit_triangle_f(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x56 | 0x76 => {
                    self.judge_hit_triangle_g(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x57 | 0x77 => {
                    self.judge_hit_triangle_h(state, x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }

                // Forces
                0x80 | 0xa0 if self.is_player() => {
                    self.judge_hit_force(x + ox, y + oy, Direction::Left, attrib & 0x20 != 0);
                }
                0x81 | 0xa1 if self.is_player() => {
                    self.judge_hit_force(x + ox, y + oy, Direction::Up, attrib & 0x20 != 0);
                }
                0x82 | 0xa2 if self.is_player() => {
                    self.judge_hit_force(x + ox, y + oy, Direction::Right, attrib & 0x20 != 0);
                }
                0x83 | 0xa3 if self.is_player() => {
                    self.judge_hit_force(x + ox, y + oy, Direction::Bottom, attrib & 0x20 != 0);
                }
                0x80 | 0xa0 if !self.is_player() => {
                    self.flags().set_force_left(true);
                    if attrib & 0x20 != 0 { self.flags().set_in_water(true); }
                }
                0x81 | 0xa1 if !self.is_player() => {
                    self.flags().set_force_up(true);
                    if attrib & 0x20 != 0 { self.flags().set_in_water(true); }
                }
                0x82 | 0xa2 if !self.is_player() => {
                    self.flags().set_force_right(true);
                    if attrib & 0x20 != 0 { self.flags().set_in_water(true); }
                }
                0x83 | 0xa3 if !self.is_player() => {
                    self.flags().set_force_down(true);
                    if attrib & 0x20 != 0 { self.flags().set_in_water(true); }
                }
                _ => {}
            }
        }
    }
}
