use nalgebra::clamp;

use crate::common::Direction;
use crate::ggez::GameResult;
use crate::npc::NPC;
use crate::SharedGameState;

impl NPC {
    pub(crate) fn tick_n001_experience(&mut self, state: &mut SharedGameState) -> GameResult {
        if state.control_flags.wind() {
            if self.action_num == 0 {
                self.action_num = 1;

                self.vel_x = state.game_rng.range(-0x80..0x80) as isize;
                self.vel_y = state.game_rng.range(-0x7f..0x100) as isize;
            }

            self.vel_x -= 0x8;

            if self.x < 80 * 0x200 {
                self.cond.set_alive(false);
                return Ok(());
            }

            if self.vel_x < -0x600 {
                self.vel_x = -0x600;
            }

            if self.flags.hit_left_wall() {
                self.vel_x = 0x100;
            }

            if self.flags.hit_top_wall() {
                self.vel_y = 0x40;
            }

            if self.flags.hit_bottom_wall() {
                self.vel_y = -0x40;
            }
        } else {
            if self.action_num == 0 {
                self.action_num = 1;
                self.anim_num = state.game_rng.range(0..4) as u16;

                self.vel_x = state.game_rng.range(-0x200..0x200) as isize;
                self.vel_y = state.game_rng.range(-0x400..0) as isize;

                self.direction = if state.game_rng.range(0..1) != 0 {
                    Direction::Left
                } else {
                    Direction::Right
                };
            }

            self.vel_y += if self.flags.in_water() {
                0x15
            } else {
                0x2a
            };

            if self.flags.hit_left_wall() && self.vel_x < 0 {
                self.vel_x = -self.vel_x;
            }

            if self.flags.hit_right_wall() && self.vel_x > 0 {
                self.vel_x = -self.vel_x;
            }

            if self.flags.hit_top_wall() && self.vel_y < 0 {
                self.vel_y = -self.vel_y;
            }

            if self.flags.hit_bottom_wall() {
                // todo play sound 45

                self.vel_y = -0x280;
                self.vel_x = 2 * self.vel_x / 3;
            }

            if self.flags.hit_left_wall() || self.flags.hit_right_wall() || self.flags.hit_bottom_wall() {
                // todo play sound 45
                self.action_counter2 += 1;

                if self.action_counter2 > 2 {
                    self.vel_y -= 0x200;
                }
            } else {
                self.action_counter2 = 0;
            }

            self.vel_x = clamp(self.vel_x, -0x5ff, 0x5ff);
            self.vel_y = clamp(self.vel_y, -0x5ff, 0x5ff);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_counter += 1;

        if self.direction == Direction::Left {
            if self.anim_counter > 2 {
                self.anim_counter = 0;

                self.anim_num += 1;
                if self.anim_num > 5 {
                    self.anim_num = 0;
                }
            }
        } else if self.anim_counter > 2 {
            self.anim_counter = 0;

            if self.anim_num > 0 {
                self.anim_num -= 1;
            } else {
                self.anim_num = 5;
            }
        }

        self.anim_rect = state.constants.npc.n001_experience[self.anim_num as usize];

        if self.action_num != 0 {
            if self.exp >= 5 {
                self.anim_rect.top += 16;
                self.anim_rect.bottom += 16;
            } else if self.exp >= 20 {
                self.anim_rect.top += 32;
                self.anim_rect.bottom += 32;
            }
        }

        self.action_counter += 1;
        if self.action_counter > 500 && self.anim_num == 5 && self.anim_counter == 2 {
            self.cond.set_alive(false);
            return Ok(());
        }

        if self.action_counter > 400 && (self.action_counter / 2 % 2) != 0 {
            self.anim_rect.left = 0;
            self.anim_rect.top = 0;
            self.anim_rect.right = 0;
            self.anim_rect.bottom = 0;
        }

        Ok(())
    }
}
