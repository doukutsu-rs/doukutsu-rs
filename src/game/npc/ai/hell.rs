use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::Stage;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n337_numahachi(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y -= 0x1000;
        }

        if self.action_num == 1 {
            self.action_num = 2;
            self.anim_num = 0;
            self.vel_x = 0;
        }

        if self.action_num == 2 {
            self.animate(50, 0, 1);
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n337_numahachi[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n357_puppy_ghost(&mut self, state: &mut SharedGameState) -> GameResult {
        self.anim_rect = state.constants.npc.n357_puppy_ghost;

        match self.action_num {
            0 => {
                self.action_counter += 1;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    state.sound_manager.play_sfx(29);
                }

                self.action_counter += 1;

                if self.action_counter & 2 != 0 {
                    self.anim_rect.right = self.anim_rect.left;
                }

                if self.action_counter > 50 {
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        if self.action_counter % 8 == 1 {
            state.create_caret(
                self.x + self.rng.range(-8..8) * 0x200,
                self.y + 0x1000,
                CaretType::LittleParticles,
                Direction::Up,
            );
        }

        Ok(())
    }

    pub(crate) fn tick_n309_bute(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                self.action_num = 1;

                if (self.direction == Direction::Left && player.x > self.x - 0x24000 && player.x < self.x - 0x22000)
                    || (self.direction != Direction::Left && player.x < self.x + 0x24000 && player.x > self.x + 0x22000)
                {
                    self.action_num = 10;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.npc_flags.set_shootable(true);
                    self.damage = 5;
                }

                self.face_player(player);
                self.vel_x2 += 0x10 * self.direction.vector_x();
                self.vel_y2 += 0x10 * if self.y > player.y { -1 } else { 1 };

                if self.vel_x2 < 0 && self.flags.hit_left_wall() {
                    self.vel_x2 *= -1
                };
                if self.vel_x2 > 0 && self.flags.hit_right_wall() {
                    self.vel_x2 *= -1
                };
                if self.vel_y2 < 0 && self.flags.hit_top_wall() {
                    self.vel_y2 *= -1
                };
                if self.vel_y2 > 0 && self.flags.hit_bottom_wall() {
                    self.vel_y2 *= -1
                };

                self.vel_x2 = self.vel_x2.clamp(-0x5FF, 0x5FF);
                self.vel_y2 = self.vel_y2.clamp(-0x5FF, 0x5FF);

                self.x += self.vel_x2;
                self.y += self.vel_y2;

                self.animate(1, 0, 1);
                let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };
                self.anim_rect = state.constants.npc.n309_bute[self.anim_num as usize + dir_offset];
            }
            _ => (),
        }

        if self.life <= 996 {
            self.npc_type = 316;
            self.action_num = 0;
        }

        Ok(())
    }

    pub(crate) fn tick_n310_bute_sword(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.damage = 0;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_invulnerable(true);
                }

                self.face_player(player);
                self.anim_counter = 0;

                if player.x > self.x - 0x10000
                    && player.x < self.x + 0x10000
                    && player.y > self.y - 0x10000
                    && player.y < self.y + 0x2000
                {
                    self.action_num = 10;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.vel_x = 0;
                    self.action_counter = 0;
                    self.damage = 0;
                    self.anim_counter = 0;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_invulnerable(true);
                }

                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 20;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                    self.damage = 0;
                    self.npc_flags.set_shootable(true);
                    self.npc_flags.set_invulnerable(false);
                    self.face_player(player);
                }

                self.vel_x = 0x400 * self.direction.vector_x();

                self.animate(3, 0, 1);

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 10;
                }

                if self.x < player.x + 0x5000 && self.x > player.x - 0x5000 {
                    self.vel_y = -0x300;
                    self.vel_x /= 2;
                    self.anim_num = 2;
                    self.action_num = 30;
                    state.sound_manager.play_sfx(30);
                }
            }
            30 => {
                if self.vel_y > -0x80 {
                    self.action_num = 31;
                    self.anim_counter = 0;
                    self.anim_num = 3;
                    self.damage = 9;
                }
            }
            31 => {
                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num = 4;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 32;
                    self.action_counter = 0;
                    self.vel_x = 0;
                    self.damage = 3;
                }
            }
            32 => {
                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 10;
                    self.damage = 0;
                }
            }
            _ => (),
        }

        self.vel_y += 0x20;

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 5 };
        self.anim_rect = state.constants.npc.n310_bute_sword[self.anim_num as usize + dir_offset];

        if self.life <= 996 {
            self.npc_type = 316;
            self.action_num = 0;
        }

        Ok(())
    }

    pub(crate) fn tick_n311_bute_archer(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                self.action_num = 1;

                if (player.y > self.y - 0x14000 && player.y < self.y + 0x14000)
                    && ((self.direction == Direction::Left && player.x > self.x - 0x28000 && player.x < self.x)
                    || (self.direction != Direction::Left && player.x > self.x && player.x < self.x + 0x28000))
                {
                    self.action_num = 10;
                }
            }
            10 | 11 => {
                self.action_num = 11;

                self.face_player(player);

                if player.x > self.x - 0x1C000 && player.x < self.x + 0x1C000 && player.y > self.y - 0x1000 {
                    self.anim_num = 1;
                    self.action_counter2 = 0;
                } else {
                    self.anim_num = 4;
                    self.action_counter2 = 1;
                }

                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 20;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                }

                self.animate(1, 1 + self.action_counter2 * 3, 2 + self.action_counter2 * 3);

                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 30;
                }
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 0;

                    let mut npc = NPC::create(312, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.vel_x = 0x600 * self.direction.vector_x();
                    npc.vel_y = self.action_counter2 as i32 * -0x600;
                    npc.direction = self.direction;

                    let _ = npc_list.spawn(0x100, npc);

                    self.anim_num = 3 + self.action_counter2 * 3;
                }

                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 40;
                    self.action_counter = self.rng.range(0..100) as u16;
                }
            }
            40 => {
                self.anim_num = 0;

                self.action_counter += 1;
                if self.action_counter > 150 {
                    self.action_num = 10;
                }

                if player.x < self.x - 0x2C000
                    || player.x > self.x + 0x2C000
                    || player.y < self.y - 0x1E000
                    || player.y > self.y + 0x1E000
                {
                    self.action_num = 40;
                    self.action_counter = 0;
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 7 };
        self.anim_rect = state.constants.npc.n311_bute_archer[self.anim_num as usize + dir_offset];

        if self.life <= 992 {
            self.npc_type = 316;
            self.action_num = 0;
        }

        Ok(())
    }

    pub(crate) fn tick_n312_bute_arrow_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.flags.hit_anything() && self.action_num > 0 && self.action_num < 20 {
            self.action_num = 20;
        }

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;

                    self.direction = if self.vel_x < 0 { Direction::Left } else { Direction::Right };
                    self.anim_num = if self.vel_y < 0 { 0 } else { 2 };
                }

                self.action_counter += 1;

                if self.action_counter == 4 {
                    self.npc_flags.set_ignore_solidity(false)
                };
                if self.action_counter > 10 {
                    self.action_num = 10
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_counter = 0;
                    self.vel_x = 3 * self.vel_x / 4;
                    self.vel_y = 3 * self.vel_y / 4;
                }

                self.vel_y += 32;

                self.animate(10, 4, 4);
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.damage = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 30
                }
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.cond.set_alive(false);
                    return Ok(());
                }
            }
            _ => (),
        }

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 5 };
        self.anim_rect = state.constants.npc.n312_bute_arrow_projectile[self.anim_num as usize + dir_offset];

        if self.action_num == 31 && self.action_counter & 0x02 != 0 {
            self.anim_rect.left = 0;
            self.anim_rect.right = 0;
        }

        Ok(())
    }

    pub(crate) fn tick_n316_bute_dead(&mut self, state: &mut SharedGameState) -> GameResult {
        // (nearly) same as Gaudi death
        match self.action_num {
            0 => {
                self.npc_flags.set_shootable(false);
                self.npc_flags.set_ignore_solidity(false);
                self.damage = 0;
                self.action_num = 1;
                self.anim_num = 0;
                self.display_bounds.top = 0x1800;
                self.display_bounds.right = 0x1800;
                self.display_bounds.left = 0x1800;
                self.vel_y = -0x200;
                self.vel_x = 0x100 * self.direction.opposite().vector_x();
                state.sound_manager.play_sfx(50);
            }
            1 if self.flags.hit_bottom_wall() => {
                self.action_num = 2;
                self.action_counter = 0;
                self.anim_num = 1;
                self.anim_counter = 0;
            }
            2 => {
                self.vel_x = 8 * self.vel_x / 9;
                self.animate(3, 1, 2);

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.cond.set_explode_die(true);
                }
            }
            _ => (),
        }

        self.vel_y += 0x20;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n316_bute_dead[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n323_bute_spinning(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        if self.action_num == 0 || self.action_num == 1 {
            if self.action_num == 0 {
                self.action_num = 1;

                self.vel_x = 0x600 * self.direction.vector_x();
                self.vel_y = 0x600 * self.direction.vector_y();
            }

            self.action_counter += 1;
            if self.action_counter == 16 {
                self.npc_flags.set_ignore_solidity(false)
            };

            self.x += self.vel_x;
            self.y += self.vel_y;

            if self.flags.hit_anything() {
                self.action_num = 10
            };

            let player = self.get_closest_player_ref(&players);
            if self.action_counter > 20
                && ((self.direction == Direction::Left && self.x <= player.x + 0x4000)
                || (self.direction == Direction::Up && self.y <= player.y + 0x4000)
                || (self.direction == Direction::Right && self.x >= player.x - 0x4000)
                || (self.direction == Direction::Bottom && self.y >= player.y - 0x4000))
            {
                self.action_num = 10
            }
        }
        if self.action_num == 10 {
            self.npc_type = 309;
            self.anim_num = 0;
            self.action_num = 11;
            self.npc_flags.set_shootable(true);
            self.npc_flags.set_ignore_solidity(false);
            self.damage = 5;
            self.display_bounds.top = 0x1000;
        }

        self.animate(3, 0, 3);
        self.anim_rect = state.constants.npc.n323_bute_spinning[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n324_bute_generator(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if self.action_num == 10 {
            self.action_num = 11;
            self.action_counter = 0;
        }

        if self.action_num == 11 {
            self.action_counter += 1;
            if self.action_counter % 50 == 1 {
                let mut npc = NPC::create(323, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;
                npc.y = self.y;
                npc.direction = self.direction;

                let _ = npc_list.spawn(0x100, npc);
            }

            if self.action_counter > 351 {
                self.action_num = 0
            };
        }

        Ok(())
    }

    pub(crate) fn tick_n317_mesa(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        let player = self.get_closest_player_ref(&players);

        match self.action_num {
            0 | 1 | 2 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.y -= 0x1000;
                    self.target_x = self.x;
                }
                if self.action_num == 1 {
                    self.action_num = 2;
                    self.vel_x = 0;
                    self.anim_num = 0;
                    self.action_counter = 0;
                }
                self.face_player(player);
                self.animate(40, 0, 1);

                if player.x > self.x - 0x28000
                    && player.x < self.x + 0x28000
                    && player.y > self.y - 0x14000
                    && player.y < self.y + 0x14000
                {
                    self.action_counter += 1;
                    if self.action_counter > 50 {
                        self.action_num = 10
                    }
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 2;

                    let mut npc = NPC::create(319, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.parent_id = self.id;

                    let _ = npc_list.spawn(0x100, npc);
                }

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_counter = 0;
                    self.action_num = 12;
                    self.anim_num = 3;
                    state.sound_manager.play_sfx(39);
                }
            }
            12 => {
                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 1
                }
            }
            _ => (),
        }

        self.vel_y += 0x55;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };
        self.anim_rect = state.constants.npc.n317_mesa[self.anim_num as usize + dir_offset];

        if self.life <= 936 {
            self.npc_type = 318;
            self.action_num = 0;
        }

        Ok(())
    }

    pub(crate) fn tick_n318_mesa_dead(&mut self, state: &mut SharedGameState) -> GameResult {
        // (nearly) same as Gaudi death
        match self.action_num {
            0 => {
                self.npc_flags.set_shootable(false);
                self.npc_flags.set_ignore_solidity(false);
                self.npc_flags.set_solid_soft(false);
                self.damage = 0;
                self.action_num = 1;
                self.anim_num = 0;
                self.vel_y = -0x200;
                self.vel_x = 0x40 * self.direction.opposite().vector_x();
                state.sound_manager.play_sfx(54);
            }
            1 if self.flags.hit_bottom_wall() => {
                self.action_num = 2;
                self.action_counter = 0;
                self.anim_num = 1;
                self.anim_counter = 0;
            }
            2 => {
                self.vel_x = 8 * self.vel_x / 9;
                self.animate(3, 1, 2);

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.cond.set_explode_die(true);
                }
            }
            _ => (),
        }

        self.vel_y += 0x20;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n318_mesa_dead[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n319_mesa_block(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 => {
                if let Some(parent) = self.get_parent_ref_mut(npc_list) {
                    let parent = parent.borrow();
                    
                    self.y = parent.y + 0x1400;
                    self.x = parent.x + 0xE00 * parent.direction.opposite().vector_x();

                    if parent.npc_type == 318 {
                        npc_list.create_death_smoke(self.x, self.y, 0, 3, state, &mut self.rng);
                        self.cond.set_alive(false);
                    }

                    if parent.anim_num != 2 {
                        self.action_num = 2;
                        self.action_counter = 0;
                        self.vel_y = -0x400;
                        self.y = parent.y - 0x800;
                        self.vel_x = 0x400 * parent.direction.vector_x();
                    }
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter == 4 {
                    self.npc_flags.set_ignore_solidity(false)
                };

                self.vel_y += 0x2A;
                self.clamp_fall_speed();

                self.x += self.vel_x;
                self.y += self.vel_y;

                if self.flags.hit_bottom_wall() {
                    state.sound_manager.play_sfx(12);
                    npc_list.create_death_smoke(self.x, self.y, 0, 3, state, &mut self.rng);
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        self.animate(0, 0, 2);
        self.anim_rect = state.constants.npc.n319_mesa_block[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n322_deleet(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
        stage: &mut Stage,
    ) -> GameResult {
        if self.action_num < 2 && self.life <= 968 {
            self.action_num = 2;
            self.action_counter = 0;
            self.npc_flags.set_shootable(false);
            self.npc_flags.set_invulnerable(true);
            state.sound_manager.play_sfx(22);
        }

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    if self.direction == Direction::Left {
                        self.y += 0x1000
                    } else {
                        self.x += 0x1000
                    };
                }

                if self.shock > 0 {
                    self.anim_counter += 1
                } else {
                    self.anim_counter = 0
                };
                self.anim_num = self.anim_counter & 0x02;
            }
            2 => {
                self.anim_num = 2;

                let mut npc = NPC::create(207, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x + 0x800;
                npc.y = self.y;

                match self.action_counter {
                    0 => {
                        npc.tsc_direction = 0;
                        let _ = npc_list.spawn(0x180, npc);
                    }
                    50 => {
                        npc.tsc_direction = 1;
                        let _ = npc_list.spawn(0x180, npc);
                    }
                    100 => {
                        npc.tsc_direction = 2;
                        let _ = npc_list.spawn(0x180, npc);
                    }
                    150 => {
                        npc.tsc_direction = 3;
                        let _ = npc_list.spawn(0x180, npc);
                    }
                    200 => {
                        npc.tsc_direction = 4;
                        let _ = npc_list.spawn(0x180, npc);
                    }
                    250 => {
                        self.hit_bounds.right = 0x6000;
                        self.hit_bounds.left = 0x6000;
                        self.hit_bounds.top = 0x6000;
                        self.hit_bounds.bottom = 0x6000;
                        self.damage = 12;
                        self.cond.set_explode_die(true);

                        state.quake_counter = 10;
                        state.quake_rumble_counter = 10;
                        state.sound_manager.play_sfx(26);
                        npc_list.create_death_smoke(self.x, self.y, 0x6000, 40 as usize, state, &mut self.rng);

                        let x = (self.x / (state.tile_size.as_int() * 0x100)) as usize;
                        let y = (self.y / (state.tile_size.as_int() * 0x100)) as usize;

                        let mut change_tile_with_smoke = |x, y| {
                            if stage.change_tile(x, y, 0) {
                                let mut npc = NPC::create(4, &state.npc_table);
                                npc.cond.set_alive(true);
                                npc.x = (x as i32) * state.tile_size.as_int() * 0x200;
                                npc.y = (y as i32) * state.tile_size.as_int() * 0x200;
                                let _ = npc_list.spawn(0, npc.clone());
                                let _ = npc_list.spawn(0, npc.clone());
                                let _ = npc_list.spawn(0, npc);
                            }
                        };
                        if self.direction == Direction::Left {
                            change_tile_with_smoke(x / 2, (y + 1) / 2);
                            change_tile_with_smoke(x / 2, (y - 1) / 2);
                        } else {
                            change_tile_with_smoke((x + 1) / 2, y / 2);
                            change_tile_with_smoke((x - 1) / 2, y / 2);
                        }
                    }
                    _ => (),
                }
                self.action_counter += 1;
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n322_deleet[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n330_rolling(&mut self, state: &mut SharedGameState, npc_list: &NPCList, stage: &mut Stage) -> GameResult {
        match self.action_num {
            0 => {
                let x = (self.x / (state.tile_size.as_int() * 0x200)) as usize;
                let y = (self.y / (state.tile_size.as_int() * 0x200)) as usize;
                if stage.change_tile(x, y, 0) {
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = (x as i32) * state.tile_size.as_int() * 0x200;
                    npc.y = (y as i32) * state.tile_size.as_int() * 0x200;
                    let _ = npc_list.spawn(0, npc.clone());
                    let _ = npc_list.spawn(0, npc.clone());
                    let _ = npc_list.spawn(0, npc);
                }

                self.action_num = if self.direction == Direction::Left { 10 } else { 30 };
            }
            10 => {
                self.vel_x -= 0x40;
                self.vel_y = 0;
                if self.flags.hit_left_wall() {
                    self.action_num = 20
                };
            }
            20 => {
                self.vel_x = 0;
                self.vel_y -= 0x40;
                if self.flags.hit_top_wall() {
                    self.action_num = 30
                };
            }
            30 => {
                self.vel_x += 0x40;
                self.vel_y = 0;
                if self.flags.hit_right_wall() {
                    self.action_num = 40
                };
            }
            40 => {
                self.vel_x = 0;
                self.vel_y += 0x40;
                if self.flags.hit_bottom_wall() {
                    self.action_num = 10
                };
            }
            _ => (),
        }

        self.vel_x = self.vel_x.clamp(-0x400, 0x400);
        self.vel_y = self.vel_y.clamp(-0x400, 0x400);

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.animate(1, 0, 2);
        self.anim_rect = state.constants.npc.n330_rolling[self.anim_num as usize];

        Ok(())
    }
}
