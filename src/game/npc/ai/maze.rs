use crate::common::{CDEG_RAD, Direction};
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n147_critter_purple(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.y += 0x600;
                    self.action_num = 1;
                }

                let player = self.get_closest_player_mut(players);
                if self.action_counter <= 7
                    || self.x - 0xC000 >= player.x
                    || self.x + 0xC000 <= player.x
                    || self.y - 0xC000 >= player.y
                    || self.y + 0x4000 <= player.y
                {
                    if self.action_counter <= 7 {
                        self.action_counter += 1;
                    }
                    self.anim_num = 0;
                } else {
                    if self.x <= player.x {
                        self.direction = Direction::Right;
                    } else {
                        self.direction = Direction::Left;
                    }
                    self.anim_num = 1;
                }
                if self.shock != 0 {
                    self.action_num = 2;
                    self.anim_num = 0;
                    self.action_counter = 0;
                }
                if self.action_counter > 7
                    && self.x - 0x6000 < player.x
                    && self.x + 0x6000 > player.x
                    && self.y - 0xC000 < player.y
                    && self.y + 0x4000 > player.y
                {
                    self.action_num = 2;
                    self.anim_num = 0;
                    self.action_counter = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 3;
                    self.anim_num = 2;
                    self.vel_y = -0x5FF;
                    state.sound_manager.play_sfx(30);

                    let player = self.get_closest_player_mut(players);
                    if self.x <= player.x {
                        self.direction = Direction::Right;
                    } else {
                        self.direction = Direction::Left;
                    }
                }
            }
            3 => {
                if self.vel_y > 0x100 {
                    self.target_y = self.y;
                    self.action_num = 4;
                    self.anim_num = 3;
                    self.action_counter = 0;
                    self.action_counter = 0;
                }
            }
            4 => {
                let player = self.get_closest_player_mut(players);
                if self.x >= player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }
                self.action_counter += 1;
                if (self.flags.hit_left_wall() || self.flags.hit_right_wall() || self.flags.hit_top_wall())
                    || self.action_counter > 60
                {
                    self.damage = 3;
                    self.action_num = 5;
                    self.anim_num = 2;
                } else {
                    if self.action_counter % 4 == 1 {
                        state.sound_manager.play_sfx(109);
                    }
                    if self.flags.hit_bottom_wall() {
                        self.vel_y = -0x200;
                    }
                    if self.action_counter % 30 == 6 {
                        let angle = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64)
                            + (self.rng.range(-6..6) as f64 * CDEG_RAD);

                        let mut npc = NPC::create(148, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.x;
                        npc.y = self.y;
                        npc.vel_x = (angle.cos() * -1536.0) as i32;
                        npc.vel_y = (angle.sin() * -1536.0) as i32;

                        let _ = npc_list.spawn(0x100, npc);
                        state.sound_manager.play_sfx(39);
                    }
                    self.anim_counter += 1;
                    if self.anim_counter > 0 {
                        self.anim_counter = 0;
                        self.anim_num += 1;
                    }
                    if self.anim_num > 5 {
                        self.anim_num = 3;
                    }
                }
            }
            5 => {
                if self.flags.hit_bottom_wall() {
                    self.damage = 2;
                    self.vel_x = 0;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.action_num = 1;
                    state.sound_manager.play_sfx(23);
                }
            }
            _ => (),
        }

        if self.action_num == 4 {
            self.vel_y = if self.y <= self.target_y { self.vel_y + 0x10 } else { self.vel_y - 0x10 };

            self.vel_x = self.vel_x.clamp(-0x200, 0x200);
            self.vel_y = self.vel_y.clamp(-0x200, 0x200);
        } else {
            self.vel_y += 0x20;
            self.clamp_fall_speed();
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n147_critter_purple[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n148_critter_purple_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.flags.hit_anything() {
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
            self.cond.set_alive(false);
        }
        self.y += self.vel_y;
        self.x += self.vel_x;

        self.anim_num += 1;
        if self.anim_num > 1 {
            self.anim_num = 0;
        }

        self.anim_rect = state.constants.npc.n148_critter_purple_projectile[self.anim_num as usize];

        self.action_counter3 += 1;
        if self.action_counter3 > 300 {
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
            self.cond.set_alive(false);
        }

        Ok(())
    }

    pub(crate) fn tick_n153_gaudi(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        let player = self.get_closest_player_mut(players);

        if !(self.x <= player.x + 0x28000
            && self.x >= player.x - 0x28000
            && self.y <= player.y + 0x1E000
            && self.y >= player.y - 0x1E000)
        {
            return Ok(());
        }

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.vel_x = 0;
                    self.anim_num = 0;
                    self.y += 0x600;
                }
                if self.rng.range(0..100) == 1 {
                    self.action_num = 2;
                    self.anim_num = 1;
                    self.action_counter = 0;
                }
                if self.rng.range(0..100) == 1 {
                    if self.direction != Direction::Left {
                        self.direction = Direction::Left;
                    } else {
                        self.direction = Direction::Right;
                    }
                }
                if self.rng.range(0..100) == 1 {
                    self.action_num = 10;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = self.rng.range(25..100) as u16;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }
                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }
                if self.anim_num > 5 {
                    self.anim_num = 2;
                }
                if self.direction != Direction::Left {
                    self.vel_x = 0x200;
                } else {
                    self.vel_x = -0x200;
                }

                if self.action_counter != 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.vel_x = 0;
                }
                if self.direction != Direction::Left || !self.flags.hit_left_wall() {
                    if self.direction == Direction::Right && self.flags.hit_right_wall() {
                        self.anim_num = 2;
                        self.vel_y = -0x5FF;
                        self.action_num = 20;
                        if !player.cond.hidden() {
                            state.sound_manager.play_sfx(30);
                        }
                    }
                } else {
                    self.anim_num = 2;
                    self.vel_y = -0x5FF;
                    self.action_num = 20;
                    if !player.cond.hidden() {
                        state.sound_manager.play_sfx(30);
                    }
                }
            }
            20 => {
                if self.direction != Direction::Left || !self.flags.hit_left_wall() {
                    if self.direction == Direction::Right && self.flags.hit_right_wall() {
                        self.action_counter3 += 1;
                    } else {
                        self.action_counter3 = 0;
                    }
                } else {
                    self.action_counter3 += 1;
                }

                if self.action_counter3 > 10 {
                    if self.direction != Direction::Left {
                        self.direction = Direction::Left;
                    } else {
                        self.direction = Direction::Right;
                    }
                }
                if self.direction != Direction::Left {
                    self.vel_x = 0x100;
                } else {
                    self.vel_x = -0x100;
                }
                if self.flags.hit_bottom_wall() {
                    self.action_num = 21;
                    self.anim_num = 6;
                    self.action_counter = 0;
                    self.vel_x = 0;

                    if !player.cond.hidden() {
                        state.sound_manager.play_sfx(23);
                    }
                }
            }
            21 => {
                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();
        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 7 };

        self.anim_rect = state.constants.npc.n153_gaudi[self.anim_num as usize + dir_offset];

        if self.life <= 985 {
            self.npc_type = 154;
            self.action_num = 0;
        }

        Ok(())
    }

    pub(crate) fn tick_n154_gaudi_dead(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.npc_flags.set_shootable(false);
                self.npc_flags.set_ignore_solidity(false);
                self.damage = 0;
                self.action_num = 1;
                self.anim_num = 0;
                self.vel_y = -0x200;

                match self.direction {
                    Direction::Left => self.vel_x = 0x100,
                    Direction::Right => self.vel_x = -0x100,
                    _ => (),
                };
                state.sound_manager.play_sfx(53);
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
        self.anim_rect = state.constants.npc.n154_gaudi_dead[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n155_gaudi_flying(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        if self.x > player.x + 0x28000
            || self.x < player.x - 0x28000
            || self.y > player.y + 0x1E000
            || self.y < player.y - 0x1E000
        {
            return Ok(());
        }

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    let deg = self.rng.range(0..255) as f64 * CDEG_RAD;
                    self.vel_x = (deg.cos() * 512.0) as i32;
                    self.target_x = self.x + 8 * ((deg + 64.0 * CDEG_RAD).cos() * 512.0) as i32;

                    let deg = self.rng.range(0..255) as f64 * CDEG_RAD;
                    self.vel_y = (deg.sin() * 512.0) as i32;
                    self.target_y = self.y + 8 * ((deg + 64.0 * CDEG_RAD).sin() * 512.0) as i32;

                    self.action_num = 1;
                    self.action_counter3 = 120;
                    self.action_counter = self.rng.range(70..150) as u16;
                    self.anim_num = 0;
                }

                self.anim_num += 1;
                if self.anim_num > 1 {
                    self.anim_num = 0;
                }
                if self.action_counter != 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 2;
                    self.anim_num = 2;
                }
            }
            2 => {
                self.anim_num += 1;
                if self.anim_num > 3 {
                    self.anim_num = 2;
                }

                self.action_counter += 1;
                if self.action_counter > 30 {
                    let angle = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64)
                        + (self.rng.range(-6..6) as f64 * CDEG_RAD);

                    let mut npc = NPC::create(156, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.vel_x = (angle.cos() * -1536.0) as i32;
                    npc.vel_y = (angle.sin() * -1536.0) as i32;

                    let _ = npc_list.spawn(0x100, npc);

                    if !player.cond.hidden() {
                        state.sound_manager.play_sfx(39);
                    }
                    self.action_num = 1;
                    self.action_counter = self.rng.range(70..150) as u16;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }
            }
            _ => (),
        }
        if player.x >= self.x {
            self.direction = Direction::Right;
        } else {
            self.direction = Direction::Left;
        }

        if self.target_x < self.x {
            self.vel_x -= 0x10;
        }
        if self.target_x > self.x {
            self.vel_x += 0x10;
        }
        if self.target_y < self.y {
            self.vel_y -= 0x10;
        }
        if self.target_y > self.y {
            self.vel_y += 0x10;
        }

        self.vel_x = self.vel_x.clamp(-0x200, 0x200);
        self.vel_y = self.vel_y.clamp(-0x200, 0x200);

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };
        self.anim_rect = state.constants.npc.n155_gaudi_flying[self.anim_num as usize + dir_offset];

        if self.life <= 985 {
            self.npc_type = 154;
            self.action_num = 0;
        }
        Ok(())
    }

    pub(crate) fn tick_n156_gaudi_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_counter > 300 || (self.flags.0 & 0xff) != 0 {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_num = (self.anim_num + 1) % 3;

        self.anim_rect = state.constants.npc.n156_gaudi_projectile[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n160_puu_black(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.npc_flags.set_solid_soft(false);
                    self.action_num = 1;
                }

                let player = self.get_closest_player_mut(players);

                if self.x >= player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }
                self.vel_y = 2560;
                if self.y > 0xFFFF {
                    self.npc_flags.set_ignore_solidity(false);
                    self.action_num = 2;
                } else {
                    self.action_counter3 += 1;
                }
            }
            2 => {
                self.vel_y = 0xA00;
                if self.flags.hit_bottom_wall() {
                    npc_list.kill_npcs_by_type(161, true, state);

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;

                    for _ in 0..4 {
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }

                    self.action_num = 3;
                    self.action_counter = 0;
                    state.quake_counter = 30;
                    state.quake_rumble_counter = 30;

                    state.sound_manager.play_sfx(26);
                    state.sound_manager.play_sfx(72);
                }

                let player = self.get_closest_player_mut(players);
                if self.y < player.y && player.flags.hit_bottom_wall() {
                    self.damage = 20;
                } else {
                    self.damage = 0;
                }
            }
            3 => {
                self.damage = 0;
                self.action_counter += 1;
                if self.action_counter > 24 {
                    self.action_num = 4;
                    self.action_counter3 = 0;
                    self.action_counter2 = 0;
                }
            }
            4 => {
                state.npc_super_pos = (self.x, self.y);

                if (self.shock & 1) != 0 {
                    let mut npc = NPC::create(161, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;

                    npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                    npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                    npc.vel_x = self.rng.range(-0x600..0x600) as i32;
                    npc.vel_y = self.rng.range(-0x600..0x600) as i32;

                    let _ = npc_list.spawn(0x100, npc);

                    self.action_counter3 += 1;
                    if self.action_counter3 > 30 {
                        self.action_counter3 = 0;
                        self.action_num = 5;
                        self.vel_y = -3072;
                        self.npc_flags.set_ignore_solidity(true);
                    }
                }
            }
            5 => {
                state.npc_super_pos = (self.x, self.y);

                self.action_counter3 += 1;
                if self.action_counter3 > 60 {
                    self.action_counter3 = 0;
                    self.action_num = 6;
                }
            }
            6 => {
                let player = self.get_closest_player_mut(players);
                state.npc_super_pos = (player.x, 3276800);
                self.action_counter3 += 1;
                if self.action_counter3 > 110 {
                    self.action_counter3 = 10;
                    self.x = player.x;
                    self.y = 0;
                    self.vel_y = 1535;
                    self.action_num = 1;
                }
            }
            _ => (),
        }
        self.y += self.vel_y;

        self.anim_num = match self.action_num {
            0 | 1 | 2 | 5 | 6 => 3,
            3 => 2,
            4 => 0,
            _ => 0,
        };

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };
        self.anim_rect = state.constants.npc.n160_puu_black[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n161_puu_black_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        self.exp = 0;

        self.vel_x = if self.x >= state.npc_super_pos.0 { self.vel_x - 64 } else { self.vel_x + 64 };
        self.vel_y = if self.y >= state.npc_super_pos.1 { self.vel_y - 64 } else { self.vel_y + 64 };

        self.vel_x = self.vel_x.clamp(-0x11fd, 0x11fd);
        self.vel_y = self.vel_y.clamp(-0x11fd, 0x11fd);

        if self.life <= 99 {
            self.npc_flags.set_shootable(false);
            self.npc_flags.set_invulnerable(false);
            self.damage = 0;
            self.anim_num = 2;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;
        if self.anim_num <= 1 {
            self.anim_num = if self.rng.range(0..10) != 2 { 1 } else { 0 };
        }

        self.anim_rect = state.constants.npc.n161_puu_black_projectile[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n166_chaba(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
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

        self.anim_rect = state.constants.npc.n166_chaba[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n162_puu_black_dead(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    npc_list.kill_npcs_by_type(161, true, state);
                    state.sound_manager.play_sfx(72);

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);

                    for _ in 0..10 {
                        npc.x = self.x + self.rng.range(-12..12) * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) * 0x200;
                        npc.vel_x = self.rng.range(-0x600..0x600);
                        npc.vel_y = self.rng.range(-0x600..0x600);

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }

                    let player = self.get_closest_player_mut(players);
                    if self.x <= player.x {
                        self.direction = Direction::Right;
                    } else {
                        self.direction = Direction::Left;
                    }

                    self.anim_num = if self.direction != Direction::Left { 1 } else { 0 };
                    self.action_counter3 = 0;
                    self.action_num = 1;
                }
                self.action_counter3 += 1;
                if (self.action_counter3 & 3) == 0 {
                    let mut npc = NPC::create(161, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x + self.rng.range(-12..12) * 0x200;
                    npc.y = self.y + self.rng.range(-12..12) * 0x200;

                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.action_counter3 > 160 {
                    self.action_counter3 = 0;
                    self.action_num = 2;
                    self.target_y = self.y;
                }
            }
            2 => {
                state.quake_counter = 2;
                state.quake_rumble_counter = 2;
                self.action_counter3 += 1;
                if self.action_counter3 > 240 {
                    self.anim_num = 2;
                    self.action_counter3 = 0;
                    self.action_num = 3;
                } else {
                    self.anim_num = if self.direction != Direction::Left { 1 } else { 0 };

                    self.y = ((self.action_counter3 as i32 / 8) * 0x200) + self.target_y;
                }
                if self.action_counter3 % 3 == 2 {
                    let mut npc = NPC::create(161, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x + self.rng.range(-12..12) * 0x200;
                    npc.y = self.y - 6144;
                    npc.vel_x = self.rng.range(-512..512);
                    npc.vel_y = 0x100;

                    let _ = npc_list.spawn(0x100, npc);
                }
                if self.action_counter3 % 4 == 2 {
                    state.sound_manager.play_sfx(21);
                }
            }
            3 => {
                self.action_counter3 += 1;
                if self.action_counter3 > 59 {
                    npc_list.kill_npcs_by_type(161, true, state);
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        state.npc_super_pos = (self.x, -512000);

        self.anim_rect = state.constants.npc.n162_puu_black_dead[self.anim_num as usize];

        if self.action_num == 2 && self.action_counter3 <= 240 {
            self.anim_rect.top += self.action_counter3 / 8;
            self.anim_rect.left -= if (self.action_counter3 & 0x02) != 0 { 1 } else { 0 };
        }

        Ok(())
    }

    pub(crate) fn tick_n163_dr_gero(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
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

        self.anim_rect = state.constants.npc.n163_dr_gero[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n164_nurse_hasumi(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
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

        self.anim_rect = state.constants.npc.n164_nurse_hasumi[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n168_boulder(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.target_x = self.x;
                }

                self.action_counter += 1;
                self.x = self.target_x;
                if ((self.action_counter / 3) & 1) != 0 {
                    self.x += 0x200;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                    self.vel_y = -0x400;
                    self.vel_x = 0x100;
                    state.sound_manager.play_sfx(25);
                }

                self.vel_y += 0x10;
                self.x += self.vel_x;
                self.y += self.vel_y;

                if self.action_counter != 0 && self.flags.hit_bottom_wall() {
                    state.sound_manager.play_sfx(35);
                    state.quake_counter = 40;
                    state.quake_rumble_counter = 40;
                    self.action_num = 0;
                }

                if self.action_counter == 0 {
                    self.action_counter += 1;
                }
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n168_boulder;

        Ok(())
    }

    pub(crate) fn tick_n171_fire_whirrr(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);
        self.face_player(player);

        if self.action_num == 0 {
            self.action_num = 1;
            self.action_counter = self.rng.range(0..50) as u16;
            self.action_counter3 = 100;
            self.target_y = self.y;
        }

        if self.action_num == 1 {
            if self.action_counter > 0 {
                self.action_counter -= 1;
            } else {
                self.action_num = 10;
                self.vel_y = 0x200;
            }
        }

        self.animate(0, 0, 1);

        self.vel_y = if self.y >= self.target_y { self.vel_y - 0x10 } else { self.vel_y + 0x10 };
        self.vel_y = self.vel_y.clamp(-0x200, 0x200);
        self.y += self.vel_y;

        if self.direction != Direction::Left {
            if player.y < self.y + 0xA000
                && player.y > self.y - 0xA000
                && player.x < self.x + 0x14000
                && player.x > self.x
            {
                self.action_counter3 += 1;
            }
        } else if player.y < self.y + 0xA000
            && player.y > self.y - 0xA000
            && player.x < self.x
            && player.x > self.x - 0x14000
        {
            self.action_counter3 += 1;
        }

        if self.action_counter3 > 120 {
            self.action_counter3 = 0;
            state.npc_curly_counter = self.rng.range(80..100) as u16;
            state.npc_curly_target = (self.x, self.y);

            let mut npc = NPC::create(172, &state.npc_table);
            npc.cond.set_alive(true);
            npc.x = self.x;
            npc.y = self.y;
            npc.direction = self.direction;

            let _ = npc_list.spawn(0x100, npc);
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n171_fire_whirrr[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n172_fire_whirrr_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            // pixel what?
            self.action_num = 1;
        }

        if self.action_num == 1 {
            self.animate(1, 0, 2);
            self.x += self.direction.vector_x() * 0x200;

            if self.flags.hit_left_wall() || self.flags.hit_right_wall() {
                state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
                self.vanish(state);
            }
        }

        self.anim_rect = state.constants.npc.n172_fire_whirrr_projectile[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n173_gaudi_armored(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        if self.x <= player.x + 0x28000
            && self.x >= player.x - 0x28000
            && self.y <= player.y + 0x1e000
            && self.y >= player.y - 0x1e000
        {
            match self.action_num {
                0 | 1 => {
                    if self.action_num == 0 {
                        self.target_x = self.x;
                        self.action_num = 1;
                    }
                    self.anim_num = 0;
                    self.vel_x = 0;
                    if self.action_counter > 4 {
                        if self.x - 0x18000 < player.x
                            && self.x + 0x18000 > player.x
                            && self.y - 0x14000 < player.y
                            && self.y + 0x14000 > player.y
                        {
                            self.action_num = 10;
                            self.action_counter = 0;
                            self.anim_num = 1;
                        }
                    } else {
                        self.action_counter += 1;
                    }
                }
                10 => {
                    self.action_counter += 1;
                    if self.action_counter > 3 {
                        self.action_counter3 += 1;
                        if self.action_counter3 == 3 {
                            state.sound_manager.play_sfx(30);
                            self.action_counter3 = 0;
                            self.action_num = 25;
                            self.action_counter = 0;
                            self.anim_num = 2;
                            self.vel_y = -0x600;

                            if self.x >= self.target_x {
                                self.vel_x = -0x80;
                            } else {
                                self.vel_x = 0x80;
                            }
                        } else {
                            state.sound_manager.play_sfx(30);
                            self.action_num = 20;
                            self.anim_num = 2;
                            self.vel_y = -0x200;
                            if self.x >= self.target_x {
                                self.vel_x = -0x200;
                            } else {
                                self.vel_x = 0x200;
                            }
                        }
                    }
                }
                20 => {
                    self.action_counter += 1;
                    if self.flags.hit_bottom_wall() {
                        state.sound_manager.play_sfx(23);
                        self.anim_num = 1;
                        self.action_num = 30;
                        self.action_counter = 0;
                    }
                }
                25 => {
                    self.action_counter += 1;
                    if self.action_counter == 30 || self.action_counter == 40 {
                        let angle = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64)
                            + (self.rng.range(-6..6) as f64 * CDEG_RAD);

                        let mut npc = NPC::create(174, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.x;
                        npc.y = self.y;
                        npc.vel_x = (angle.cos() * -1536.0) as i32;
                        npc.vel_y = (angle.sin() * -1536.0) as i32;

                        let _ = npc_list.spawn(0x100, npc);
                        state.sound_manager.play_sfx(39);

                        self.anim_num = 3;
                        state.npc_curly_counter = self.rng.range(80..100) as u16;
                        state.npc_curly_target = (self.x, self.y);
                    }

                    if self.action_counter == 35 || self.action_counter == 45 {
                        self.anim_num = 2;
                    }

                    if self.flags.hit_bottom_wall() {
                        state.sound_manager.play_sfx(23);
                        self.anim_num = 1;
                        self.action_num = 30;
                        self.action_counter = 0;
                    }
                }
                30 => {
                    self.vel_x = 7 * self.vel_x / 8;
                    self.action_counter += 1;
                    if self.action_counter > 3 {
                        self.anim_num = 0;
                        self.action_num = 1;
                        self.action_counter = 0;
                    }
                }
                _ => (),
            }

            self.vel_y += 0x33;
            self.direction = if player.x >= self.x { Direction::Right } else { Direction::Left };

            self.vel_y = self.vel_y.clamp(-0x5ff, 0x5ff);

            self.x += self.vel_x;
            self.y += self.vel_y;

            if self.life <= 985 {
                npc_list.create_death_smoke(self.x, self.y, 0, 2, state, &self.rng);

                self.npc_type = 154;
                self.action_num = 0;
            }

            let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

            self.anim_rect = state.constants.npc.n173_gaudi_armored[self.anim_num as usize + dir_offset];
        }

        Ok(())
    }

    pub(crate) fn tick_n174_gaudi_armored_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.direction == Direction::Right {
                    self.action_num = 2;
                }

                // So we're going to move *before* checking collisions, huh?
                self.x += self.vel_x;
                self.y += self.vel_y;

                let mut hit = false;

                if self.flags.hit_left_wall() {
                    hit = true;
                    self.vel_x = 0x200;
                }
                if self.flags.hit_right_wall() {
                    hit = true;
                    self.vel_x = -0x200;
                }
                if self.flags.hit_top_wall() {
                    hit = true;
                    self.vel_y = 0x200;
                }
                if self.flags.hit_bottom_wall() {
                    hit = true;
                    self.vel_y = -0x200;
                }

                if hit {
                    self.action_num = 2;
                    self.action_counter2 += 1;
                    state.sound_manager.play_sfx(31);
                }
            }
            2 => {
                self.vel_y += 0x40;
                self.x += self.vel_x;
                self.y += self.vel_y;

                if self.flags.hit_bottom_wall() {
                    self.action_counter2 += 1;
                    if self.action_counter2 > 1 {
                        state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
                        self.cond.set_alive(false);
                    }
                }
            }
            _ => (),
        }

        self.vel_y = self.vel_y.clamp(-0x5ff, 0x5ff);

        self.anim_num += 1;
        if self.anim_num > 2 {
            self.anim_num = 0;
        }

        self.anim_rect = state.constants.npc.n174_gaudi_armored_projectile[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n175_gaudi_egg(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num < 3 && self.life < 90 {
            self.cond.set_drs_novanish(true);
            self.cond.set_explode_die(true);

            self.action_num = 10;
            self.anim_num = 1;
            self.npc_flags.set_shootable(false);
            self.damage = 0;
        }

        if self.action_num == 0 {
            self.anim_num = 0;
            self.action_num = 1;
        }

        self.vel_y += if self.direction != Direction::Left { -0x20 } else { 0x20 };
        self.vel_y = self.vel_y.clamp(-0x5ff, 0x5ff);
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n175_gaudi_egg[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n176_buyo_buyo_base(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        if self.action_num < 3 && self.life < 940 {
            self.cond.set_drs_novanish(true);
            self.cond.set_explode_die(true);

            self.action_num = 10;
            self.anim_num = 2;
            self.npc_flags.set_shootable(false);
            self.damage = 0;
        }

        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = 0;
            self.anim_counter = 0;
        }

        let player = self.get_closest_player_mut(players);
        let counter3 = unsafe { std::mem::transmute::<&mut u16, &mut i16>(&mut self.action_counter3) };

        if self.action_num == 1 {
            if self.direction != Direction::Left {
                if self.x < player.x + 0x14000
                    && self.x > player.x - 0x14000
                    && self.y < player.y + 0x2000
                    && self.y > player.y - 0x14000
                {
                    *counter3 += 1;
                }
            } else if self.x < player.x + 0x14000
                && self.x > player.x - 0x14000
                && self.y < player.y + 0x14000
                && self.y > player.y - 0x2000
            {
                *counter3 += 1;
            }
            if *counter3 > 10 {
                self.action_num = 2;
                self.action_counter = 0;
            }
        } else if self.action_num == 2 {
            self.animate(3, 0, 1);

            self.action_counter += 1;
            if self.action_counter > 10 {
                self.action_counter2 += 1;
                if self.action_counter2 < 3 {
                    *counter3 = -10;
                } else {
                    self.action_counter2 = 0;
                    *counter3 = -90;
                }

                let mut npc = NPC::create(177, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;
                if self.direction != Direction::Left {
                    npc.direction = Direction::Right;
                    npc.y = self.y + 0x1000;
                } else {
                    npc.direction = Direction::Left;
                    npc.y = self.y - 0x1000;
                }
                let _ = npc_list.spawn(0x100, npc);

                state.sound_manager.play_sfx(39);
                self.action_num = 0;
                self.anim_num = 0;

                state.npc_curly_counter = self.rng.range(80..100) as u16;
                state.npc_curly_target = (self.x, self.y);
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n176_buyo_buyo_base[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n177_buyo_buyo(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        if self.flags.hit_anything() {
            state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
            self.cond.set_alive(false);
            return Ok(());
        }

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    if self.direction != Direction::Left {
                        self.vel_y = 0x600;
                    } else {
                        self.vel_y = -0x600;
                    }
                }

                let player = self.get_closest_player_mut(players);

                if self.y < player.y + 0x2000 && self.y > player.y - 0x2000 {
                    self.action_num = 10;
                    self.target_x = self.x;
                    self.target_y = self.y;
                    self.direction = if self.x <= player.x { Direction::Right } else { Direction::Left };
                    self.vel_x = (2 * self.rng.range(0..1) - 1) * 0x200;
                    self.vel_y = (2 * self.rng.range(0..1) - 1) * 0x200;
                }
            }
            10 => {
                self.vel_x += if self.x >= self.target_x { -0x20 } else { 0x20 };
                self.vel_y += if self.y >= self.target_y { -0x20 } else { 0x20 };

                self.action_counter += 1;
                if self.action_counter > 300 {
                    state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
                    self.cond.set_alive(false);
                    return Ok(());
                }

                self.target_x += if self.direction != Direction::Left { 0x200 } else { -0x200 };
            }
            _ => (),
        }

        self.vel_x = self.vel_x.clamp(-0x400, 0x400);
        self.vel_y = self.vel_y.clamp(-0x400, 0x400);

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.animate(6, 0, 1);

        self.anim_rect = state.constants.npc.n177_buyo_buyo[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n184_shutter(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.x += 0x1000;
                self.y += 0x1000;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 1;
                    self.action_counter = 0;
                    self.npc_flags.set_ignore_solidity(true);
                }

                match self.direction {
                    Direction::Left => self.x -= 0x80,
                    Direction::Up => self.y -= 0x80,
                    Direction::Right => self.x += 0x80,
                    Direction::Bottom => self.y += 0x80,
                    _ => (),
                };

                self.action_counter += 1;
                if (self.action_counter & 7) == 0 {
                    state.sound_manager.play_sfx(26);
                }
                state.quake_counter = 20;
                state.quake_rumble_counter = 20;
            }
            20 => {
                let mut npc = NPC::create(4, &state.npc_table);
                npc.cond.set_alive(true);
                npc.direction = Direction::Left;

                for _ in 0..4 {
                    npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                    npc.y = self.y + 0x2000;
                    npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                    npc.vel_y = self.rng.range(-0x600..0) as i32;

                    let _ = npc_list.spawn(0x100, npc.clone());
                }

                self.action_num = 1;
            }
            _ => (),
        }

        self.animate(10, 0, 3);
        self.anim_rect = state.constants.npc.n184_shutter[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n185_small_shutter(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.anim_rect = state.constants.npc.n185_small_shutter;
                self.action_num = 1;
                self.y += 0x1000;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 1;
                    self.npc_flags.set_ignore_solidity(true);
                }

                match self.direction {
                    Direction::Left => self.x -= 0x80,
                    Direction::Up => self.y -= 0x80,
                    Direction::Right => self.x += 0x80,
                    Direction::Bottom => self.y += 0x80,
                    _ => (),
                };
            }
            20 => {
                self.y -= 0x3000;
                self.action_num = 1;
            }
            _ => (),
        }

        Ok(())
    }

    pub(crate) fn tick_n186_lift_block(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 1;
                    self.npc_flags.set_ignore_solidity(true);
                }

                match self.direction {
                    Direction::Left => self.x -= 0x80,
                    Direction::Up => self.y -= 0x80,
                    Direction::Right => self.x += 0x80,
                    Direction::Bottom => self.y += 0x80,
                    _ => (),
                };
            }
            _ => (),
        }

        self.animate(10, 0, 3);
        self.anim_rect = state.constants.npc.n186_lift_block[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n187_fuzz_core(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.target_x = self.x;
            self.target_y = self.y;
            self.action_counter3 = 0x78;
            self.action_counter = self.rng.range(0..50) as u16;

            let mut npc = NPC::create(188, &state.npc_table);
            npc.cond.set_alive(true);
            npc.parent_id = self.id;

            for i in 0..5 {
                npc.tsc_direction = 0x33 * i as u16;
                let _ = npc_list.spawn(0x100, npc.clone());
            }
        }

        if self.action_num == 1 {
            self.action_counter += 1;
            if self.action_counter >= 50 {
                self.action_counter = 0;
                self.action_num = 2;
                self.vel_y = 0x300;
            }
        } else if self.action_num == 2 {
            let player = self.get_closest_player_mut(players);

            self.action_counter3 = self.action_counter3.wrapping_add(4);
            self.direction = if player.x >= self.x { Direction::Right } else { Direction::Left };

            self.vel_y += (self.target_y - self.y).signum() * 0x10;
            self.vel_y = self.vel_y.clamp(-0x355, 0x355);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.animate(2, 0, 1);

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n187_fuzz_core[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n188_fuzz(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.action_counter3 = self.tsc_direction;
        }

        let player = self.get_closest_player_mut(players);

        if self.action_num == 1 {
            if let Some(parent) = self.get_parent_ref_mut(npc_list) {
                if parent.npc_type == 187 && parent.cond.alive() {
                    let deg = (self.action_counter3.wrapping_add(parent.action_counter3) & 0xff) as f64 * CDEG_RAD;

                    self.x = parent.x + 20 * (deg.sin() * 512.0) as i32;
                    self.y = parent.y + 32 * (deg.cos() * 512.0) as i32;
                } else {
                    self.vel_x = self.rng.range(-512..512);
                    self.vel_y = self.rng.range(-512..512);
                    self.action_num = 10;
                }
            }
        } else if self.action_num == 10 {
            self.vel_x += if player.x >= self.x { 0x20 } else { -0x20 };

            self.vel_y += if player.y >= self.y { 0x20 } else { -0x20 };

            self.vel_x = self.vel_x.clamp(-0x800, 0x800);
            self.vel_y = self.vel_y.clamp(-0x200, 0x200);
            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        if player.x >= self.x {
            self.direction = Direction::Right;
        } else {
            self.direction = Direction::Left;
        }

        self.animate(2, 0, 1);

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n188_fuzz[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n304_gaudi_hospital(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.y += 0x1400;
                }

                self.anim_num = 0;
            }
            10 => {
                self.anim_num = 1;
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.anim_num = 2;
                }

                self.animate(10, 2, 3);
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n304_gaudi_hospital[self.anim_num as usize];

        Ok(())
    }
}
