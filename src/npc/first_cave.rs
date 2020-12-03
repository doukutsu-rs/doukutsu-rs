use ggez::GameResult;
use num_traits::{abs, clamp};

use crate::common::Direction;
use crate::npc::NPC;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n059_eye_door(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                }

                let player = self.get_closest_player_mut(players);
                if self.x - (64 * 0x200) < player.x
                    && self.x + (64 * 0x200) > player.x
                    && self.y - (64 * 0x200) < player.y
                    && self.y + (64 * 0x200) > player.y {
                    self.action_num = 2;
                    self.anim_counter = 0;
                }
            }
            2 => {
                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num == 2 {
                        self.action_num = 3;
                    }
                }
            }
            3 => {
                let player = self.get_closest_player_mut(players);
                if !(self.x - (64 * 0x200) < player.x
                    && self.x + (64 * 0x200) > player.x
                    && self.y - (64 * 0x200) < player.y
                    && self.y + (64 * 0x200) > player.y) {
                    self.action_num = 4;
                    self.anim_counter = 0;
                }
            }
            4 => {
                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num -= 1;

                    if self.anim_num == 0 {
                        self.action_num = 1;
                    }
                }
            }
            _ => {}
        }

        if self.shock > 0 {
            self.anim_rect = state.constants.npc.n059_eye_door[3];
        } else {
            self.anim_rect = state.constants.npc.n059_eye_door[self.anim_num as usize];
        }
        Ok(())
    }


    pub(crate) fn tick_n064_first_cave_critter(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.y += 3 * 0x200;
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_rect = state.constants.npc.n064_first_cave_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                }

                let player = self.get_closest_player_mut(players);
                if self.x > player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }

                if self.target_x < 100 {
                    self.target_x += 1;
                }

                if self.action_counter >= 8
                    && self.x - (112 * 0x200) < player.x
                    && self.x + (112 * 0x200) > player.x
                    && self.y - (80 * 0x200) < player.y
                    && self.y + (80 * 0x200) > player.y {
                    if self.anim_num != 1 {
                        self.anim_num = 1;
                        self.anim_rect = state.constants.npc.n064_first_cave_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                } else {
                    if self.action_counter < 8 {
                        self.action_counter += 1;
                    }

                    if self.anim_num != 0 {
                        self.anim_num = 0;
                        self.anim_rect = state.constants.npc.n064_first_cave_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                }

                if self.shock > 0 {
                    self.action_num = 2;
                    self.action_counter = 0;

                    if self.anim_num != 0 {
                        self.anim_num = 0;
                        self.anim_rect = state.constants.npc.n064_first_cave_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                }

                if self.action_counter >= 8
                    && self.target_x >= 100
                    && self.x - (64 * 0x200) < player.x
                    && self.x + (64 * 0x200) > player.x
                    && self.y - (80 * 0x200) < player.y
                    && self.y + (80 * 0x200) > player.y {
                    self.action_num = 2;
                    self.action_counter = 0;

                    if self.anim_num != 0 {
                        self.anim_num = 0;
                        self.anim_rect = state.constants.npc.n064_first_cave_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 3;

                    if self.anim_num != 2 {
                        self.anim_num = 2;
                        self.anim_rect = state.constants.npc.n064_first_cave_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }

                    self.vel_y = -0x5ff;
                    state.sound_manager.play_sfx(30);

                    if self.direction == Direction::Left {
                        self.vel_x = -0x100;
                    } else {
                        self.vel_x = 0x100;
                    }
                }
            }
            3 => {
                if self.flags.hit_bottom_wall() {
                    self.vel_x = 0;
                    self.action_counter = 0;
                    self.action_num = 1;

                    state.sound_manager.play_sfx(23);

                    if self.anim_num != 0 {
                        self.anim_num = 0;
                        self.anim_rect = state.constants.npc.n064_first_cave_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                }
            }
            _ => {}
        }

        self.vel_y += 0x40;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n065_first_cave_bat(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.target_x = self.x;
                    self.target_y = self.y;

                    self.action_num = 1;
                    self.action_counter = state.game_rng.range(0..50) as u16;
                }

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.vel_y = 0x300;
                }
            }
            2 => {
                let player = self.get_closest_player_mut(players);
                if self.x > player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }

                if self.target_y < self.y {
                    self.vel_y -= 0x10;
                } else if self.target_y > self.y {
                    self.vel_y += 0x10;
                }

                self.vel_y = clamp(self.vel_y, -0x300, 0x300);
            }
            _ => {}
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num += 1;

            if self.anim_num > 2 {
                self.anim_num = 0;
            }

            self.anim_rect = state.constants.npc.n065_first_cave_bat[self.anim_num as usize + if self.direction == Direction::Right { 4 } else { 0 }];
        }

        Ok(())
    }
}
