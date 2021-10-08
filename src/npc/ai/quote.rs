use crate::framework::error::GameResult;

use crate::common::Direction;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub fn tick_n111_quote_teleport_out(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 0;
                self.y -= 0x2000;
            }
            1 => {
                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.vel_y = -0x2ff;
                }
            }
            2 => {
                if self.vel_y > 0 {
                    self.hit_bounds.bottom = 0x2000;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_counter = 0;
                    self.action_num = 3;
                    self.anim_num = 0;
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.action_counter = 64;
                    self.action_num = 4;

                    state.sound_manager.play_sfx(29);
                }
            }
            4 => {
                self.anim_num = 0;
                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };
        self.anim_rect = state.constants.npc.n111_quote_teleport_out[self.anim_num as usize + dir_offset];

        let offset = players[state.textscript_vm.executor_player.index()].get_texture_offset();
        self.anim_rect.top += offset;
        self.anim_rect.bottom += offset;

        if self.action_num == 4 {
            self.anim_rect.bottom = self.anim_rect.top + self.action_counter / 4;

            if self.action_counter / 2 % 2 != 0 {
                self.anim_rect.left += 1;
            }
        }

        Ok(())
    }

    pub fn tick_n112_quote_teleport_in(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 0;
                self.anim_counter = 0;
                self.x += 0x2000;
                self.y += 0x1000;

                state.sound_manager.play_sfx(29);
            }
            1 => {
                self.action_counter += 1;
                if self.action_counter >= 64 {
                    self.action_num = 2;
                    self.action_counter = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 3;
                    self.anim_num = 1;
                    self.hit_bounds.bottom = 0x1000;
                }
            }
            3 => {
                if self.flags.hit_bottom_wall() {
                    self.action_counter = 0;
                    self.action_num = 4;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };
        self.anim_rect = state.constants.npc.n111_quote_teleport_out[self.anim_num as usize + dir_offset];

        let offset = players[state.textscript_vm.executor_player.index()].get_texture_offset();
        self.anim_rect.top += offset;
        self.anim_rect.bottom += offset;

        if self.action_num == 1 {
            self.anim_rect.bottom = self.anim_rect.top + self.action_counter / 4;

            if self.action_counter / 2 % 2 != 0 {
                self.anim_rect.left += 1;
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n150_quote(&mut self, state: &mut SharedGameState, players: [&mut Player; 2], npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 0;

                if self.tsc_direction > 10 {
                    let player = &players[state.textscript_vm.executor_player.index()];
                    self.x = player.x;
                    self.y = player.y;

                    self.direction = Direction::from_int(self.tsc_direction.saturating_sub(10) as usize)
                        .unwrap_or(Direction::Left);
                } else {
                    self.direction = Direction::from_int(self.tsc_direction as usize)
                        .unwrap_or(Direction::Left);
                }
            }
            2 => {
                self.anim_num = 1;
            }
            10 => {
                self.action_num = 11;
                self.anim_num = 2;

                state.sound_manager.play_sfx(71);

                let mut npc = NPC::create(4, &state.npc_table);
                npc.cond.set_alive(true);
                npc.direction = Direction::Left;
                npc.x = self.x;
                npc.y = self.y;

                for _ in 0..4 {
                    npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                    npc.vel_y = self.rng.range(-0x600..0) as i32;

                    let _ = npc_list.spawn(0x100, npc.clone());
                }
            }
            11 => {
                self.anim_num = 2;
            }
            20 => {
                self.action_num = 21;
                self.action_counter = 63;

                state.sound_manager.play_sfx(29);
            }
            21 => {
                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.cond.set_alive(false);
                }
            }
            50 | 51 => {
                if self.action_num == 50 {
                    self.action_num = 51;
                    self.anim_num = 3;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 6 {
                        self.anim_num = 3;
                    }
                }

                self.x += self.direction.vector_x() * 0x200;
            }
            60 | 61 => {
                if self.action_num == 60 {
                    self.action_num = 61;
                    self.anim_num = 7;
                    self.target_x = self.x;
                    self.target_y = self.y;
                }

                self.target_y += 0x100;
                self.x = self.target_x + self.rng.range(-1..1) as i32 * 0x200;
                self.y = self.target_y + self.rng.range(-1..1) as i32 * 0x200;
            }
            70 | 71 => {
                if self.action_num == 70 {
                    self.action_num = 71;
                    self.action_counter = 0;
                    self.anim_num = 3;
                    self.anim_counter = 0;
                }

                self.x += (self.direction.vector_x() as i32 | 1) * 0x100;

                self.anim_counter += 1;
                if self.anim_counter > 8 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 6 {
                        self.anim_num = 3;
                    }
                }
            }
            80 => {
                self.anim_num = 8;
            }
            99 | 100 | 101 => {
                if self.action_num == 99 || self.action_num == 100 {
                    self.action_num = 101;
                    self.anim_num = 3;
                    self.anim_counter = 0;
                }

                self.vel_y += 0x40;
                if self.vel_y > 0x5ff {
                    self.vel_y = 0x5ff;
                }

                if self.flags.hit_bottom_wall() {
                    self.vel_y = 0;
                    self.action_num = 102;
                }

                self.y += self.vel_y;
            }
            102 => {
                self.anim_counter += 1;
                if self.anim_counter > 8 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 6 {
                        self.anim_num = 3;
                    }
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 9 };
        self.anim_rect = state.constants.npc.n150_quote[self.anim_num as usize + dir_offset];

        if self.action_num == 21 {
            self.anim_rect.bottom = self.anim_rect.top + self.action_counter / 4;
        }

        let offset = players[state.textscript_vm.executor_player.index()].get_texture_offset();
        self.anim_rect.top += offset;
        self.anim_rect.bottom += offset;

        Ok(())
    }
}
