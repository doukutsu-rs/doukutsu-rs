use ggez::GameResult;
use num_traits::{abs, clamp};

use crate::common::Direction;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n044_polish(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 | 1 => {
                self.anim_num = 0;
                self.action_num = match self.direction {
                    Direction::Left => 8,
                    Direction::Right => 2,
                    _ => 8,
                };
            }
            2 => {
                self.vel_y += 0x20;
                if self.vel_y > 0 && self.flags.hit_bottom_wall() {
                    self.vel_y = -0x100;
                    self.vel_x += 0x100;
                }

                if self.flags.hit_right_wall() {
                    self.action_num = 3;
                }
            }
            3 => {
                self.vel_x += 0x20;
                if self.vel_x > 0 && self.flags.hit_right_wall() {
                    self.vel_x = -0x100;
                    self.vel_y -= 0x100;
                }

                if self.flags.hit_top_wall() {
                    self.action_num = 4;
                }
            }
            4 => {
                self.vel_y -= 0x20;
                if self.vel_y < 0 && self.flags.hit_top_wall() {
                    self.vel_y = 0x100;
                    self.vel_x -= 0x100;
                }

                if self.flags.hit_left_wall() {
                    self.action_num = 5;
                }
            }
            5 => {
                self.vel_x -= 0x20;
                if self.vel_x < 0 && self.flags.hit_left_wall() {
                    self.vel_x = 0x100;
                    self.vel_y += 0x100;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 2;
                }
            }
            6 => {
                self.vel_y += 0x20;
                if self.vel_y > 0 && self.flags.hit_bottom_wall() {
                    self.vel_y = -0x100;
                    self.vel_x -= 0x100;
                }

                if self.flags.hit_left_wall() {
                    self.action_num = 7;
                }
            }
            7 => {
                self.vel_x -= 0x20;
                if self.vel_x < 0 && self.flags.hit_left_wall() {
                    self.vel_x = 0x100;
                    self.vel_y -= 0x100;
                }

                if self.flags.hit_top_wall() {
                    self.action_num = 8;
                }
            }
            8 => {
                self.vel_y -= 0x20;
                if self.vel_y < 0 && self.flags.hit_top_wall() {
                    self.vel_y = 0x100;
                    self.vel_x += 0x100;
                }

                if self.flags.hit_right_wall() {
                    self.action_num = 9;
                }
            }
            9 => {
                self.vel_x += 0x20;
                if self.vel_x > 0 && self.flags.hit_right_wall() {
                    self.vel_x = -0x100;
                    self.vel_y += 0x100;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 6;
                }
            }
            _ => {}
        }

        if self.life <= 100 {
            npc_list.create_death_smoke(self.x, self.y, self.display_bounds.right, 8, state, &self.rng);
            state.sound_manager.play_sfx(25);
            self.cond.set_alive(false);

            let mut npc = NPC::create(45, &state.npc_table);
            npc.cond.set_alive(true);
            npc.x = self.x;
            npc.y = self.y;
            for _ in 0..9 {
                let _ = npc_list.spawn(0x100, npc.clone());
            }
        }

        self.vel_x = clamp(self.vel_x, -0x200, 0x200);
        self.vel_y = clamp(self.vel_y, -0x200, 0x200);

        if self.shock > 0 {
            self.x += self.vel_x / 2;
            self.y += self.vel_y / 2;
        } else {
            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        if self.action_num > 1 && self.action_num <= 9 {
            self.anim_num += 1;
            if self.anim_num > 2 {
                self.anim_num = 1;
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n044_polish[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n045_baby(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 2;
            self.vel_x = if self.rng.next_u16() & 1 != 0 {
                self.rng.range(-0x200..-0x100) as i32
            } else {
                self.rng.range(0x100..0x200) as i32
            };
            self.vel_y = if self.rng.next_u16() & 1 != 0 {
                self.rng.range(-0x200..-0x100) as i32
            } else {
                self.rng.range(0x100..0x200) as i32
            };
            self.vel_x2 = self.vel_x;
            self.vel_y2 = self.vel_y;
        }

        match self.action_num {
            1 | 2 => {
                self.anim_num += 1;
                if self.anim_num > 2 {
                    self.anim_num = 1;
                }
            }
            _ => {}
        }

        if self.vel_x2 < 0 && self.flags.hit_left_wall() {
            self.vel_x2 = -self.vel_x2;
        }

        if self.vel_x2 > 0 && self.flags.hit_right_wall() {
            self.vel_x2 = -self.vel_x2;
        }

        if self.vel_y2 < 0 && self.flags.hit_top_wall() {
            self.vel_y2 = -self.vel_y2;
        }

        if self.vel_y2 > 0 && self.flags.hit_bottom_wall() {
            self.vel_y2 = -self.vel_y2;
        }

        self.vel_x2 = clamp(self.vel_x2, -0x200, 0x200);
        self.vel_y2 = clamp(self.vel_y2, -0x200, 0x200);

        if self.shock > 0 {
            self.x += self.vel_x2 / 2;
            self.y += self.vel_y2 / 2;
        } else {
            self.x += self.vel_x2;
            self.y += self.vel_y2;
        }

        self.anim_rect = state.constants.npc.n045_baby[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n047_sandcroc(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.target_y = self.y;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_ignore_solidity(false);
                    self.npc_flags.set_invulnerable(false);
                    self.npc_flags.set_solid_soft(false);
                }

                let player = self.get_closest_player_mut(players);
                if abs(self.x - player.x) < 8 * 0x200 && player.y > self.y && player.y < self.y + 8 * 0x200 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    state.sound_manager.play_sfx(102);
                }

                self.x += (player.x - self.x).signum() * 2 * 0x200;
            }
            2 => {
                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_num += 1;
                    self.anim_counter = 0;
                }

                match self.anim_num {
                    3 => self.damage = 10,
                    4 => {
                        self.action_num = 3;
                        self.action_counter = 0;
                        self.npc_flags.set_shootable(true);
                    }
                    _ => {}
                }
            }
            3 => {
                self.damage = 0;
                self.npc_flags.set_solid_soft(true);

                self.action_counter += 1;
                if self.shock > 0 {
                    self.action_num = 4;
                    self.action_counter = 0;
                }
            }
            4 => {
                self.npc_flags.set_ignore_solidity(true);
                self.y += 0x200;
                self.action_counter += 1;
                if self.action_counter == 32 {
                    self.action_num = 5;
                    self.action_counter = 0;
                    self.npc_flags.set_solid_soft(false);
                    self.npc_flags.set_shootable(false);
                }
            }
            5 => {
                if self.action_counter > 99 {
                    self.y = self.target_y;
                    self.action_num = 0;
                    self.anim_num = 0;
                } else {
                    self.action_counter += 1;
                }
            }
            _ => {}
        }

        self.anim_rect = state.constants.npc.n047_sandcroc[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n049_skullhead(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        let parent = self.get_parent_ref_mut(npc_list);

        Ok(())
    }

    pub(crate) fn tick_n124_sunstone(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.x += 8 * 0x200;
                    self.y += 8 * 0x200;
                }

                self.npc_flags.set_ignore_solidity(false);
                self.anim_num = 0;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 1;

                    self.npc_flags.set_ignore_solidity(true);
                }

                match self.direction {
                    Direction::Left => self.x -= 0x80,
                    Direction::Up => self.y -= 0x80,
                    Direction::Right => self.x += 0x80,
                    Direction::Bottom => self.y += 0x80,
                    Direction::FacingPlayer => {}
                }

                state.quake_counter = 20;
                if self.action_counter % 8 == 0 {
                    state.sound_manager.play_sfx(26);
                }
            }
            _ => {}
        }

        self.anim_rect = state.constants.npc.n124_sunstone[self.anim_num as usize];

        Ok(())
    }
}
