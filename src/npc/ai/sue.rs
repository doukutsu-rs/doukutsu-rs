use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::npc::boss::BossNPC;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;
use crate::stage::Stage;

impl NPC {
    pub fn tick_n042_sue(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
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

                    self.parent_id = npc_list
                        .iter_alive()
                        .find_map(|npc| if npc.event_num == 501 { Some(npc.id) } else { None })
                        .unwrap_or(0);
                }

                if let Some(npc) = self.get_parent_ref_mut(npc_list) {
                    self.direction = npc.direction.opposite();
                    self.x = npc.x + npc.direction.vector_x() * 0xc00;
                    self.y = npc.y + 0x800;

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
                    npc.x = self.x + 0x10000;
                    npc.y = self.y;
                    npc.direction = Direction::Left;
                    npc.cond.set_alive(true);
                    let _ = npc_list.spawn(0, npc.clone());

                    npc.direction = Direction::Right;
                    let _ = npc_list.spawn(0x80, npc);
                }

                state.npc_super_pos = (self.x - 0x3000, self.y - 0x1000);
            }
            17 => {
                self.vel_x = 0;
                self.anim_num = 12;

                state.npc_super_pos = (self.x, self.y - 0x1000);
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
                if self.x < player.x - 0x1000 {
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
            _ => (),
        }

        if self.action_num != 14 {
            self.vel_y += 0x40;

            self.vel_x = self.vel_x.clamp(-0x400, 0x400);
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

                    self.x -= 0x800;
                    self.y += 0x2000;
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
            _ => (),
        }

        self.anim_rect = state.constants.npc.n092_sue_at_pc[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n280_sue_teleported(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_counter = 0;
                    self.x += 0xc00;
                    self.target_x = self.x;
                }

                self.action_counter += 1;
                if self.action_counter > 64 {
                    self.action_num = 2;
                    self.action_counter = 0;
                }
            }
            2 => {
                self.anim_num = 0;
                if self.flags.hit_bottom_wall() {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    state.sound_manager.play_sfx(23);
                }
            }
            _ => (),
        }

        if self.action_num > 1 {
            self.vel_y += 0x20;
            if self.vel_y > 0x5FF {
                self.vel_y = 0x5FF;
            }

            self.y += self.vel_y;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n280_sue_teleported[self.anim_num as usize + dir_offset];

        if self.action_num == 1 {
            self.anim_rect.bottom = self.anim_rect.top + self.action_counter / 4;

            self.x = if self.action_counter & 2 != 0 { self.target_x } else { self.target_x + 0x200 };
        }

        Ok(())
    }

    pub(crate) fn tick_n284_sue_possessed(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        stage: &mut Stage,
        boss: &mut BossNPC,
    ) -> GameResult {
        if self.action_num < 100 && (!boss.parts[0].cond.alive() || self.life < 400) {
            self.action_num = 100;
        }

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.y -= 0x800;
                    self.action_counter3 = self.life;

                    state.sound_manager.play_sfx(29);
                }

                self.action_counter += 1;
                if self.action_counter & 2 != 0 {
                    self.display_bounds.top = 0x2000;
                    self.display_bounds.right = 0x2000;
                    self.display_bounds.left = 0x2000;
                    self.anim_num = 11;
                } else {
                    self.display_bounds.top = 0x600;
                    self.display_bounds.right = 0x1000;
                    self.display_bounds.left = 0x1000;
                    self.anim_num = 12;
                }

                if self.action_counter > 50 {
                    self.action_num = 10;
                }
            }
            10 => {
                self.action_num = 11;
                self.anim_num = 11;
                self.display_bounds.top = 0x2000;
                self.display_bounds.right = 0x2000;
                self.display_bounds.left = 0x2000;

                npc_list.kill_npcs_by_type(257, true, state);
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.damage = 0;
                    self.npc_flags.set_shootable(true);
                    self.npc_flags.set_ignore_solidity(false);
                }

                self.vel_x = 7 * self.vel_x / 8;
                self.vel_y = 7 * self.vel_y / 8;

                self.animate(20, 0, 1);

                self.action_counter += 1;
                if self.action_counter > 80 {
                    self.action_num = 30;
                }

                let player = self.get_closest_player_ref(&players);

                self.direction = if player.x > self.x { Direction::Left } else { Direction::Right };

                if self.life + 50 < self.action_counter3 {
                    self.action_counter3 = self.life;
                    state.npc_super_pos.0 = 10;
                }
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.vel_x = 0;
                    self.vel_y = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_counter2 += 1;
                    self.action_counter2 &= 3;

                    self.action_num = match self.action_counter2 {
                        0 | 2 => 32,
                        1 | 3 => 34,
                        _ => self.action_num,
                    };
                }
            }
            32 | 33 => {
                let player = self.get_closest_player_ref(&players);

                if self.action_num == 32 {
                    self.action_num = 33;
                    self.action_counter = 0;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_ignore_solidity(false);
                    self.target_x = if player.x >= self.x { player.x + 0x14000 } else { player.x - 0x14000 };
                    self.target_y = player.y;

                    let angle = f64::atan2((self.y - self.target_y) as f64, (self.x - self.target_x) as f64);

                    self.vel_x = (-1536.0 * angle.cos()) as i32;
                    self.vel_y = (-1536.0 * angle.sin()) as i32;

                    let half_w = stage.map.width as i32 * state.tile_size.as_int() * 0x200 / 2;
                    let half_h = stage.map.height as i32 * state.tile_size.as_int() * 0x200 / 2;

                    if ((self.x < half_w && self.vel_x > 0) || (self.x > half_w && self.vel_x < 0))
                        || ((self.y < half_h && self.vel_y > 0) || (self.y > half_h && self.vel_y < 0))
                    {
                        self.npc_flags.set_ignore_solidity(true);
                    }

                    self.direction = if self.vel_x <= 0 { Direction::Left } else { Direction::Right };
                }

                self.action_counter += 1;
                self.anim_num = if self.action_counter & 2 != 0 { 3 } else { 8 };

                if self.action_counter > 50 || (self.flags.hit_right_wall() || self.flags.hit_left_wall()) {
                    self.action_num = 20;
                }
            }
            34 | 35 => {
                let player = self.get_closest_player_ref(&players);

                if self.action_num == 34 {
                    self.action_num = 35;
                    self.action_counter = 0;
                    self.damage = 4;
                    self.npc_flags.set_ignore_solidity(false);
                    self.target_x = player.x;
                    self.target_y = player.y;

                    let angle = f64::atan2((self.y - self.target_y) as f64, (self.x - self.target_x) as f64);

                    self.vel_x = (-1536.0 * angle.cos()) as i32;
                    self.vel_y = (-1536.0 * angle.sin()) as i32;

                    let half_w = stage.map.width as i32 * state.tile_size.as_int() * 0x200 / 2;
                    let half_h = stage.map.height as i32 * state.tile_size.as_int() * 0x200 / 2;

                    if ((self.x < half_w && self.vel_x > 0) || (self.x > half_w && self.vel_x < 0))
                        || ((self.y < half_h && self.vel_y > 0) || (self.y > half_h && self.vel_y < 0))
                    {
                        self.npc_flags.set_ignore_solidity(true);
                    }

                    self.direction = if self.vel_x <= 0 { Direction::Left } else { Direction::Right };
                }

                self.action_counter += 1;
                if self.action_counter > 20 && self.shock != 0 {
                    self.action_num = 40;
                } else if self.action_counter > 50 || (self.flags.hit_right_wall() || self.flags.hit_left_wall()) {
                    self.action_num = 20;
                }

                self.animate(1, 4, 7);

                if self.action_counter % 5 == 1 {
                    state.sound_manager.play_sfx(109);
                }
            }
            40 | 41 => {
                if self.action_num == 40 {
                    self.action_num = 41;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.damage = 0;
                    self.npc_flags.set_ignore_solidity(false);
                }

                self.vel_x = 7 * self.vel_x / 8;
                self.vel_y = 7 * self.vel_y / 8;

                self.action_counter += 1;
                if self.action_counter > 6 {
                    self.action_num = 42;
                    self.action_counter = 0;

                    self.vel_x = self.direction.vector_x() * 0x200;
                    self.vel_y = -0x200;
                }
            }
            42 => {
                self.anim_num = 9;
                if self.flags.hit_bottom_wall() {
                    self.action_num = 43;
                    self.action_counter = 0;
                    self.anim_num = 2;

                    let player = self.get_closest_player_ref(&players);
                    self.direction = if self.x < player.x { Direction::Right } else { Direction::Left };
                }

                self.vel_y += 0x20;
                if self.vel_y > 0x5FF {
                    self.vel_y = 0x5FF;
                }
            }
            43 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_num = 20;
                }
            }
            99 => {
                self.vel_x = 0;
                self.vel_y = 0;
                self.anim_num = 9;
                self.npc_flags.set_shootable(false);
            }
            100 | 101 => {
                if self.action_num == 100 {
                    self.action_num = 101;
                    self.anim_num = 9;
                    self.damage = 0;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_ignore_solidity(true);
                    self.shock += 50;
                    boss.parts[0].anim_num += 1;
                }

                self.vel_y += 0x20;
                if self.y > 0x1B000 - self.hit_bounds.bottom as i32 {
                    self.y = 0x1B000 - self.hit_bounds.bottom as i32;
                    self.action_num = 102;
                    self.anim_num = 10;
                    self.vel_x = 0;
                    self.vel_y = 0;
                }
            }
            _ => (),
        }

        self.x += if self.shock > 0 { self.vel_x / 2 } else { self.vel_x };
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 13 };

        self.anim_rect = state.constants.npc.n284_sue_possessed[self.anim_num as usize + dir_offset];

        Ok(())
    }
}
