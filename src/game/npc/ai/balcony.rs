use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n254_helicopter(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;

                // blades
                let mut npc = NPC::create(255, &state.npc_table);
                npc.cond.set_alive(true);

                npc.x = self.x + 0x2400;
                npc.y = self.y - 0x7200;
                npc.parent_id = self.id;

                let _ = npc_list.spawn(0x100, npc.clone());

                npc.x = self.x - 0x4000;
                npc.y = self.y - 0x6800;
                npc.direction = Direction::Right;

                let _ = npc_list.spawn(0x100, npc);
            }
            20 => {
                self.action_num = 21;
                self.action_counter = 0;
                self.action_counter2 = 60;
            }
            30 => {
                self.action_num = 21;

                // momorin
                let mut npc = NPC::create(223, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x - 0x1600;
                npc.y = self.y - 0x1c00;

                let _ = npc_list.spawn(0x100, npc);
            }
            40 => {
                self.action_num = 21;

                // momorin
                let mut npc = NPC::create(223, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x - 0x1200;
                npc.y = self.y - 0x1c00;

                let _ = npc_list.spawn(0x100, npc);

                // santa
                let mut npc = NPC::create(40, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x - 0x2c00;
                npc.y = self.y - 0x1c00;

                let _ = npc_list.spawn(0x100, npc);

                // chaco
                let mut npc = NPC::create(223, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x - 0x4600;
                npc.y = self.y - 0x1c00;

                let _ = npc_list.spawn(0x100, npc);
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 1 };

        self.anim_rect = state.constants.npc.n254_helicopter[dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n255_helicopter_blades(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    if self.direction == Direction::Left {
                        self.display_bounds.left = 0x7000;
                        self.display_bounds.right = 0x7000;
                    } else {
                        self.display_bounds.left = 0x5000;
                        self.display_bounds.right = 0x5000;
                    }
                }

                if let Some(parent) = self.get_parent_ref_mut(npc_list) {
                    if parent.action_num >= 20 {
                        self.action_num = 10;
                    }
                }
            }
            10 | 11 => {
                self.action_num = 11;

                self.anim_num += 1;
                if self.anim_num > 3 {
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.npc.n255_helicopter_blades[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n260_shovel_brigade_caged(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.x += 0x200;
                    self.y -= 0x400;
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..160) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 12 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            10 => {
                self.action_num = 11;
                self.anim_num = 2;

                // create heart
                let mut npc = NPC::create(87, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;
                npc.y = self.y - 0x2000;

                let _ = npc_list.spawn(0x100, npc);
            }
            _ => (),
        }

        let player = self.get_closest_player_ref(&players);
        self.face_player(player);

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n260_shovel_brigade_caged[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n261_chie_caged(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.x -= 0x200;
                    self.y -= 0x400;
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }

                if self.rng.range(0..160) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 12 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        let player = self.get_closest_player_ref(&players);
        self.face_player(player);

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n261_chie_caged[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n262_chaco_caged(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.x -= 0x200;
                    self.y -= 0x400;
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }

                if self.rng.range(0..160) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 12 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        let player = self.get_closest_player_ref(&players);
        self.face_player(player);

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n262_chaco_caged[self.anim_num as usize + dir_offset];

        Ok(())
    }
}
