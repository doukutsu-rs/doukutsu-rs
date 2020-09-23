use std::cell::RefCell;
use std::collections::HashMap;

use num_traits::clamp;
use num_traits::real::Real;

use crate::common::Direction;
use crate::ggez::GameResult;
use crate::npc::NPC;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n068_balrog_running(&mut self, state: &mut SharedGameState, player: &mut Player) -> GameResult {
        println!("x {} y {} vx {} vy {}", self.x, self.y, self.vel_x, self.vel_y);
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.action_counter = 30;

                    if self.x > player.x {
                        self.direction = Direction::Left;
                    } else {
                        self.direction = Direction::Right;
                    }
                }

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 2;
                    self.action_counter2 += 1;
                }
            }
            2 | 3 => {
                if self.action_num == 2 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num == 2 || self.anim_num == 4 {
                        state.sound_manager.play_sfx(23);
                    }

                    if self.anim_num > 4 {
                        self.anim_num = 1;
                    }
                }

                self.vel_x += 0x10 * self.direction.vector_x(); // 0.03125fix9

                if self.action_counter >= 8 && (player.x - self.x).abs() < 12 * 0x200
                    && self.y - 12 * 0x200 < player.y && self.y + 8 * 0x200 > player.y {
                    self.action_num = 10;
                    self.anim_num = 5;
                    player.cond.set_hidden(true);
                    player.damage(2, state);
                } else {
                    self.action_counter += 1;

                    if self.flags.hit_left_wall() || self.flags.hit_right_wall() || self.action_counter > 75 {
                        self.action_num = 9;
                        self.anim_num = 0;
                    } else if self.action_counter2 % 3 == 0 && self.action_counter > 25 {
                        self.action_num = 4;
                        self.anim_num = 7;
                        self.vel_y = -0x400; // -2.0fix9
                    }
                }
            }
            4 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 9;
                    self.anim_num = 8;
                    state.quake_counter = 30;
                    state.sound_manager.play_sfx(26);
                }

                if self.action_counter >= 8 && (player.x - self.x).abs() < 12 * 0x200
                    && self.y - 12 * 0x200 < player.y && self.y + 8 * 0x200 > player.y {
                    self.action_num = 10;
                    self.anim_num = 5;
                    player.cond.set_hidden(true);
                    player.damage(2, state);
                }
            }
            9 => {
                self.vel_x = self.vel_x * 4 / 5;

                if self.vel_x == 0 {
                    self.action_num = 0;
                }
            }
            10 => {
                player.x = self.x;
                player.y = self.y;

                self.vel_x = self.vel_x * 4 / 5;

                if self.vel_x == 0 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 5;
                    self.anim_counter = 0;
                }
            }
            11 => {
                player.x = self.x;
                player.y = self.y;

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num > 6 {
                        self.anim_num = 5;
                    }
                }

                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 20;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    state.sound_manager.play_sfx(25);
                    player.cond.set_hidden(false);

                    self.direction = self.direction.opposite();

                    player.direction = self.direction;
                    player.x += 4 * 0x200 * self.direction.vector_x();
                    player.y -= 8 * 0x200;
                    player.vel_x = 0x5ff * self.direction.vector_x();
                    player.vel_y = -0x200;

                    self.action_num = 21;
                    self.action_counter = 0;
                    self.anim_num = 7;
                }

                self.action_counter += 1;
                if self.action_counter >= 50 {
                    self.action_num = 0;
                }
            }
            _ => {}
        }

        self.vel_x = clamp(self.vel_x, -0x400, 0x400);
        self.vel_y += 0x20;

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 9 };

        self.anim_rect = state.constants.npc.n068_balrog_running[self.anim_num as usize + dir_offset];

        Ok(())
    }
}

