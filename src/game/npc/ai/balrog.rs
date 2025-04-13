use num_traits::clamp;

use crate::common::{Direction, CDEG_RAD};
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::list::BorrowedNPC;
use crate::game::npc::{NPCContext, NPC};
use crate::game::shared_game_state::SharedGameState;
use crate::util::rng::RNG;

impl BorrowedNPC<'_> {
    pub(crate) fn tick_n009_balrog_falling_in(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { npc_list, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 2;
                }

                self.vel_y += 0x20;

                if self.action_counter2 > 39 {
                    self.npc_flags.set_ignore_solidity(false);
                    self.npc_flags.set_solid_soft(true);
                } else {
                    self.action_counter2 += 1;
                }

                if self.flags.hit_bottom_wall() {
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;

                    for _ in 0..3 {
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }

                    self.action_num = 2;
                    self.anim_num = 1;
                    self.action_counter = 0;

                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;
                    state.quake_rumble_counter = 30;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_counter = 3;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }
            }
            _ => (),
        }

        self.vel_y = clamp(self.vel_y, -0x5ff, 0x5ff);

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n009_balrog_falling_in[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n010_balrog_shooting(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                }

                self.action_counter += 1;
                if self.action_counter > 12 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.action_counter2 = 3;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_counter2 -= 1;
                    self.action_counter = 0;

                    let mut npc = NPC::create(11, &state.npc_table);

                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y + 0x800; // 4.0fix9

                    let mut angle = ((self.y + 0x800 - player.y) as f64 / (self.x - player.y) as f64).atan();
                    angle += self.rng.range(-16..16) as f64 * std::f64::consts::FRAC_PI_8;
                    npc.vel_x = (angle.cos() * 512.0) as i32; // 1.0fix9
                    npc.vel_y = (angle.sin() * 512.0) as i32;

                    let _ = npc_list.spawn(0x100, npc);

                    state.sound_manager.play_sfx(39);

                    if self.action_counter2 == 0 {
                        self.action_num = 3;
                        self.action_counter = 0;
                    }
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 3 {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.vel_x = (player.x - self.x) / 0x64;
                    self.vel_y = -0x600;
                    self.anim_num = 3;
                }
            }
            4 => {
                if self.flags.hit_left_wall() || self.flags.hit_right_wall() {
                    self.vel_x = 0;
                }

                self.damage = if self.y + 0x2000 < player.y { 5 } else { 0 };

                if self.flags.hit_bottom_wall() {
                    self.action_num = 5;
                    self.action_counter = 5;
                    self.anim_num = 2;
                    self.damage = 0;

                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;
                    state.quake_rumble_counter = 30;
                }
            }
            5 => {
                self.vel_x = 0;

                self.action_counter += 1;
                if self.action_counter > 3 {
                    self.action_num = 1;
                    self.action_counter = 0;
                }
            }
            _ => (),
        }

        self.vel_y += 0x20;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.npc.n010_balrog_shooting[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n011_balrogs_projectile(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        if self.flags.hit_anything() {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.animate(1, 0, 2);

        self.anim_rect = state.constants.npc.n011_balrog_energy_shot[self.anim_num as usize];

        self.action_counter2 += 1;
        if self.action_counter2 > 150 {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        Ok(())
    }

    pub(crate) fn tick_n012_balrog_cutscene(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, stage, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    if self.direction == Direction::FacingPlayer {
                        let player = self.get_closest_player_mut(players);

                        if self.x <= player.x {
                            self.direction = Direction::Right;
                        } else {
                            self.direction = Direction::Left;
                        }
                    }

                    self.action_num = 1;
                    self.anim_num = 0;
                }

                if self.rng.range(0..100) == 0 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    if self.direction == Direction::FacingPlayer {
                        let player = self.get_closest_player_mut(players);

                        if self.x <= player.x {
                            self.direction = Direction::Right;
                        } else {
                            self.direction = Direction::Left;
                        }
                    }

                    self.action_num = 11;
                    self.anim_num = 2;
                    self.action_counter = 0;
                    self.target_x = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 12;
                    self.action_counter = 0;
                    self.anim_num = 3;
                    self.vel_y = -0x800;
                    self.npc_flags.set_ignore_solidity(true);
                }
            }
            12 => {
                if self.flags.hit_left_wall() || self.flags.hit_right_wall() {
                    self.vel_x = 0;
                }

                if self.y < 0 {
                    self.npc_type = 0;
                    self.spritesheet_id = 20; // NpcSym
                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;
                    state.quake_rumble_counter = 30;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    if self.direction == Direction::FacingPlayer {
                        let player = self.get_closest_player_mut(players);

                        if self.x <= player.x {
                            self.direction = Direction::Right;
                        } else {
                            self.direction = Direction::Left;
                        }
                    }

                    self.action_num = 21;
                    self.anim_num = 5;
                    self.action_counter = 0;
                    self.action_counter2 = 0;

                    let mut npc = NPC::create(4, &state.npc_table);

                    for _ in 0..3 {
                        npc.cond.set_alive(true);
                        npc.direction = Direction::Left;
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }

                    state.sound_manager.play_sfx(72);
                }

                self.target_x = 1; // ???

                if self.flags.hit_bottom_wall() {
                    self.action_counter += 1;
                }

                self.action_counter2 += 1;
                self.x += if self.action_counter2 & 0x02 != 0 { 0x200 } else { -0x200 };

                if self.action_counter > 100 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 2;
                }

                self.vel_y += 0x20;
                self.clamp_fall_speed();
            }
            30 => {
                self.anim_num = 4;
                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 0;
                    self.anim_num = 0;
                }
            }
            40 | 41 => {
                if self.action_num == 40 {
                    if self.direction == Direction::FacingPlayer {
                        let player = self.get_closest_player_mut(players);

                        if self.x <= player.x {
                            self.direction = Direction::Right;
                        } else {
                            self.direction = Direction::Left;
                        }
                    }

                    self.action_num = 41;
                    self.action_counter = 0;
                    self.anim_num = 5;
                }

                self.anim_counter += 1;
                self.anim_num = if self.anim_counter & 0x02 != 0 { 5 } else { 6 };
            }
            42 | 43 => {
                if self.action_num == 42 {
                    if self.direction == Direction::FacingPlayer {
                        let player = self.get_closest_player_mut(players);

                        if self.x <= player.x {
                            self.direction = Direction::Right;
                        } else {
                            self.direction = Direction::Left;
                        }
                    }

                    self.action_num = 43;
                    self.anim_num = 6;
                    self.action_counter = 0;
                }

                self.anim_counter += 1;
                self.anim_num = if self.anim_counter & 0x02 != 0 { 7 } else { 6 };
            }
            50 => {
                self.anim_num = 8;
                self.vel_x = 0;
            }
            60 | 61 => {
                if self.action_num == 60 {
                    self.action_num = 61;
                    self.anim_num = 9;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_counter = 0;

                    self.anim_num += 1;
                    if self.anim_num == 10 || self.anim_num == 11 {
                        state.sound_manager.play_sfx(23);
                    }

                    if self.anim_num > 12 {
                        self.anim_num = 9;
                    }
                }

                self.vel_x = match self.direction {
                    Direction::Left => -0x200,
                    Direction::Right => 0x200,
                    _ => 0,
                };
            }
            70 | 71 => {
                if self.action_num == 70 {
                    self.action_num = 71;
                    self.action_counter = 64;
                    self.anim_num = 13;
                    state.sound_manager.play_sfx(29);
                }

                self.action_counter -= 1;
                if self.action_counter == 0 {
                    self.cond.set_alive(false);
                }
            }
            80 | 81 => {
                if self.action_num == 80 {
                    self.action_num = 81;
                    self.action_counter2 = 0;
                }

                self.action_counter2 += 1;
                self.x += if self.action_counter2 & 0x02 != 0 { 0x200 } else { -0x200 };

                self.anim_num = 5;
                self.vel_x = 0;
                self.vel_y += 0x20;
            }
            100 | 101 => {
                if self.action_num == 100 {
                    self.action_num = 101;
                    self.action_counter = 0;
                    self.anim_num = 2;
                }

                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 102;
                    self.action_counter = 0;
                    self.anim_num = 3;
                    self.vel_y = -0x800;
                    self.npc_flags.set_ignore_solidity(true);

                    npc_list.kill_npcs_by_type(150, false, state, self);
                    npc_list.kill_npcs_by_type(117, false, state, self);

                    let mut npc = NPC::create(355, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.parent_id = self.id;

                    let _ = npc_list.spawn(0x100, npc.clone());
                    if players[1].cond.alive() {
                        npc.tsc_direction = 4;
                        let _ = npc_list.spawn(0x100, npc.clone());
                        npc.tsc_direction = 0;
                    }
                    npc.direction = Direction::Up;
                    let _ = npc_list.spawn(0x100, npc);
                }
            }
            102 => {
                let x = clamp(self.x / (0x2000), 0, stage.map.width as i32) as usize;
                let y = clamp(self.y / (0x2000), 0, stage.map.height as i32) as usize;

                if y <= 34 && stage.change_tile(x, y, 0) {
                    state.sound_manager.play_sfx(44);
                    state.super_quake_counter = 10;
                    state.super_quake_rumble_counter = 10;

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = x as i32 * 0x2000;
                    npc.y = y as i32 * 0x2000;

                    let _ = npc_list.spawn(0, npc.clone());
                    let _ = npc_list.spawn(0, npc.clone());
                    let _ = npc_list.spawn(0, npc.clone());

                    if x > 0 && stage.change_tile(x - 1, y, 0) {
                        npc.x = (x - 1) as i32 * 0x2000;
                        let _ = npc_list.spawn(0, npc.clone());
                        let _ = npc_list.spawn(0, npc.clone());
                        let _ = npc_list.spawn(0, npc.clone());
                    }

                    if x < stage.map.width as usize && stage.change_tile(x + 1, y, 0) {
                        npc.x = (x + 1) as i32 * 0x2000;
                        let _ = npc_list.spawn(0, npc.clone());
                        let _ = npc_list.spawn(0, npc.clone());
                        let _ = npc_list.spawn(0, npc);
                    }
                }

                if self.y < -32 * 0x200 {
                    self.npc_type = 0;
                    state.quake_counter = 30;
                    state.quake_rumble_counter = 30;
                }
            }
            _ => (),
        }

        if self.target_x != 0 && self.rng.range(0..10) == 0 {
            let mut npc = NPC::create(4, &state.npc_table);
            npc.cond.set_alive(true);
            npc.direction = Direction::Left;
            npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
            npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
            npc.vel_x = self.rng.range(-0x155..0x155) as i32;
            npc.vel_y = self.rng.range(-0x600..0) as i32;

            let _ = npc_list.spawn(0x100, npc);
        }

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 14 };

        self.anim_rect = state.constants.npc.n012_balrog_cutscene[self.anim_num as usize + dir_offset];

        if self.action_num == 71 {
            self.anim_rect.bottom = self.anim_rect.top + self.action_counter as u16 / 2;
            if self.action_counter % 2 == 0 {
                self.anim_rect.left += 1;
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n019_balrog_bust_in(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { npc_list, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;

                    for _ in 0..16 {
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }

                    state.sound_manager.play_sfx(12);
                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;
                    state.quake_rumble_counter = 30;

                    self.y += 0x1400;
                    self.action_num = 1;
                    self.anim_num = 3;
                    self.vel_y = -0x100;
                }

                self.vel_y += 0x10;

                if self.vel_y > 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 2;
                    self.anim_num = 2;
                    self.action_counter = 0;

                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;
                    state.quake_rumble_counter = 30;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_num = 3;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }
            }
            3 => {
                if self.rng.range(0..100) == 0 {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            4 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_num = 3;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        self.vel_y = clamp(self.vel_y, -0x5ff, 0x5ff);

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.npc.n019_balrog_bust_in[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n033_balrog_bouncing_projectile(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        if self.flags.hit_left_wall() || self.flags.hit_right_wall() {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        } else if self.flags.hit_bottom_wall() {
            self.vel_y = -0x400;
        }

        self.vel_y += 0x2a;

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_counter += 1;
        if self.anim_counter > 2 {
            self.anim_counter = 0;

            self.anim_num += 1;
            if self.anim_num > 1 {
                self.anim_num = 0;
            }
        }

        self.anim_rect = state.constants.npc.n033_balrog_bouncing_projectile[self.anim_num as usize];

        self.action_counter += 1;
        if self.action_counter > 250 {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        Ok(())
    }

    pub(crate) fn tick_n036_balrog_hover(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                }

                self.action_counter += 1;
                if self.action_counter > 12 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.vel_x2 = 3;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.vel_x2 -= 1;
                    self.action_counter = 0;

                    let angle = f64::atan2((self.y + 0x800 - player.y) as f64, (self.x - player.x) as f64)
                        + self.rng.range(-16..16) as f64 * CDEG_RAD;

                    let mut npc = NPC::create(11, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.vel_x = (angle.cos() * -512.0) as i32;
                    npc.vel_y = (angle.sin() * -512.0) as i32;
                    npc.x = self.x;
                    npc.y = self.y + 0x800;

                    let _ = npc_list.spawn(0x100, npc);
                    state.sound_manager.play_sfx(39);

                    if self.vel_x2 == 0 {
                        self.action_num = 3;
                        self.action_counter = 0;
                    }
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 3 {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.anim_num = 3;

                    self.vel_x = (player.x - self.x) / 100;
                    self.vel_y = -0x600;
                }
            }
            4 => {
                if self.vel_y > -0x200 {
                    if self.life > 60 {
                        self.action_num = 5;
                        self.action_counter = 0;
                        self.anim_num = 4;
                        self.anim_counter = 0;
                        self.target_y = self.y;
                    } else {
                        self.action_num = 6;
                    }
                }
            }
            5 => {
                self.anim_counter += 1;
                if self.anim_counter > 1 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 5 {
                        self.anim_num = 4;
                        state.sound_manager.play_sfx(47);
                    }
                }

                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 6;
                    self.anim_num = 3;
                }

                self.vel_y += if self.y < self.target_y { 0x40 } else { -0x40 };
                self.vel_y = clamp(self.vel_y, -0x200, 0x200);
            }
            6 => {
                if self.y + 0x2000 < player.y {
                    self.damage = 10;
                } else {
                    self.damage = 0;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 7;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.damage = 0;

                    state.sound_manager.play_sfx(25);
                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;
                    state.quake_rumble_counter = 30;

                    let mut npc_smoke = NPC::create(4, &state.npc_table);
                    npc_smoke.cond.set_alive(true);
                    npc_smoke.direction = Direction::Left;

                    let mut npc_proj = NPC::create(33, &state.npc_table);
                    npc_proj.cond.set_alive(true);
                    npc_proj.direction = Direction::Left;

                    for _ in 0..8 {
                        npc_smoke.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc_smoke.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc_smoke.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc_smoke.vel_y = self.rng.range(-0x600..0) as i32;
                        let _ = npc_list.spawn(0x100, npc_smoke.clone());

                        npc_proj.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc_proj.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc_proj.vel_x = self.rng.range(-0x400..0x400) as i32;
                        npc_proj.vel_y = self.rng.range(-0x400..0) as i32;

                        let _ = npc_list.spawn(0x100, npc_proj.clone());
                    }
                }
            }
            7 => {
                self.vel_x = 0;

                self.action_counter += 1;
                if self.action_counter > 3 {
                    self.action_num = 1;
                    self.action_counter = 0;
                }
            }
            _ => (),
        }

        if self.action_num != 5 {
            self.vel_y += 0x33;
            self.direction = if self.x < player.x { Direction::Right } else { Direction::Left };
        }

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n036_balrog_hover[self.anim_num as usize + dir_offset];

        Ok(())
    }

    /// note: vel_y2 stores currently caught player
    pub(crate) fn tick_n068_balrog_running(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { mut players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.action_counter = 30;

                    let player = self.get_closest_player_mut(players);
                    if self.x > player.x {
                        self.direction = Direction::Left;
                    } else {
                        self.direction = Direction::Right;
                    }
                }

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 2;
                    self.action_counter2 += 1;
                }
            }
            2 | 3 => {
                if self.action_num == 2 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num == 2 || self.anim_num == 4 {
                        state.sound_manager.play_sfx(23);
                    }

                    if self.anim_num > 4 {
                        self.anim_num = 1;
                    }
                }

                self.vel_x += 0x10 * self.direction.vector_x();

                let pi = self.get_closest_player_idx_mut(&players);
                if self.action_counter >= 8
                    && (players[pi].x - self.x).abs() < 0x1800
                    && self.y - 0x1800 < players[pi].y
                    && self.y + 0x1000 > players[pi].y
                {
                    self.action_num = 10;
                    self.anim_num = 5;
                    self.vel_y2 = pi as i32;
                    players[pi].cond.set_hidden(true);
                    players[pi].damage(2, state, npc_list);
                } else {
                    self.action_counter += 1;

                    if (self.flags.hit_left_wall() || self.flags.hit_right_wall()) || self.action_counter > 75 {
                        self.action_num = 9;
                        self.anim_num = 0;
                    } else if (self.action_counter2 % 3 == 0) && self.action_counter > 25 {
                        self.action_num = 4;
                        self.anim_num = 7;
                        self.vel_y = -0x400; // -2.0fix9
                    }
                }
            }
            4 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 9;
                    self.anim_num = 8;
                    state.quake_counter = 30;
                    state.quake_rumble_counter = 30;
                    state.sound_manager.play_sfx(26);
                }

                let pi = self.get_closest_player_idx_mut(&players);
                if self.action_counter >= 8
                    && (players[pi].x - self.x).abs() < 0x1800
                    && self.y - 0x1800 < players[pi].y
                    && self.y + 0x1000 > players[pi].y
                {
                    self.action_num = 10;
                    self.anim_num = 5;
                    self.vel_y2 = pi as i32;
                    players[pi].cond.set_hidden(true);
                    players[pi].damage(2, state, npc_list);
                }
            }
            9 => {
                self.vel_x = self.vel_x * 4 / 5;

                if self.vel_x == 0 {
                    self.action_num = 0;
                }
            }
            10 => {
                let player = &mut players[self.vel_y2 as usize];
                player.x = self.x;
                player.y = self.y;

                self.vel_x = self.vel_x * 4 / 5;

                if self.vel_x == 0 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 5;
                    self.anim_counter = 0;
                }
            }
            11 => {
                let player = &mut players[self.vel_y2 as usize];
                player.x = self.x;
                player.y = self.y;

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num > 6 {
                        self.anim_num = 5;
                    }
                }

                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 20;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    let player = &mut players[self.vel_y2 as usize];
                    state.sound_manager.play_sfx(25);
                    player.cond.set_hidden(false);

                    self.direction = self.direction.opposite();

                    player.direction = self.direction;
                    player.x += 0x800 * self.direction.vector_x();
                    player.y -= 0x1000;
                    player.vel_x = 0x5ff * self.direction.vector_x();
                    player.vel_y = -0x200;

                    self.action_num = 21;
                    self.action_counter = 0;
                    self.anim_num = 7;
                }

                self.action_counter += 1;
                if self.action_counter >= 50 {
                    self.action_num = 0;
                }
            }
            _ => (),
        }

        self.vel_x = clamp(self.vel_x, -0x400, 0x400);
        self.vel_y += 0x20;

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 9 };

        self.anim_rect = state.constants.npc.n068_balrog_running[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n169_balrog_shooting_missiles(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { mut players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 30;
                    self.anim_num = 0;

                    let player = self.get_closest_player_mut(players);
                    if self.x <= player.x {
                        self.direction = Direction::Right;
                    } else {
                        self.direction = Direction::Left;
                    }
                }

                self.action_counter = self.action_counter.saturating_sub(1);
                if self.action_counter == 0 {
                    self.action_num = 2;
                    self.action_counter3 += 1;
                }
            }
            2 | 3 => {
                if self.action_num == 2 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num == 2 || self.anim_num == 4 {
                        state.sound_manager.play_sfx(23);
                    }

                    if self.anim_num > 4 {
                        self.anim_num = 1;
                    }
                }

                self.vel_x += self.direction.vector_x() * 0x20;

                let player_idx = self.get_closest_player_idx_mut(&players);
                let player = &mut players[player_idx];

                if self.action_counter <= 7
                    || self.x - 0x1800 >= player.x
                    || self.x + 0x1800 <= player.x
                    || self.y - 0x1800 >= player.y
                    || self.y + 0x1000 <= player.y
                {
                    self.action_counter += 1;
                    if self.action_counter <= 75 {
                        if self.flags.hit_left_wall() || self.flags.hit_right_wall() {
                            if self.action_counter2 > 4 {
                                self.action_num = 4;
                                self.action_counter = 0;
                                self.anim_num = 7;
                                self.vel_y = -0x5FF;
                            } else {
                                self.action_counter2 += 1;
                            }
                        } else {
                            self.action_counter2 = 0;
                        }
                        if (self.action_counter3 & 1) == 0 && self.action_counter > 25 {
                            self.action_num = 4;
                            self.action_counter = 0;
                            self.anim_num = 7;
                            self.vel_y = -0x5FF;
                        }
                    } else {
                        self.action_num = 9;
                        self.anim_num = 0;
                    }
                } else {
                    self.action_num = 10;
                    self.anim_num = 5;
                    self.target_x = player_idx as i32;

                    player.cond.set_hidden(true);
                    player.damage(5, state, npc_list);
                }
            }
            4 => {
                let player_idx = self.get_closest_player_idx_mut(&players);
                let player = &mut players[player_idx];

                if self.x <= player.x {
                    self.direction = Direction::Right;
                } else {
                    self.direction = Direction::Left;
                }

                self.action_counter += 1;
                if self.action_counter <= 29 && self.action_counter % 6 == 1 {
                    state.sound_manager.play_sfx(39);

                    let mut npc = NPC::create(170, &state.npc_table);
                    npc.cond.set_alive(true);

                    npc.x = self.x;
                    npc.y = self.y;
                    npc.direction = self.direction;

                    let _ = npc_list.spawn(0x100, npc);
                }
                if self.flags.hit_bottom_wall() {
                    self.action_num = 9;
                    self.anim_num = 8;
                    state.quake_counter = 30;
                    state.quake_rumble_counter = 30;
                    state.sound_manager.play_sfx(26);
                }
                if self.action_counter > 7
                    && self.x - 0x1800 < player.x
                    && self.x + 0x1800 > player.x
                    && self.y - 0x1800 < player.y
                    && self.y + 0x1000 > player.y
                {
                    self.action_num = 10;
                    self.anim_num = 5;

                    player.cond.set_hidden(true);
                    player.damage(10, state, npc_list);
                }
            }
            9 => {
                self.vel_x = 4 * self.vel_x / 5;
                if self.vel_x == 0 {
                    self.action_num = 0;
                }
            }
            10 => {
                let player = &mut players[self.target_x as usize];
                player.x = self.x;
                player.y = self.y;

                self.vel_x = 4 * self.vel_x / 5;
                if self.vel_x == 0 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 5;
                    self.anim_counter = 0;
                }
            }
            11 => {
                let player = &mut players[self.target_x as usize];
                player.x = self.x;
                player.y = self.y;

                self.animate(2, 5, 6);

                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 20;
                }
            }
            20 | 21 => {
                let player = &mut players[self.target_x as usize];

                if self.action_num == 20 {
                    state.sound_manager.play_sfx(25);
                    player.cond.set_hidden(false);

                    if self.direction != Direction::Left {
                        player.x -= 0x800;
                        player.y -= 0x1000;
                        player.vel_x = -0x5FF;
                        player.vel_y = -0x200;
                        player.direction = Direction::Left;
                        self.direction = Direction::Left;
                    } else {
                        player.x += 0x800;
                        player.y -= 0x1000;
                        player.vel_x = 0x5FF;
                        player.vel_y = -0x200;
                        player.direction = Direction::Right;
                        self.direction = Direction::Right;
                    }
                    self.action_num = 21;
                    self.action_counter = 0;
                    self.anim_num = 7;
                }

                self.action_counter += 1;
                if self.action_counter >= 50 {
                    self.action_num = 0;
                }
            }
            _ => (),
        }

        self.vel_y += 0x20;
        self.vel_x = self.vel_x.clamp(-0x300, 0x300);

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 9 };

        self.anim_rect = state.constants.npc.n169_balrog_shooting_missiles[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n170_balrog_missile(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        if (self.direction == Direction::Left && self.flags.hit_left_wall())
            || (self.direction == Direction::Right && self.flags.hit_right_wall())
        {
            state.sound_manager.play_sfx(44);
            npc_list.create_death_smoke(self.x, self.y, 0, 3, state, &self.rng);
            self.vanish(state);

            return Ok(());
        }

        if self.action_num == 0 {
            self.action_num = 1;
            self.vel_x = if self.direction != Direction::Left {
                self.rng.range(-2..-1) * 0x200
            } else {
                self.rng.range(1..2) * 0x200
            };
            self.vel_y = self.rng.range(-2..0) * 0x200;
        }

        if self.action_num == 1 {
            self.action_counter3 += 1;
            self.vel_x += self.direction.vector_x() * 0x20;
            if self.action_counter3 % 3 == 1 {
                state.create_caret(
                    self.x + 0x1000 * self.direction.opposite().vector_x(),
                    self.y,
                    CaretType::Exhaust,
                    self.direction.opposite(),
                );
            }

            if self.action_counter3 >= 50 {
                self.vel_y = 0;
            } else {
                let player = self.get_closest_player_mut(players);

                self.vel_y = if self.y >= player.y { self.vel_y - 32 } else { self.vel_y + 32 };
            }
            self.anim_num = (self.anim_num + 1) & 1;
        }

        if self.vel_x < -0x400 {
            self.vel_x = -0x600;
        }
        if self.vel_x > 0x400 {
            self.vel_x = 0x600;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n170_balrog_missile[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n306_balrog_nurse(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.y += 0x800;
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
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n306_balrog_nurse[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n356_balrog_rescuing(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 11 => {
                if self.action_num == 0 {
                    self.action_num = 11;
                    self.anim_counter = 0;
                    self.target_x = self.x - 0xC00;
                    self.target_y = self.y - 0x2000;
                    self.vel_y = 0;
                    let mut npc = NPC::create(355, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.parent_id = self.id;
                    if players[1].cond.alive() {
                        npc.tsc_direction = 5;
                        let _ = npc_list.spawn(0xAA, npc.clone());
                        npc.tsc_direction = 6;
                    }
                    npc.direction = Direction::Bottom;
                    let _ = npc_list.spawn(0xAA, npc.clone());
                    npc.direction = Direction::Right;
                    let _ = npc_list.spawn(0xAA, npc);
                }

                self.vel_x += 8 * if self.x < self.target_x { 1 } else { -1 };
                self.vel_y += 8 * if self.y < self.target_y { 1 } else { -1 };

                self.x += self.vel_x;
                self.y += self.vel_y;
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.vel_x = -0x400;
                    self.vel_y = 0x200;
                }
                self.anim_counter += 1;
                self.vel_x += 16;
                self.vel_y -= 8;
                self.x += self.vel_x;
                self.y += self.vel_y;

                if self.x > 0x78000 {
                    self.action_num = 22;
                }
            }
            22 => {
                self.vel_x = 0;
                self.vel_y = 0;
            }
            _ => (),
        }

        self.animate(4, 0, 1);

        self.anim_rect = state.constants.npc.n356_balrog_rescuing[self.anim_num as usize];

        Ok(())
    }
}
