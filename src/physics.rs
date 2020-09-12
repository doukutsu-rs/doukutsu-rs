use num_traits::clamp;

use crate::common::{Condition, Flag, Rect};
use crate::SharedGameState;
use crate::stage::Stage;

pub const OFF_X: [isize; 9] = [0, 1, 0, 1, 2, 2, 2, 0, 1];
pub const OFF_Y: [isize; 9] = [0, 0, 1, 1, 0, 1, 2, 2, 2];

pub trait PhysicalEntity {
    fn x(&self) -> isize;
    fn y(&self) -> isize;
    fn vel_x(&self) -> isize;
    fn vel_y(&self) -> isize;

    fn size(&self) -> u8;

    fn hit_bounds(&self) -> &Rect<usize>;

    fn set_x(&mut self, x: isize);
    fn set_y(&mut self, y: isize);
    fn set_vel_x(&mut self, x: isize);
    fn set_vel_y(&mut self, y: isize);

    fn cond(&mut self) -> &mut Condition;
    fn flags(&mut self) -> &mut Flag;

    fn is_player(&self) -> bool;
    fn ignore_tile_44(&self) -> bool { true }

    fn judge_hit_block(&mut self, state: &SharedGameState, x: isize, y: isize) {
        // left wall
        if (self.y() - self.hit_bounds().top as isize) < (y * 16 + 4) * 0x200
            && self.y() + self.hit_bounds().bottom as isize > (y * 16 - 4) * 0x200
            && (self.x() - self.hit_bounds().right as isize) < (x * 16 + 8) * 0x200
            && (self.x() - self.hit_bounds().right as isize) > x * 16 * 0x200 {
            self.set_x(((x * 16 + 8) * 0x200) + self.hit_bounds().right as isize);

            if self.is_player() {
                if self.vel_x() < -0x180 {
                    self.set_vel_x(-0x180);
                }

                if !state.key_state.left() && self.vel_x() < 0 {
                    self.set_vel_x(0);
                }
            }

            self.flags().set_hit_left_wall(true);
        }

        // right wall
        if (self.y() - self.hit_bounds().top as isize) < (y * 16 + 4) * 0x200
            && self.y() + self.hit_bounds().bottom as isize > (y * 16 - 4) * 0x200
            && (self.x() + self.hit_bounds().right as isize) > (x * 16 - 8) * 0x200
            && (self.x() + self.hit_bounds().right as isize) < x * 16 * 0x200 {
            self.set_x(((x * 16 - 8) * 0x200) - self.hit_bounds().right as isize);

            if self.is_player() {
                if self.vel_x() > 0x180 {
                    self.set_vel_x(0x180);
                }

                if !state.key_state.right() && self.vel_x() > 0 {
                    self.set_vel_x(0);
                }
            }

            self.flags().set_hit_right_wall(true);
        }

        // ceiling
        if (self.x() - self.hit_bounds().right as isize) < (x * 16 + 5) * 0x200
            && (self.x() + self.hit_bounds().right as isize) > (x * 16 - 5) * 0x200
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 + 8) * 0x200
            && (self.y() - self.hit_bounds().top as isize) > y * 16 * 0x200 {
            self.set_y(((y * 16 + 8) * 0x200) + self.hit_bounds().top as isize);

            if self.is_player() {
                if !self.cond().hidden() && self.vel_y() < -0x200 {
                    self.flags().set_head_bounced(true);
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
        if ((self.x() - self.hit_bounds().right as isize) < (x * 16 + 5) * 0x200)
            && ((self.x() + self.hit_bounds().right as isize) > (x * 16 - 5) * 0x200)
            && ((self.y() + self.hit_bounds().bottom as isize) > (y * 16 - 8) * 0x200)
            && ((self.y() + self.hit_bounds().bottom as isize) < y * 16 * 0x200) {
            self.set_y(((y * 16 - 8) * 0x200) - self.hit_bounds().bottom as isize);

            if self.is_player() {
                if self.vel_y() > 0x400 {
                    // PlaySoundObject(23, SOUND_MODE_PLAY); todo
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
    fn judge_hit_triangle_a(&mut self, x: isize, y: isize) {
        if self.x() < (x * 16 + 8) * 0x200
            && self.x() > (x * 16 - 8) * 0x200
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 * 0x200) - (self.x() - x * 16 * 0x200) / 2 + 0x800
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 - 8) * 0x200 {
            self.set_y((y * 16 * 0x200) - ((self.x() - x * 16 * 0x200) / 2) + 0x800 + self.hit_bounds().top as isize);

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                self.flags().set_head_bounced(true);
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // upper left slope (smaller half)
    fn judge_hit_triangle_b(&mut self, x: isize, y: isize) {
        if self.x() < (x * 16 + 8) * 0x200
            && self.x() > (x * 16 - 8) * 0x200
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 * 0x200) - (self.x() - x * 16 * 0x200) / 2 - 0x800
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 - 8) * 0x200 {
            self.set_y((y * 16 * 0x200) - ((self.x() - x * 16 * 0x200) / 2) - 0x800 + self.hit_bounds().top as isize);

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                self.flags().set_head_bounced(true);
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // upper right slope (smaller half)
    fn judge_hit_triangle_c(&mut self, x: isize, y: isize) {
        if self.x() < (x * 16 + 8) * 0x200
            && self.x() > (x * 16 - 8) * 0x200
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 * 0x200) + (self.x() - x * 16 * 0x200) / 2 - 0x800
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 - 8) * 0x200 {
            self.set_y((y * 16 * 0x200) + ((self.x() - x * 16 * 0x200) / 2) - 0x800 + self.hit_bounds().top as isize);

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                self.flags().set_head_bounced(true);
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // upper right slope (bigger half)
    fn judge_hit_triangle_d(&mut self, x: isize, y: isize) {
        if (self.x() < (x * 16 + 8) * 0x200)
            && (self.x() > (x * 16 - 8) * 0x200)
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 * 0x200) + (self.x() - x * 16 * 0x200) / 2 + 0x800
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 - 8) * 0x200 {
            self.set_y((y * 16 * 0x200) + ((self.x() - x * 16 * 0x200) / 2) + 0x800 + self.hit_bounds().top as isize);

            if self.is_player() && !self.cond().hidden() && self.vel_y() < -0x200 {
                self.flags().set_head_bounced(true);
            }

            if self.vel_y() < 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_top_wall(true);
        }
    }

    // lower left half (bigger)
    fn judge_hit_triangle_e(&mut self, x: isize, y: isize) {
        self.flags().set_hit_left_bigger_half(true);

        if (self.x() < (x * 16 + 8) * 0x200)
            && (self.x() > (x * 16 - 8) * 0x200)
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 * 0x200) + (self.x() - x * 16 * 0x200) / 2 - 0x800
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 + 8) * 0x200 {
            self.set_y((y * 16 * 0x200) + ((self.x() - x * 16 * 0x200) / 2) - 0x800 - self.hit_bounds().bottom as isize);

            if self.is_player() && self.vel_y() > 0x400 {
                // PlaySoundObject(23, SOUND_MODE_PLAY); todo
            }

            if self.vel_y() > 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_left_slope(true);
            self.flags().set_hit_bottom_wall(true);
        }
    }

    // lower left half (smaller)
    fn judge_hit_triangle_f(&mut self, x: isize, y: isize) {
        self.flags().set_hit_left_smaller_half(true);

        if (self.x() < (x * 16 + 8) * 0x200)
            && (self.x() > (x * 16 - 8) * 0x200)
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 * 0x200) + (self.x() - x * 16 * 0x200) / 2 + 0x800
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 + 8) * 0x200 {
            self.set_y((y * 16 * 0x200) + ((self.x() - x * 16 * 0x200) / 2) + 0x800 - self.hit_bounds().bottom as isize);

            if self.is_player() && self.vel_y() > 0x400 {
                // PlaySoundObject(23, SOUND_MODE_PLAY); todo
            }

            if self.vel_y() > 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_left_slope(true);
            self.flags().set_hit_bottom_wall(true);
        }
    }

    // lower right half (smaller)
    fn judge_hit_triangle_g(&mut self, x: isize, y: isize) {
        self.flags().set_hit_right_smaller_half(true);

        if (self.x() < (x * 16 + 8) * 0x200)
            && (self.x() > (x * 16 - 8) * 0x200)
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 * 0x200) - (self.x() - x * 16 * 0x200) / 2 + 0x800
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 + 8) * 0x200 {
            self.set_y((y * 16 * 0x200) - ((self.x() - x * 16 * 0x200) / 2) + 0x800 - self.hit_bounds().bottom as isize);

            if self.is_player() && self.vel_y() > 0x400 {
                // PlaySoundObject(23, SOUND_MODE_PLAY); todo
            }

            if self.vel_y() > 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_right_slope(true);
            self.flags().set_hit_bottom_wall(true);
        }
    }

    // lower right half (bigger)
    fn judge_hit_triangle_h(&mut self, x: isize, y: isize) {
        self.flags().set_hit_right_bigger_half(true);

        if (self.x() < (x * 16 + 8) * 0x200)
            && (self.x() > (x * 16 - 8) * 0x200)
            && (self.y() + self.hit_bounds().bottom as isize) > (y * 16 * 0x200) - (self.x() - x * 16 * 0x200) / 2 - 0x800
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 + 8) * 0x200 {
            self.set_y((y * 16 * 0x200) - ((self.x() - x * 16 * 0x200) / 2) - 0x800 - self.hit_bounds().bottom as isize);

            if self.is_player() && self.vel_y() > 0x400 {
                // PlaySoundObject(23, SOUND_MODE_PLAY); todo
            }

            if self.vel_y() > 0 {
                self.set_vel_y(0);
            }

            self.flags().set_hit_right_slope(true);
            self.flags().set_hit_bottom_wall(true);
        }
    }

    fn judge_hit_water(&mut self, x: isize, y: isize) {
        if (self.x() - self.hit_bounds().right as isize) < (x * 16 + 5) * 0x200
            && (self.x() + self.hit_bounds().right as isize) > (x * 16 - 5) * 0x200
            && (self.y() - self.hit_bounds().top as isize) < (y * 16 + 5) * 0x200
            && (self.y() + self.hit_bounds().bottom as isize) > y * 16 * 0x200 {
            self.flags().set_in_water(true);
        }
    }

    fn judge_hit_spike(&mut self, x: isize, y: isize) {
        if (self.x() - 0x800) < (x * 16 + 4) * 0x200
            && (self.x() + 0x800) > (x * 16 - 4) * 0x200
            && (self.y() - 0x800) < (y * 16 + 3) * 0x200
            && (self.y() + 0x800) > (y * 16 - 3) * 0x200 {
            self.flags().set_hit_by_spike(true);
        }
    }

    fn tick_map_collisions(&mut self, state: &mut SharedGameState, stage: &mut Stage) {
        let big = self.size() >= 3;
        let x = clamp((self.x() - if big { 0x1000 } else { 0 }) / 16 / 0x200, 0, stage.map.width as isize);
        let y = clamp((self.y() - if big { 0x1000 } else { 0 }) / 16 / 0x200, 0, stage.map.height as isize);

        for (idx, (&ox, &oy)) in OFF_X.iter().zip(OFF_Y.iter()).enumerate() {
            if idx == 4 && big {
                break;
            }

            let attrib = stage.map.get_attribute((x + ox) as usize, (y + oy) as usize);
            match attrib {
                // Spikes
                0x62 | 0x42 if self.is_player() => {
                    if attrib & 0x20 != 0 { self.flags().set_in_water(true); }
                    self.judge_hit_spike(x + ox, y + ox);
                }

                // Blocks
                0x02 | 0x60 | 0x62 => {
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
                    self.judge_hit_triangle_a(x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x51 | 0x71 => {
                    self.judge_hit_triangle_b(x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x52 | 0x72 => {
                    self.judge_hit_triangle_c(x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x53 | 0x73 => {
                    self.judge_hit_triangle_d(x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x54 | 0x74 => {
                    self.judge_hit_triangle_e(x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x55 | 0x75 => {
                    self.judge_hit_triangle_f(x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x56 | 0x76 => {
                    self.judge_hit_triangle_g(x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x57 | 0x77 => {
                    self.judge_hit_triangle_h(x + ox, y + oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(x + ox, y + oy); }
                }
                0x61 => {
                    self.judge_hit_water(x + ox, y + oy);
                    self.judge_hit_block(state, x + ox, y + oy);
                }
                0x04 | 0x64 if !self.is_player() => {
                    self.judge_hit_water(x + ox, y + oy);
                    self.judge_hit_block(state, x + ox, y + oy);
                }

                // Forces
                0x80 | 0xa0 => {
                    if attrib & 0x20 != 0 { self.flags().set_in_water(true); }
                    self.flags().set_force_left(true);
                }
                0x81 | 0xa1 => {
                    if attrib & 0x20 != 0 { self.flags().set_in_water(true); }
                    self.flags().set_force_up(true);
                }
                0x82 | 0xa2 => {
                    if attrib & 0x20 != 0 { self.flags().set_in_water(true); }
                    self.flags().set_force_right(true);
                }
                0x83 | 0xa3 => {
                    if attrib & 0x20 != 0 { self.flags().set_in_water(true); }
                    self.flags().set_force_down(true);
                }
                _ => {}
            }
        }
    }
}
