use num_traits::clamp;

use crate::player::Player;
use crate::stage::Stage;
use crate::SharedGameState;
use crate::caret::CaretType;
use crate::common::Direction;

const OFF_X: &[isize; 4] = &[0, 1, 0, 1];
const OFF_Y: &[isize; 4] = &[0, 0, 1, 1];

impl Player {
    fn judge_hit_block(&mut self, state: &SharedGameState, x: isize, y: isize) {
        // left wall
        if (self.y - self.hit.top as isize) < (y * 0x10 + 4) * 0x200
            && self.y + self.hit.bottom as isize > (y * 0x10 - 4) * 0x200
            && (self.x - self.hit.right as isize) < (x * 0x10 + 8) * 0x200
            && (self.x - self.hit.right as isize) > x * 0x10 * 0x200 {
            self.x = ((x * 0x10 + 8) * 0x200) + self.hit.right as isize;

            if self.vel_x < -0x180 {
                self.vel_x = -0x180;
            }

            if !state.key_state.left() && self.vel_x < 0 {
                self.vel_x = 0;
            }

            self.flags.set_hit_left_wall(true);
        }

        // right wall
        if (self.y - self.hit.top as isize) < (y * 0x10 + 4) * 0x200
            && self.y + self.hit.bottom as isize > (y * 0x10 - 4) * 0x200
            && (self.x + self.hit.right as isize) > (x * 0x10 - 8) * 0x200
            && (self.x + self.hit.right as isize) < x * 0x10 * 0x200 {
            self.x = ((x * 0x10 - 8) * 0x200) - self.hit.right as isize;

            if self.vel_x > 0x180 {
                self.vel_x = 0x180;
            }

            if !state.key_state.right() && self.vel_x > 0 {
                self.vel_x = 0;
            }

            self.flags.set_hit_right_wall(true);
        }

        // ceiling
        if (self.x - self.hit.right as isize) < (x * 0x10 + 5) * 0x200
            && (self.x + self.hit.right as isize) > (x * 0x10 - 5) * 0x200
            && (self.y - self.hit.top as isize) < (y * 0x10 + 8) * 0x200
            && (self.y - self.hit.top as isize) > y * 0x10 * 0x200 {
            self.y = ((y * 0x10 + 8) * 0x200) + self.hit.top as isize;

            if !self.cond.hidden() && self.vel_y < -0x200 {
                self.flags.set_head_bounced(true);
            }

            if self.vel_y < 0 {
                self.vel_y = 0;
            }

            self.flags.set_hit_top_wall(true);
        }

        // floor
        if ((self.x - self.hit.right as isize) < (x * 0x10 + 5) * 0x200)
            && ((self.x + self.hit.right as isize) > (x * 0x10 - 5) * 0x200)
            && ((self.y + self.hit.bottom as isize) > (y * 0x10 - 8) * 0x200)
            && ((self.y + self.hit.bottom as isize) < y * 0x10 * 0x200) {
            self.y = ((y * 0x10 - 8) * 0x200) - self.hit.bottom as isize;

            if self.vel_y > 0x400 {
                // PlaySoundObject(23, SOUND_MODE_PLAY); todo
            }

            if self.vel_y > 0 {
                self.vel_y = 0;
            }

            self.flags.set_hit_bottom_wall(true);
        }
    }

    // upper left slope (bigger half)
    fn judge_hit_triangle_a(&mut self, state: &SharedGameState, x: isize, y: isize) {
        if self.x < (x * 0x10 + 8) * 0x200
            && self.x > (x * 0x10 - 8) * 0x200
            && (self.y - self.hit.top as isize) < (y * 0x10 * 0x200) - (self.x - x * 0x10 * 0x200) / 2 + 0x800
            && (self.y + self.hit.bottom as isize) > (y * 0x10 - 8) * 0x200 {
            self.y = (y * 0x10 * 0x200) - ((self.x - x * 0x10 * 0x200) / 2) + 0x800 + self.hit.top as isize;

            if !self.cond.hidden() && self.vel_y < -0x200 {
                self.flags.set_head_bounced(true);
            }

            if self.vel_y < 0 {
                self.vel_y = 0;
            }

            self.flags.set_hit_top_wall(true);
        }
    }

    // upper left slope (smaller half)
    fn judge_hit_triangle_b(&mut self, state: &SharedGameState, x: isize, y: isize) {
        if self.x < (x * 0x10 + 8) * 0x200
            && self.x > (x * 0x10 - 8) * 0x200
            && (self.y - self.hit.top as isize) < (y * 0x10 * 0x200) - (self.x - x * 0x10 * 0x200) / 2 - 0x800
            && (self.y + self.hit.bottom as isize) > (y * 0x10 - 8) * 0x200 {
            self.y = (y * 0x10 * 0x200) - ((self.x - x * 0x10 * 0x200) / 2) - 0x800 + self.hit.top as isize;

            if !self.cond.hidden() && self.vel_y < -0x200 {
                self.flags.set_head_bounced(true);
            }

            if self.vel_y < 0 {
                self.vel_y = 0;
            }

            self.flags.set_hit_top_wall(true);
        }
    }

    // upper right slope (smaller half)
    fn judge_hit_triangle_c(&mut self, state: &SharedGameState, x: isize, y: isize) {
        if self.x < (x * 0x10 + 8) * 0x200
            && self.x > (x * 0x10 - 8) * 0x200
            && (self.y - self.hit.top as isize) < (y * 0x10 * 0x200) + (self.x - x * 0x10 * 0x200) / 2 - 0x800
            && (self.y + self.hit.bottom as isize) > (y * 0x10 - 8) * 0x200 {
            self.y = (y * 0x10 * 0x200) + ((self.x - x * 0x10 * 0x200) / 2) - 0x800 + self.hit.top as isize;

            if !self.cond.hidden() && self.vel_y < -0x200 {
                self.flags.set_head_bounced(true);
            }

            if self.vel_y < 0 {
                self.vel_y = 0;
            }

            self.flags.set_hit_top_wall(true);
        }
    }

    // upper right slope (bigger half)
    fn judge_hit_triangle_d(&mut self, state: &SharedGameState, x: isize, y: isize) {
        if (self.x < (x * 0x10 + 8) * 0x200)
            && (self.x > (x * 0x10 - 8) * 0x200)
            && (self.y - self.hit.top as isize) < (y * 0x10 * 0x200) + (self.x - x * 0x10 * 0x200) / 2 + 0x800
            && (self.y + self.hit.bottom as isize) > (y * 0x10 - 8) * 0x200 {
            self.y = (y * 0x10 * 0x200) + ((self.x - x * 0x10 * 0x200) / 2) + 0x800 + self.hit.top as isize;

            if !self.cond.hidden() && self.vel_y < -0x200 {
                self.flags.set_head_bounced(true);
            }

            if self.vel_y < 0 {
                self.vel_y = 0;
            }

            self.flags.set_hit_top_wall(true);
        }
    }

    // lower left half (bigger)
    fn judge_hit_triangle_e(&mut self, state: &SharedGameState, x: isize, y: isize) {
        self.flags.set_hit_left_bigger_half(true);

        if (self.x < (x * 0x10 + 8) * 0x200)
            && (self.x > (x * 0x10 - 8) * 0x200)
            && (self.y + self.hit.bottom as isize) > (y * 0x10 * 0x200) + (self.x - x * 0x10 * 0x200) / 2 - 0x800
            && (self.y - self.hit.top as isize) < (y * 0x10 + 8) * 0x200 {
            self.y = (y * 0x10 * 0x200) + ((self.x - x * 0x10 * 0x200) / 2) - 0x800 - self.hit.bottom as isize;

            if self.vel_y > 0x400 {
                // PlaySoundObject(23, SOUND_MODE_PLAY); todo
            }

            if self.vel_y > 0 {
                self.vel_y = 0;
            }

            self.flags.set_hit_left_slope(true);
            self.flags.set_hit_bottom_wall(true);
        }
    }

    // lower left half (smaller)
    fn judge_hit_triangle_f(&mut self, state: &SharedGameState, x: isize, y: isize) {
        self.flags.set_hit_left_smaller_half(true);

        if (self.x < (x * 0x10 + 8) * 0x200)
            && (self.x > (x * 0x10 - 8) * 0x200)
            && (self.y + self.hit.bottom as isize) > (y * 0x10 * 0x200) + (self.x - x * 0x10 * 0x200) / 2 + 0x800
            && (self.y - self.hit.top as isize) < (y * 0x10 + 8) * 0x200 {
            self.y = (y * 0x10 * 0x200) + ((self.x - x * 0x10 * 0x200) / 2) + 0x800 - self.hit.bottom as isize;

            if self.vel_y > 0x400 {
                // PlaySoundObject(23, SOUND_MODE_PLAY); todo
            }

            if self.vel_y > 0 {
                self.vel_y = 0;
            }

            self.flags.set_hit_left_slope(true);
            self.flags.set_hit_bottom_wall(true);
        }
    }

    // lower right half (smaller)
    fn judge_hit_triangle_g(&mut self, state: &SharedGameState, x: isize, y: isize) {
        self.flags.set_hit_right_smaller_half(true);

        if (self.x < (x * 0x10 + 8) * 0x200)
            && (self.x > (x * 0x10 - 8) * 0x200)
            && (self.y + self.hit.bottom as isize) > (y * 0x10 * 0x200) - (self.x - x * 0x10 * 0x200) / 2 + 0x800
            && (self.y - self.hit.top as isize) < (y * 0x10 + 8) * 0x200 {
            self.y = (y * 0x10 * 0x200) - ((self.x - x * 0x10 * 0x200) / 2) + 0x800 - self.hit.bottom as isize;

            if self.vel_y > 0x400 {
                // PlaySoundObject(23, SOUND_MODE_PLAY); todo
            }

            if self.vel_y > 0 {
                self.vel_y = 0;
            }

            self.flags.set_hit_right_slope(true);
            self.flags.set_hit_bottom_wall(true);
        }
    }

    // lower right half (bigger)
    fn judge_hit_triangle_h(&mut self, state: &SharedGameState, x: isize, y: isize) {
        self.flags.set_hit_right_bigger_half(true);

        if (self.x < (x * 0x10 + 8) * 0x200)
            && (self.x > (x * 0x10 - 8) * 0x200)
            && (self.y + self.hit.bottom as isize) > (y * 0x10 * 0x200) - (self.x - x * 0x10 * 0x200) / 2 - 0x800
            && (self.y - self.hit.top as isize) < (y * 0x10 + 8) * 0x200 {
            self.y = (y * 0x10 * 0x200) - ((self.x - x * 0x10 * 0x200) / 2) - 0x800 - self.hit.bottom as isize;

            if self.vel_y > 0x400 {
                // PlaySoundObject(23, SOUND_MODE_PLAY); todo
            }

            if self.vel_y > 0 {
                self.vel_y = 0;
            }

            self.flags.set_hit_right_slope(true);
            self.flags.set_hit_bottom_wall(true);
        }
    }

    fn judge_hit_water(&mut self, state: &SharedGameState, x: isize, y: isize) {
        if (self.x - self.hit.right as isize) < (x * 0x10 + 5) * 0x200
            && (self.x + self.hit.right as isize) > (x * 0x10 - 5) * 0x200
            && (self.y - self.hit.top as isize) < (y * 0x10 + 5) * 0x200
            && (self.y + self.hit.bottom as isize) > y * 0x10 * 0x200 {
            self.flags.set_in_water(true);
        }
    }

    pub fn tick_map_collisions(&mut self, state: &SharedGameState, stage: &Stage) {
        let x = clamp(self.x / 0x10 / 0x200, 0, stage.map.width as isize);
        let y = clamp(self.y / 0x10 / 0x200, 0, stage.map.height as isize);

        for (ox, oy) in OFF_X.iter().zip(OFF_Y) {
            let attrib = stage.map.get_attribute((x + *ox) as usize, (y + *oy) as usize);
            match attrib {
                // Block
                0x02 | 0x60 => {
                    self.judge_hit_water(state, x + *ox, y + *oy);
                }
                0x05 | 0x41 | 0x43 | 0x46 => {
                    self.judge_hit_block(state, x + *ox, y + *oy);
                }
                0x50 | 0x70 => {
                    self.judge_hit_triangle_a(state, x + *ox, y + *oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(state, x + *ox, y + *oy); }
                }
                0x51 | 0x71 => {
                    self.judge_hit_triangle_b(state, x + *ox, y + *oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(state, x + *ox, y + *oy); }
                }
                0x52 | 0x72 => {
                    self.judge_hit_triangle_c(state, x + *ox, y + *oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(state, x + *ox, y + *oy); }
                }
                0x53 | 0x73 => {
                    self.judge_hit_triangle_d(state, x + *ox, y + *oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(state, x + *ox, y + *oy); }
                }
                0x54 | 0x74 => {
                    self.judge_hit_triangle_e(state, x + *ox, y + *oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(state, x + *ox, y + *oy); }
                }
                0x55 | 0x75 => {
                    self.judge_hit_triangle_f(state, x + *ox, y + *oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(state, x + *ox, y + *oy); }
                }
                0x56 | 0x76 => {
                    self.judge_hit_triangle_g(state, x + *ox, y + *oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(state, x + *ox, y + *oy); }
                }
                0x57 | 0x77 => {
                    self.judge_hit_triangle_h(state, x + *ox, y + *oy);
                    if attrib & 0x20 != 0 { self.judge_hit_water(state, x + *ox, y + *oy); }
                }
                0x61 => {
                    self.judge_hit_water(state, x + *ox, y + *oy);
                    self.judge_hit_block(state, x + *ox, y + *oy);
                }
                _ => {}
            }
        }
    }

    pub fn tick_npc_collisions(&mut self, state: &mut SharedGameState, stage: &Stage) {
        if self.question {
            state.create_caret(self.x, self.y, CaretType::QuestionMark, Direction::Left);
        }
    }
}
