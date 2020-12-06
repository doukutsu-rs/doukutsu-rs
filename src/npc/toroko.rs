use num_traits::clamp;

use crate::common::Direction;
use ggez::GameResult;
use crate::npc::NPC;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n060_toroko(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                let player = self.get_closest_player_mut(players);
                if (self.x - (16 * 0x200) < player.x) && (self.x + (16 * 0x200) > player.x)
                    && (self.y - (16 * 0x200) < player.y) && (self.y + (16 * 0x200) > player.y) {
                    if self.x > player.x {
                        self.direction = Direction::Left;
                    } else {
                        self.direction = Direction::Right;
                    }
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            3 | 4 => {
                if self.action_num == 3 {
                    self.action_num = 4;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }

                if self.anim_num > 4 {
                    self.anim_num = 1;
                }

                if self.flags.hit_left_wall() {
                    self.direction = Direction::Right;
                    self.vel_x = 0x200;
                }

                if self.flags.hit_right_wall() {
                    self.direction = Direction::Left;
                    self.vel_x = -0x200;
                }

                if self.direction == Direction::Left {
                    self.vel_x = -0x400;
                } else {
                    self.vel_x = 0x400;
                }
            }
            6 | 7 => {
                if self.action_num == 6 {
                    self.action_num = 7;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                    self.vel_y = -0x400;
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }

                if self.anim_num > 4 {
                    self.anim_num = 1;
                }

                if self.direction == Direction::Left {
                    self.vel_x = -0x100;
                } else {
                    self.vel_x = 0x100;
                }

                if self.action_counter != 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 3;
                }

                self.action_counter += 1;
            }
            8 | 9 => {
                if self.action_num == 8 {
                    self.anim_num = 1;
                    self.action_counter = 0;
                    self.action_num = 9;
                    self.vel_y = -0x200;
                }

                if self.action_counter != 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 0;
                }

                self.action_counter += 1;
            }
            10 => {
                self.action_num = 11;
                self.anim_num = 6;
                self.vel_y = -0x400;

                state.sound_manager.play_sfx(50);

                if self.direction == Direction::Left {
                    self.vel_x = -0x100;
                } else {
                    self.vel_x = 0x100;
                }
            }
            11 => {
                if self.action_counter != 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 12;
                    self.anim_num = 7;
                    self.npc_flags.set_interactable(true);
                }

                self.action_counter += 1;
            }
            12 => {
                self.vel_x = 0;
            }
            _ => {}
        }

        self.vel_y += 0x40;
        self.vel_x = clamp(self.vel_x, -0x400, 0x400);

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.direction == Direction::Left {
            self.anim_rect = state.constants.npc.n060_toroko[self.anim_num as usize];
        } else {
            self.anim_rect = state.constants.npc.n060_toroko[self.anim_num as usize + 8];
        }

        Ok(())
    }

    pub(crate) fn tick_n063_toroko_stick(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_counter = 0;
                    self.vel_y = -0x400;
                }

                if self.vel_y > 0 {
                    self.npc_flags.set_ignore_solidity(false);
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 3 {
                        self.anim_num = 0;
                    }
                }

                self.vel_x = 0x100 * self.direction.vector_x();

                self.action_counter += 1;
                if self.action_counter != 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 2;
                }
            }
            2 | 3 => {
                if self.action_num == 2 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 3 {
                        self.anim_num = 0;
                    }
                }

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_counter = 40;
                    self.vel_x = -self.vel_x;

                    self.direction = self.direction.opposite();
                }

                if self.action_counter > 35 {
                    self.npc_flags.set_shootable(true);
                }

                self.vel_x += 0x40 * self.direction.vector_x();

                if self.shock > 0 {
                    self.action_num = 4;
                    self.anim_num = 4;
                    self.vel_y = -0x400;
                    self.npc_flags.set_shootable(false);
                    self.damage = 0;
                }
            }
            4 => {
                self.vel_x = 0x100 * self.direction.vector_x();


                self.action_counter += 1;
                if self.action_counter != 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 5;
                    self.npc_flags.set_interactable(true);
                }
            }
            5 => {
                self.vel_x = 0;
                self.anim_num = 5;
            }
            _ => {}
        }

        self.vel_y += 0x40;
        self.vel_x = clamp(self.vel_x, -0x400, 0x400);

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;


        if self.direction == Direction::Left {
            self.anim_rect = state.constants.npc.n063_toroko_stick[self.anim_num as usize];
        } else {
            self.anim_rect = state.constants.npc.n063_toroko_stick[self.anim_num as usize + 6];
        }

        Ok(())
    }
}
