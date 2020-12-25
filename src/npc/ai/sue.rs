use ggez::GameResult;
use num_traits::clamp;

use crate::common::Direction;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub fn tick_n042_sue(&mut self, state: &mut SharedGameState, players: [&mut Player; 2], npc_list: &NPCList) -> GameResult {
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
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 5 {
                        self.anim_num = 2;
                    }
                }

                self.vel_x = self.direction.vector_x() * 0x200;
            }
            5 => {
                self.anim_num = 6;
                self.vel_x = 0;
            }
            6 | 7 => {
                if self.action_num == 6 {
                    state.sound_manager.play_sfx(50);
                    self.action_counter = 0;
                    self.action_num = 7;
                    self.anim_num = 7;
                }

                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 0;
                }
            }
            8 | 9 => {
                if self.action_num == 8 {
                    state.sound_manager.play_sfx(50);
                    self.action_counter = 0;
                    self.action_num = 9;
                    self.anim_num = 7;
                    self.vel_x = self.direction.vector_x() * -0x400;
                    self.vel_y = -0x200;
                }

                self.action_counter += 1;
                if self.action_counter > 3 && self.flags.hit_bottom_wall() {
                    self.action_num = 10;
                    self.direction = self.direction.opposite();
                }
            }
            10 => {
                self.vel_x = 0;
                self.anim_num = 8;
            }
            11 | 12 => {
                if self.action_num == 11 {
                    self.action_num = 12;
                    self.action_counter = 0;
                    self.anim_num = 9;
                    self.anim_counter = 0;
                    self.vel_x = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 8 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 10 {
                        self.anim_num = 9;
                    }
                }
            }
            13 | 14 => {
                if self.action_num == 13 {
                    self.anim_num = 11;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.action_num = 14;

                    self.parent_id = npc_list.iter_alive()
                        .find_map(|npc| if npc.event_num == 501 { Some(npc.id) } else { None })
                        .unwrap_or(0);
                }

                if let Some(npc) = self.get_parent_ref_mut(npc_list) {
                    self.direction = npc.direction.opposite();
                    self.x = npc.x + npc.direction.vector_x() * 6 * 0x200;
                    self.y = npc.y + 4 * 0x200;

                    if npc.anim_num == 2 || npc.anim_num == 4 {
                        self.y -= 0x200;
                    }
                }
            }
            15 | 16 => {
                if self.action_num == 15 {
                    self.action_num = 16;
                    self.vel_x = 0;
                    self.anim_num = 0;

                    let mut npc = NPC::create(257, &state.npc_table);
                    npc.x = self.x + 128 * 0x200;
                    npc.y = self.y;
                    npc.direction = Direction::Left;
                    npc.cond.set_alive(true);
                    let _ = npc_list.spawn(0, npc.clone());

                    npc.direction = Direction::Right;
                    let _ = npc_list.spawn(0x80, npc);
                }

                state.npc_super_pos = (
                    self.x - 24 * 0x200,
                    self.y - 8 * 0x200
                );
            }
            17 => {
                self.vel_x = 0;
                self.anim_num = 12;

                state.npc_super_pos = (
                    self.x,
                    self.y - 8 * 0x200
                );
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 5 {
                        self.anim_num = 2;
                    }
                }

                self.vel_x = self.direction.vector_x() * 0x400;

                let player = self.get_closest_player_mut(players);
                if self.x < player.x - 8 * 0x200 {
                    self.direction = Direction::Right;
                    self.action_num = 0;
                }
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;

                    self.anim_num += 1;
                    if self.anim_num > 5 {
                        self.anim_num = 2;
                    }
                }

                self.vel_x = self.direction.vector_x() * 0x400;
            }
            40 => {
                self.action_num = 41;
                self.anim_num = 9;
                self.vel_y = -0x400;
            }
            _ => {}
        }

        if self.action_num != 14 {
            self.vel_y += 0x40;

            self.vel_x = clamp(self.vel_x, -0x400, 0x400);
            if self.vel_y > 0x5ff {
                self.vel_y = 0x5ff;
            }

            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 13 };
        self.anim_rect = state.constants.npc.n042_sue[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n092_sue_at_pc(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_counter = 0;

                    self.x -= 4 * 0x200;
                    self.y += 16 * 0x200;
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 1 {
                        self.anim_num = 0;
                    }
                }

                if self.rng.range(0..80) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 2;
                }
            }
            2 => {
                self.action_counter += 1;

                if self.action_counter > 40 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 2;
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 80 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => {}
        }

        self.anim_rect = state.constants.npc.n092_sue_at_pc[self.anim_num as usize];

        Ok(())
    }
}
