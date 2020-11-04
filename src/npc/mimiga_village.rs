use std::cmp::Ordering;

use num_traits::{abs, clamp};

use crate::common::Direction;
use ggez::GameResult;
use crate::npc::NPC;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n069_pignon(&mut self, state: &mut SharedGameState) -> GameResult {
        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;

                    self.anim_rect = state.constants.npc.n069_pignon[self.anim_num as usize + dir_offset];
                }

                if state.game_rng.range(0..100) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;

                    self.anim_rect = state.constants.npc.n069_pignon[self.anim_num as usize + dir_offset];
                }


                if state.game_rng.range(0..150) == 1 {
                    self.action_num = 3;
                    self.action_counter = 50;
                    self.anim_num = 0;

                    self.anim_rect = state.constants.npc.n069_pignon[self.anim_num as usize + dir_offset];
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;

                    self.anim_rect = state.constants.npc.n069_pignon[self.anim_num as usize + dir_offset];
                }
            }
            3 | 4 => {
                if self.action_num == 3 {
                    self.action_num = 4;
                    self.anim_num = 2;
                    self.anim_counter = 0;

                    self.anim_rect = state.constants.npc.n069_pignon[self.anim_num as usize + dir_offset];
                }

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 4 {
                        self.anim_num = 2;
                    }

                    self.anim_rect = state.constants.npc.n069_pignon[self.anim_num as usize + dir_offset];
                }

                if self.flags.hit_left_wall() {
                    self.direction = Direction::Right;
                }

                if self.flags.hit_right_wall() {
                    self.direction = Direction::Left;
                }

                self.vel_x = self.direction.vector_x() * 0x100; // 0.5fix9
            }
            5 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 0;
                }
            }
            _ => {}
        }

        if self.shock > 0 && [1, 2, 4].contains(&self.action_num) {
            self.vel_y = -0x200;
            self.anim_num = 5;
            self.action_num = 5;

            self.anim_rect = state.constants.npc.n069_pignon[self.anim_num as usize + dir_offset];
        }

        self.vel_y += 0x40;

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n071_chinfish(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.target_x = self.x;
            self.target_y = self.y;
            self.vel_y = 0x80;
        }

        if self.action_num == 1 {
            self.vel_y += match self.target_y.cmp(&self.y) {
                Ordering::Less => { -8 }
                Ordering::Equal => { 0 }
                Ordering::Greater => { 8 }
            };

            self.vel_y = clamp(self.vel_y, -0x100, 0x100);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_counter += 1;
        if self.anim_counter > 4 {
            self.anim_counter = 0;
            self.anim_num += 1;
        }

        if self.anim_num > 1 {
            self.anim_num = 0;
        }

        if self.shock > 0 {
            self.anim_num = 2;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n071_chinfish[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n075_kanpachi(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = 0;
            self.anim_counter = 0;
        }

        if self.action_num == 1 {
            if (self.x - (48 * 0x200) < player.x) && (self.x + (48 * 0x200) > player.x)
                && (self.y - (48 * 0x200) < player.y) && (self.y + (48 * 0x200) > player.y) {
                self.anim_num = 1;
            } else {
                self.anim_num = 0;
            }
        }

        self.anim_rect = state.constants.npc.n075_kanpachi[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n077_yamashita(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = 0;
            self.anim_counter = 0;
        }

        match self.action_num {
            1 => {
                if state.game_rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => {}
        }

        if self.direction == Direction::Left {
            self.anim_rect = state.constants.npc.n077_yamashita[self.anim_num as usize];
        } else {
            self.anim_rect = state.constants.npc.n077_yamashita[2];
        }

        Ok(())
    }

    pub(crate) fn tick_n079_mahin(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 2;
                self.anim_counter = 0;
            }
            2 => {
                self.anim_num = 0;
                if state.game_rng.range(0..120) == 10 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                if (self.x - (32 * 0x200) < player.x) && (self.x + (32 * 0x200) > player.x)
                    && (self.y - (32 * 0x200) < player.y) && (self.y + (16 * 0x200) > player.y) {
                    if self.x > player.x {
                        self.direction = Direction::Left;
                    } else {
                        self.direction = Direction::Right;
                    }
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 2;
                    self.anim_num = 0;
                }
            }
            _ => {}
        }


        self.vel_y += 0x40;

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n079_mahin[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n080_gravekeeper(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.npc_flags.set_shootable(false);
                    self.damage = 0;
                    self.action_num = 1;
                    self.hit_bounds.left = 4 * 0x200;
                }

                self.anim_num = 0;

                if abs(player.x - self.x) < 128 * 0x200
                    && self.y - 48 * 0x200 < player.y && self.y + 32 * 0x200 > player.y {
                    self.anim_counter = 0;
                    self.action_num = 2;
                }

                if self.shock > 0 {
                    self.anim_num = 1;
                    self.anim_counter = 0;
                    self.action_num = 2;
                    self.npc_flags.set_shootable(false);
                }

                self.direction = if player.x < self.x { Direction::Left } else { Direction::Right };
            }
            2 => {
                self.anim_counter += 1;
                if self.anim_counter > 6 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 3 {
                        self.anim_num = 0;
                    }
                }

                if abs(player.x - self.x) < 16 * 0x200 {
                    self.hit_bounds.left = 18 * 0x200;
                    self.action_counter = 0;
                    self.action_num = 3;
                    self.npc_flags.set_shootable(true);

                    state.sound_manager.play_sfx(34);
                }

                self.direction = if player.x < self.x { Direction::Left } else { Direction::Right };
                self.vel_x = self.direction.vector_x() * 0x100;
            }
            3 => {
                self.anim_num = 4;
                self.vel_x = 0;

                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.action_counter = 0;
                    self.action_num = 4;
                    state.sound_manager.play_sfx(106);
                }
            }
            4 => {
                self.anim_num = 5;
                self.damage = 10;

                self.action_counter += 1;
                if self.action_counter > 2 {
                    self.action_counter = 0;
                    self.action_num = 5;
                }
            }
            5 => {
                self.action_counter += 1;
                if self.action_counter > 60 {
                    self.action_num = 0;
                }

                self.anim_num = 6;
            }
            _ => {}
        }

        if (self.vel_x < 0 && self.flags.hit_left_wall())
            || (self.vel_x > 0 && self.flags.hit_right_wall()) {
            self.vel_x = 0;
        }

        self.vel_x = clamp(self.vel_x, -0x400, 0x400);
        self.vel_y = clamp(self.vel_y + 0x20, -0x5ff, 0x5ff);

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 7 };
        self.anim_rect = state.constants.npc.n080_gravekeeper[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n081_giant_pignon(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;

                    self.anim_rect = state.constants.npc.n081_giant_pignon[self.anim_num as usize + dir_offset];
                }

                if state.game_rng.range(0..100) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;

                    self.anim_rect = state.constants.npc.n081_giant_pignon[self.anim_num as usize + dir_offset];
                }


                if state.game_rng.range(0..150) == 1 {
                    self.action_num = 3;
                    self.action_counter = 50;
                    self.anim_num = 0;

                    self.anim_rect = state.constants.npc.n081_giant_pignon[self.anim_num as usize + dir_offset];
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;

                    self.anim_rect = state.constants.npc.n081_giant_pignon[self.anim_num as usize + dir_offset];
                }
            }
            3 | 4 => {
                if self.action_num == 3 {
                    self.action_num = 4;
                    self.anim_num = 2;
                    self.anim_counter = 0;

                    self.anim_rect = state.constants.npc.n081_giant_pignon[self.anim_num as usize + dir_offset];
                }

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 4 {
                        self.anim_num = 2;
                    }

                    self.anim_rect = state.constants.npc.n081_giant_pignon[self.anim_num as usize + dir_offset];
                }

                if self.flags.hit_left_wall() {
                    self.direction = Direction::Right;
                }

                if self.flags.hit_right_wall() {
                    self.direction = Direction::Left;
                }

                self.vel_x = self.direction.vector_x() * 0x100; // 0.5fix9
            }
            5 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 0;
                }
            }
            _ => {}
        }

        if self.shock > 0 && [1, 2, 4].contains(&self.action_num) {
            self.vel_x = if self.x < player.x { 0x100 } else { -0x100 };
            self.vel_y = -0x200;
            self.anim_num = 5;
            self.action_num = 5;

            self.anim_rect = state.constants.npc.n081_giant_pignon[self.anim_num as usize + dir_offset];
        }

        self.vel_y += 0x40;

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n091_mimiga_cage(&mut self, state: &SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y += 16 * 0x200;
            self.anim_rect = state.constants.npc.n091_mimiga_cage;
        }
        
        Ok(())
    }
}
