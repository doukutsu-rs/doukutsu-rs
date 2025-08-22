use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::{NPCContext, NPC};
use crate::game::shared_game_state::SharedGameState;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n060_toroko(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, .. }: NPCContext,
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

                let player = self.get_closest_player_mut(players);
                if (self.x - (0x2000) < player.x)
                    && (self.x + (0x2000) > player.x)
                    && (self.y - (0x2000) < player.y)
                    && (self.y + (0x2000) > player.y)
                {
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
            _ => (),
        }

        self.vel_y += 0x40;
        self.vel_x = self.vel_x.clamp(-0x400, 0x400);

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.direction == Direction::Left {
            self.anim_rect = state.constants.npc.n060_toroko[self.anim_num as usize];
        } else {
            self.anim_rect = state.constants.npc.n060_toroko[self.anim_num as usize + 8];
        }

        Ok(())
    }

    pub(crate) fn tick_n063_toroko_stick(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
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

                if self.action_counter != 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 2;
                }
                self.action_counter += 1;
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

                if self.action_counter != 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 5;
                    self.npc_flags.set_interactable(true);
                }
                self.action_counter += 1;
            }
            5 => {
                self.vel_x = 0;
                self.anim_num = 5;
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.vel_x = self.vel_x.clamp(-0x400, 0x400);

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.direction == Direction::Left {
            self.anim_rect = state.constants.npc.n063_toroko_stick[self.anim_num as usize];
        } else {
            self.anim_rect = state.constants.npc.n063_toroko_stick[self.anim_num as usize + 6];
        }

        Ok(())
    }

    pub(crate) fn tick_n140_toroko_frenzied(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, bullet_manager, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 9;
                    self.action_counter = 0;
                    self.npc_flags.set_interactable(false);
                }

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 8;
                }
            }
            2 => {
                self.anim_num += 1;
                if self.anim_num > 10 {
                    self.anim_num = 9;
                }
                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 10;
                    self.npc_flags.set_shootable(true);
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.action_counter = self.rng.range(20..130) as u16;
                    self.vel_x = 0;
                }

                let player = self.get_closest_player_mut(players);
                if self.x <= player.x {
                    self.direction = Direction::Right;
                } else {
                    self.direction = Direction::Left;
                }
                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }
                if self.anim_num > 1 {
                    self.anim_num = 0;
                }
                if bullet_manager.count_bullets_type_idx_all(6) != 0 || bullet_manager.count_bullets_type_idx_all(3) > 3
                {
                    self.action_num = 20;
                }
                if self.action_counter != 0 {
                    self.action_counter -= 1;
                } else if (self.rng.range(0..99) & 1) != 0 {
                    self.action_num = 20;
                } else {
                    self.action_num = 50;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.anim_num = 2;
                    self.action_counter = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 22;
                    self.action_counter = 0;
                    self.anim_num = 3;
                    self.vel_y = -0x5ff;
                    if self.direction != Direction::Left {
                        self.vel_x = 0x200;
                    } else {
                        self.vel_x = -0x200;
                    }
                }
            }
            22 => {
                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 23;
                    self.action_counter = 0;
                    self.anim_num = 6;

                    let mut npc = NPC::create(141, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.parent_id = self.id;

                    let _ = npc_list.spawn(0, npc);
                }
            }
            23 => {
                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 24;
                    self.action_counter = 0;
                    self.anim_num = 7;
                }

                let player = self.get_closest_player_mut(players);
                if self.x <= player.x {
                    self.direction = Direction::Right;
                } else {
                    self.direction = Direction::Left;
                }
            }
            24 => {
                self.action_counter += 1;
                if self.action_counter > 3 {
                    self.action_num = 25;
                    self.anim_num = 3;
                }
            }
            25 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 26;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 20;
                    state.quake_rumble_counter = 20;
                }
            }
            26 => {
                self.vel_x = 8 * self.vel_x / 9;
                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 10;
                    self.anim_num = 0;
                }
            }
            50 | 51 => {
                if self.action_num == 50 {
                    self.action_num = 51;
                    self.action_counter = 0;
                    self.anim_num = 4;

                    let mut npc = NPC::create(141, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.parent_id = self.id;

                    let _ = npc_list.spawn(0, npc);
                }
                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 52;
                    self.action_counter = 0;
                    self.anim_num = 5;
                }

                let player = self.get_closest_player_mut(players);
                if self.x <= player.x {
                    self.direction = Direction::Right;
                } else {
                    self.direction = Direction::Left;
                }
            }

            52 => {
                self.action_counter += 1;
                if self.action_counter > 3 {
                    self.action_num = 10;
                    self.anim_num = 0;
                }
            }
            100 => {
                self.anim_num = 3;
                self.action_num = 101;
                self.npc_flags.set_shootable(false);
                self.damage = 0;

                let mut npc = NPC::create(4, &state.npc_table);
                npc.cond.set_alive(true);

                for _ in 0..8 {
                    npc.x = self.x + self.rng.range(-12..12) * 0x200;
                    npc.y = self.y + self.rng.range(-12..12) * 0x200;
                    npc.vel_y = self.rng.range(-0x600..0);
                    npc.vel_x = self.rng.range(-0x155..0x155);

                    let _ = npc_list.spawn(0x100, npc.clone());
                }
            }
            101 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 102;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 20;
                    state.quake_rumble_counter = 20;
                }
            }
            102 => {
                self.vel_x = 8 * self.vel_x / 9;
                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 103;
                    self.action_counter = 0;
                    self.anim_num = 10;
                }
            }
            103 => {
                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.anim_num = 9;
                    self.action_num = 104;
                    self.action_counter = 0;
                }
            }
            104 => {
                self.anim_num += 1;
                if self.anim_num > 10 {
                    self.anim_num = 9;
                }
                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_counter = 0;
                    self.anim_num = 9;
                    self.action_num = 105;
                }
            }
            105 => {
                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.anim_counter = 0;
                    self.action_num = 106;
                    self.anim_num = 11;
                }
            }
            106 => {
                self.anim_counter += 1;
                if self.anim_counter > 50 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }
                if self.anim_num > 12 {
                    self.anim_num = 12;
                }
            }
            140 | 141 => {
                if self.action_num == 140 {
                    self.action_num = 141;
                    self.action_counter = 0;
                    self.anim_num = 12;
                    state.sound_manager.play_sfx(29);
                }

                self.anim_num += 1;
                if self.anim_num > 13 {
                    self.anim_num = 12;
                }

                self.action_counter += 1;
                if self.action_counter > 100 {
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    for _ in 0..4 {
                        npc.x = self.x + self.rng.range(-12..12) * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) * 0x200;
                        npc.vel_y = self.rng.range(-0x600..0);
                        npc.vel_x = self.rng.range(-0x155..0x155);

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        if self.action_num > 100 && self.action_num <= 104 && (self.action_counter % 9) == 0 {
            let mut npc = NPC::create(4, &state.npc_table);
            npc.cond.set_alive(true);
            npc.x = self.x + self.rng.range(-12..12) * 0x200;
            npc.y = self.y + self.rng.range(-12..12) * 0x200;
            npc.vel_y = self.rng.range(-0x600..0);
            npc.vel_x = self.rng.range(-0x155..0x155);

            let _ = npc_list.spawn(0x100, npc);
        }

        self.vel_y += 0x20;
        self.vel_y = self.vel_y.clamp(-0x5ff, 0x5ff);
        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 14 };

        self.anim_rect = state.constants.npc.n140_toroko_frenzied[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n141_toroko_block_projectile(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                }
                let parent = self.get_parent(npc_list);
                if let Some(parent) = parent {
                    let player = self.get_closest_player_mut(players);

                    if parent.direction == Direction::Left {
                        self.x = parent.x + 0x1400;
                    } else {
                        self.x = parent.x + -0x1400;
                    }

                    self.y = parent.y + -0x1000;
                    if (parent.action_num == 0x18) || (parent.action_num == 0x34) {
                        self.action_num = 10;
                        if parent.direction == Direction::Left {
                            self.x = parent.x + -0x2000;
                        } else {
                            self.x = parent.x + 0x2000;
                        }
                        self.y = parent.y;

                        let angle = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64);

                        self.vel_x = (angle.cos() * -2048.0) as i32;
                        self.vel_y = (angle.sin() * -2048.0) as i32;
                        state.sound_manager.play_sfx(0x27);
                    }
                }
            }
            10 => {
                if (self.flags.0 & 0xf) == 0 {
                    self.x += self.vel_x;
                    self.y += self.vel_y;
                } else {
                    self.action_num = 0x14;
                    self.action_counter = 0;

                    state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
                    state.sound_manager.play_sfx(0xc);

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    for _ in 0..4 {
                        npc.x = self.x;
                        npc.y = self.y;
                        npc.vel_x = self.rng.range(-0x200..0x200);
                        npc.vel_y = self.rng.range(-0x200..0x200);

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }
            }
            20 => {
                self.x += self.vel_x;
                self.y += self.vel_y;
                self.action_counter += 1;
                if 4 < self.action_counter {
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    for _ in 0..4 {
                        npc.x = self.x;
                        npc.y = self.y;
                        npc.vel_x = self.rng.range(-0x200..0x200);
                        npc.vel_y = self.rng.range(-0x200..0x200);

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }

                    self.npc_type = 0x8e;
                    self.anim_num = 0;
                    self.action_num = 0x14;
                    self.vel_x = 0;
                    self.npc_flags.set_invulnerable(false);
                    self.npc_flags.set_shootable(true);
                    self.damage = 1;
                }
            }
            _ => (),
        }

        self.anim_num += 1;
        if 1 < self.anim_num {
            self.anim_num = 0;
        }

        self.anim_rect = state.constants.npc.n141_toroko_block_projectile[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n142_flower_cub(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 0;
                    self.action_counter = 0;
                }
                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 12;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }
            }
            12 => {
                self.anim_counter += 1;
                if self.anim_counter > 8 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }
                if self.anim_num == 3 {
                    self.action_num = 20;
                    self.vel_y = -0x200;
                    let player = self.get_closest_player_mut(players);
                    if player.x >= self.x {
                        self.vel_x = 0x200;
                    } else {
                        self.vel_x = -0x200;
                    }
                }
            }
            20 => {
                if self.vel_y < -127 {
                    self.anim_num = 3;
                } else {
                    self.anim_num = 4;
                }

                if self.flags.hit_bottom_wall() {
                    self.anim_num = 2;
                    self.action_num = 21;
                    self.action_counter = 0;
                    self.vel_x = 0;
                    state.sound_manager.play_sfx(23);
                }
            }
            21 => {
                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 10;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }
        self.vel_y += 64;
        self.vel_y = self.vel_y.clamp(-0x5ff, 0x5ff);

        self.x += self.vel_x;
        self.y += self.vel_y;
        self.anim_rect = state.constants.npc.n142_flower_cub[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n144_toroko_teleporting_in(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.target_x = self.x;
                    state.sound_manager.play_sfx(29);
                }
                self.action_counter += 1;
                if self.action_counter == 64 {
                    self.action_num = 2;
                    self.action_counter = 0;
                }
            }
            2 => {
                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }
                if self.anim_num > 3 {
                    self.anim_num = 2;
                }
                if self.flags.hit_bottom_wall() {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.anim_num = 4;
                    state.sound_manager.play_sfx(23);
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }
                if self.rng.range(0..120) == 10 {
                    self.action_num = 12;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            12 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 11;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        if self.action_num > 1 {
            self.vel_y += 0x20;
            self.clamp_fall_speed();
            self.y += self.vel_y;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 5 };

        self.anim_rect = state.constants.npc.n144_toroko_teleporting_in[self.anim_num as usize + dir_offset];

        if self.action_num == 1 {
            self.anim_rect.bottom = self.action_counter / 4 + self.anim_rect.top;
            self.x = if ((self.action_counter / 2) & 1) != 0 { self.target_x } else { self.target_x + 512 }
        }

        Ok(())
    }
}
