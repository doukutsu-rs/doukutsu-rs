use std::cell::RefCell;
use std::collections::BTreeMap;

use ggez::GameResult;
use num_traits::clamp;

use crate::common::Direction;
use crate::npc::{NPC, NPCMap};
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n066_misery_bubble(&mut self, state: &mut SharedGameState, map: &BTreeMap<u16, RefCell<NPC>>) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    for (&id, npc_cell) in map.iter() {
                        if self.id == id { continue; }

                        let npc = npc_cell.borrow();

                        if npc.event_num == 1000 {
                            self.action_counter2 = npc.id;
                            self.target_x = npc.x;
                            self.target_y = npc.y;

                            let angle = ((self.y - self.target_y) as f64 / (self.x - self.target_x) as f64).atan();
                            self.vel_x = (angle.cos() * 1024.0) as isize; // 2.0fix9
                            self.vel_y = (angle.sin() * 1024.0) as isize;

                            log::info!("bubble toss: {:#x} {:#x}", self.vel_x, self.vel_y);

                            break;
                        }
                    }

                    if self.action_counter2 == 0 {
                        self.action_num = 0xffff;
                        return Ok(());
                    }

                    self.action_num = 1;
                }

                self.anim_counter += 1;
                if self.anim_counter > 1 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num > 1 {
                        self.anim_num = 0;
                    }
                }

                if (self.x - self.target_x).abs() < 3 * 0x200 && (self.y - self.target_y).abs() < 3 * 0x200 {
                    self.action_num = 2;
                    self.anim_num = 2;
                    state.sound_manager.play_sfx(21);

                    if let Some(npc) = map.get(&self.action_counter2) {
                        npc.borrow_mut().cond.set_alive(false);
                    }
                }
            }
            2 => {
                self.vel_x -= 0x20;
                self.vel_y -= 0x20;

                self.vel_x = clamp(self.vel_x, -0x5ff, 0x5ff);
                self.vel_y = clamp(self.vel_y, -0x5ff, 0x5ff);

                if self.y < -8 * 0x200 {
                    self.cond.set_alive(false);
                }

                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 3 {
                        self.anim_num = 2;
                    }
                }
            }
            _ => {}
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n066_misery_bubble[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n067_misery_floating(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.target_x = self.x;
                    self.target_y = self.y;

                    state.sound_manager.play_sfx(29);
                }

                self.x = self.target_x + self.rng.range(-1..1) as isize * 0x200;

                self.action_counter += 1;
                if self.action_counter >= 32 {
                    self.action_num = 10;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.vel_y = 0x200;
                }

                if self.target_y < self.y {
                    self.vel_y -= 0x10;
                }

                if self.target_y > self.y {
                    self.vel_y += 0x10;
                }

                self.vel_y = clamp(self.vel_y, -0x100, 0x100);
            }
            13 => {
                self.anim_num = 1;

                self.vel_y += 0x40;

                if self.vel_y > 0x5ff {
                    self.vel_y = 0x5ff;
                }

                if self.flags.hit_bottom_wall() {
                    state.sound_manager.play_sfx(23);
                    self.vel_y = 0;
                    self.action_num = 14;
                    self.npc_flags.set_ignore_solidity(true);
                    self.anim_num = 2;
                }
            }
            15 | 16 => {
                if self.action_num == 15 {
                    self.action_num = 16;
                    self.action_counter = 0;
                    self.anim_num = 4;
                }

                self.action_counter += 1;
                if self.action_counter == 30 {
                    state.sound_manager.play_sfx(21);
                    let mut npc = NPCMap::create_npc(66, &state.npc_table);
                    npc.x = self.x;
                    npc.y = self.y - 16 * 0x200;
                    npc.cond.set_alive(true);

                    state.new_npcs.push(npc);
                }

                if self.action_counter == 50 {
                    self.action_num = 14;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.anim_num = 0;
                    self.vel_y = 0;
                    self.npc_flags.set_ignore_solidity(true);
                }

                self.vel_y -= 0x20;

                if self.y < -8 * 0x200 {
                    self.cond.set_alive(false);
                }
            }
            25 | 26 => {
                if self.action_num == 25 {
                    self.action_num = 26;
                    self.action_counter = 0;
                    self.anim_num = 5;
                    self.anim_counter = 0;
                }

                self.anim_num += 1;
                if self.anim_num > 7 {
                    self.anim_num = 5;
                }

                self.action_counter += 1;
                if self.action_counter == 30 {
                    state.sound_manager.play_sfx(101);
                    // todo flash
                    self.action_num = 27;
                    self.anim_num = 7;
                }
            }
            27 => {
                self.action_counter += 1;
                if self.action_counter == 50 {
                    self.action_num = 14;
                }
            }
            _ => {}
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        match self.action_num {
            11 => {}
            14 => {}
            _ => {}
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 8 };
        self.anim_rect = state.constants.npc.n067_misery_floating[self.anim_num as usize + dir_offset];

        if self.action_num == 1 && self.anim_counter < 32 {
            self.anim_counter += 1;
            self.anim_rect.bottom = self.anim_counter / 2 + self.anim_rect.bottom - 16;
        }

        Ok(())
    }

    pub(crate) fn tick_n082_misery_standing(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 2;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 3;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 2;
                }
            }
            15 | 16 => {
                if self.action_num == 15 {
                    self.action_num = 16;
                    self.action_counter = 0;
                    self.anim_num = 4;
                }

                self.action_counter += 1;
                if self.action_counter == 30 {
                    state.sound_manager.play_sfx(21);

                    let mut npc = NPCMap::create_npc(66, &state.npc_table);
                    npc.x = self.x;
                    npc.y = self.y - 16 * 0x200;
                    npc.cond.set_alive(true);

                    state.new_npcs.push(npc);
                }

                if self.action_counter == 50 {
                    self.action_num = 14;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.anim_num = 0;
                    self.vel_y = 0;
                    self.npc_flags.set_ignore_solidity(true);
                }

                self.vel_y -= 0x20;

                if self.y < -8 * 0x200 {
                    self.cond.set_alive(false);
                }
            }
            25 | 26 => {
                if self.action_num == 25 {
                    self.action_num = 26;
                    self.action_counter = 0;
                    self.anim_num = 5;
                    self.anim_counter = 0;
                }

                self.anim_num += 1;
                if self.anim_num > 7 {
                    self.anim_num = 5;
                }

                self.action_counter += 1;
                if self.action_counter == 30 {
                    state.sound_manager.play_sfx(101);
                    // todo flash
                    self.action_num = 27;
                    self.anim_num = 7;
                }
            }
            27 => {
                self.action_counter += 1;
                if self.action_counter == 50 {
                    self.action_num = 14;
                }
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.anim_num = 3;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 10 {
                    self.action_num = 32;
                    self.anim_num = 4;
                    self.anim_counter = 0;
                }
            }
            32 => {
                self.anim_counter += 1;
                if self.anim_counter > 100 {
                    self.action_num = 1;
                    self.anim_num = 2;
                }
            }
            40 | 41 => {
                if self.action_num == 40 {
                    self.action_num = 41;
                    self.action_counter = 0;
                }

                self.anim_num = 4;

                self.action_counter += 1;
                if self.action_counter == 30 || self.action_counter == 40 || self.action_counter == 50 {
                    state.sound_manager.play_sfx(33);

                    let mut npc = NPCMap::create_npc(11, &state.npc_table);
                    npc.x = self.x + 8 * 0x200;
                    npc.y = self.y - 8 * 0x200;
                    npc.vel_x = 0x600;
                    npc.vel_y = self.rng.range(-0x200..0) as isize;
                    npc.cond.set_alive(true);

                    state.new_npcs.push(npc);
                }
            }
            50 => {
                self.anim_num = 8;
            }
            _ => {}
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.action_num == 11
        {
            if self.anim_counter != 0
            {
                self.anim_counter -= 1;
                self.anim_num = 1;
            } else {
                if self.rng.range(0..100) == 1 {
                    self.anim_counter = 30;
                }

                self.anim_num = 0;
            }
        }

        if self.action_num == 14
        {
            if self.action_counter != 0
            {
                self.action_counter -= 1;
                self.anim_num = 3;
            } else {
                if self.rng.range(0..100) == 1 {
                    self.anim_counter = 30;
                }

                self.anim_num = 2;
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 9 };
        self.anim_rect = state.constants.npc.n082_misery_standing[self.anim_num as usize + dir_offset];

        Ok(())
    }
}

