use std::cmp::Ordering;

use num_traits::{abs, clamp};

use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::game::npc::list::{NPCAccessTokenProvider, NPCList, NPCRefMut};
use crate::game::npc::NPC;
use crate::game::player::{Player, TargetPlayer};
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::Stage;
use crate::game::weapon::bullet::BulletManager;
use crate::util::rng::RNG;

impl<P: NPCAccessTokenProvider> NPCRefMut<'_, P> {
    pub(crate) fn tick_n069_pignon(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;
                }

                if self.rng.range(0..100) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                } else {
                    if self.rng.range(0..150) == 1 {
                        self.direction = self.direction.opposite();
                    }
                    if self.rng.range(0..150) == 1 {
                        self.action_num = 3;
                        self.action_counter = 50;
                        self.anim_num = 0;
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
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.action_counter = self.action_counter.saturating_sub(1);
                if self.action_counter == 0 {
                    self.action_num = 0;
                }

                self.animate(2, 2, 4);

                if self.flags.hit_left_wall() {
                    self.direction = Direction::Right;
                }

                if self.flags.hit_right_wall() {
                    self.direction = Direction::Left;
                }

                self.vel_x = self.direction.vector_x() * 0x100;
            }
            5 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 0;
                }
            }
            _ => (),
        }

        if self.shock > 0 && [1, 2, 4].contains(&self.action_num) {
            self.vel_y = -0x200;
            self.anim_num = 5;
            self.action_num = 5;
        }

        self.vel_y += 0x40;

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };
        self.anim_rect = state.constants.npc.n069_pignon[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n071_chinfish(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.target_x = self.x;
            self.target_y = self.y;
            self.vel_y = 0x80;
        }

        if self.action_num == 1 {
            self.vel_y += match self.target_y.cmp(&self.y) {
                Ordering::Less => -8,
                Ordering::Equal => 0,
                Ordering::Greater => 8,
            };

            self.vel_y = clamp(self.vel_y, -0x100, 0x100);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.animate(4, 0, 1);

        if self.shock > 0 {
            self.anim_num = 2;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n071_chinfish[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n075_kanpachi(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = 0;
            self.anim_counter = 0;
        }

        if self.action_num == 1 {
            let player = self.get_closest_player_mut(players);
            if (self.x - 0x6000 < player.x)
                && (self.x + 0x6000 > player.x)
                && (self.y - 0x6000 < player.y)
                && (self.y + 0x2000 > player.y)
            {
                self.anim_num = 1;
            } else {
                self.anim_num = 0;
            }
        }

        self.anim_rect = state.constants.npc.n075_kanpachi[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n077_yamashita(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = 0;
            self.anim_counter = 0;
        }

        match self.action_num {
            1 => {
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

        if self.direction == Direction::Left {
            self.anim_rect = state.constants.npc.n077_yamashita[self.anim_num as usize];
        } else {
            self.anim_rect = state.constants.npc.n077_yamashita[2];
        }

        Ok(())
    }

    pub(crate) fn tick_n079_mahin(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 2;
                self.anim_counter = 0;
            }
            2 => {
                self.anim_num = 0;
                if self.rng.range(0..120) == 10 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                let player = self.get_closest_player_mut(players);
                if (self.x - (0x4000) < player.x)
                    && (self.x + (0x4000) > player.x)
                    && (self.y - (0x4000) < player.y)
                    && (self.y + (0x2000) > player.y)
                {
                    self.face_player(player);
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 2;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;

        self.clamp_fall_speed();

        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n079_mahin[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n080_gravekeeper(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.npc_flags.set_shootable(false);
                    self.damage = 0;
                    self.action_num = 1;
                    self.hit_bounds.left = 0x800;
                }

                self.anim_num = 0;

                let player = self.get_closest_player_mut(players);
                if abs(player.x - self.x) < 0x10000 && self.y - 0x6000 < player.y && self.y + 0x4000 > player.y {
                    self.anim_counter = 0;
                    self.action_num = 2;
                }

                if self.shock > 0 {
                    self.anim_num = 1;
                    self.anim_counter = 0;
                    self.action_num = 2;
                    self.npc_flags.set_shootable(false);
                }

                self.face_player(player);
            }
            2 => {
                self.animate(6, 0, 3);

                let player = self.get_closest_player_mut(players);
                if abs(player.x - self.x) < 0x2000 {
                    self.hit_bounds.left = 0x2400;
                    self.action_counter = 0;
                    self.action_num = 3;
                    self.npc_flags.set_shootable(true);

                    state.sound_manager.play_sfx(34);
                }

                self.face_player(player);
                self.vel_x = self.direction.vector_x() * 0x100;
            }
            3 => {
                self.anim_num = 4;
                self.vel_x = 0;

                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.action_counter = 0;
                    self.action_num = 4;
                    state.sound_manager.play_sfx(106);
                }
            }
            4 => {
                self.anim_num = 5;
                self.damage = 10;

                self.action_counter += 1;
                if self.action_counter > 2 {
                    self.action_counter = 0;
                    self.action_num = 5;
                }
            }
            5 => {
                self.action_counter += 1;
                if self.action_counter > 60 {
                    self.action_num = 0;
                }

                self.anim_num = 6;
            }
            _ => (),
        }

        if (self.vel_x < 0 && self.flags.hit_left_wall()) || (self.vel_x > 0 && self.flags.hit_right_wall()) {
            self.vel_x = 0;
        }

        self.vel_x = clamp(self.vel_x, -0x400, 0x400);
        self.vel_y = clamp(self.vel_y + 0x20, -0x5ff, 0x5ff);

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 7 };
        self.anim_rect = state.constants.npc.n080_gravekeeper[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n081_giant_pignon(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;
                }

                if self.rng.range(0..100) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                } else {
                    if self.rng.range(0..150) == 1 {
                        self.direction = self.direction.opposite();
                    }

                    if self.rng.range(0..150) == 1 {
                        self.action_num = 3;
                        self.action_counter = 50;
                        self.anim_num = 0;
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
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.action_counter = self.action_counter.saturating_sub(1);
                if self.action_counter == 0 {
                    self.action_num = 0;
                }

                self.animate(2, 2, 4);

                if self.flags.hit_left_wall() {
                    self.direction = Direction::Right;
                }

                if self.flags.hit_right_wall() {
                    self.direction = Direction::Left;
                }

                self.vel_x = self.direction.vector_x() * 0x100;
            }
            5 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 0;
                }
            }
            _ => (),
        }

        if self.shock > 0 && [1, 2, 4].contains(&self.action_num) {
            let player = self.get_closest_player_mut(players);
            self.vel_x = if self.x < player.x { 0x100 } else { -0x100 };
            self.vel_y = -0x200;
            self.anim_num = 5;
            self.action_num = 5;
        }

        self.vel_y += 0x40;

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n081_giant_pignon[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n091_mimiga_cage(&mut self, state: &SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y += 0x2000;
            self.anim_rect = state.constants.npc.n091_mimiga_cage;
        }

        Ok(())
    }

    pub(crate) fn tick_n313_ma_pignon(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        bullet_manager: &mut BulletManager,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.y += 0x800;
                }

                self.vel_y += 0x40;

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                if player.x > self.x - 0x4000 && player.x < self.x + 0x4000 {
                    self.face_player(player);
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            100 | 110 => {
                if self.action_num == 100 {
                    self.action_num = 110;
                    self.action_counter = 0;
                    self.action_counter2 = 0;
                    self.npc_flags.set_shootable(true);
                }

                self.damage = 1;
                self.face_player(player);
                self.anim_num = 0;

                self.action_counter += 1;
                if self.action_counter > 4 {
                    self.action_counter = 0;
                    self.action_num = 120;

                    self.action_counter3 += 1;
                    if self.action_counter3 > 12 {
                        self.action_counter3 = 0;
                        self.action_num = 300;
                    }
                }
            }
            120 => {
                self.anim_num = 2;

                self.action_counter += 1;
                if self.action_counter > 4 {
                    self.action_num = 130;
                    self.anim_num = 3;
                    self.vel_x = 2 * self.rng.range(-512..512);
                    self.vel_y = -0x800;
                    state.sound_manager.play_sfx(30);
                    self.action_counter2 += 1;
                }
            }
            130 => {
                self.vel_y += 0x80;

                if self.y > 0x10000 {
                    self.npc_flags.set_ignore_solidity(false)
                };
                if (self.vel_x < 0 && self.flags.hit_left_wall()) || (self.vel_x > 0 && self.flags.hit_right_wall()) {
                    self.vel_x *= -1
                };
                self.face_player(player);
                self.anim_num = if self.vel_y < -0x200 {
                    3
                } else if self.vel_y > 0x200 {
                    4
                } else {
                    0
                };

                if self.flags.hit_bottom_wall() {
                    self.action_num = 140;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.vel_x = 0;
                }

                if self.action_counter2 > 4 && player.y < self.y + 0x800 {
                    self.action_num = 200;
                    self.action_counter = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;
                }
            }
            140 => {
                self.anim_num = 2;

                self.action_counter += 1;
                if self.action_counter > 4 {
                    self.action_num = 110;
                }
            }
            200 => {
                self.anim_num = 5;

                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 210;
                    self.anim_num = 6;

                    self.vel_x = 0x5FF * self.direction.vector_x();

                    state.sound_manager.play_sfx(25);
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_invulnerable(true);
                    self.damage = 10;
                }
            }
            210 => {
                self.animate(1, 6, 7);

                if (self.vel_x < 0 && self.flags.hit_left_wall()) || (self.vel_x > 0 && self.flags.hit_right_wall()) {
                    self.action_num = 220
                };
            }
            220 | 221 => {
                if self.action_num == 220 {
                    self.action_num = 221;
                    self.action_counter = 0;
                    state.quake_counter = 16;
                    state.quake_rumble_counter = 16;
                    state.sound_manager.play_sfx(26);
                    self.damage = 4
                }

                self.animate(1, 6, 7);

                self.action_counter += 1;
                if self.action_counter % 6 == 0 {
                    let mut npc = NPC::create(314, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = (self.rng.range(4..16) + state.constants.game.tile_offset_x) * 0x2000;
                    npc.y = 0x2000;

                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.action_counter > 30 {
                    self.action_counter2 = 0;
                    self.action_num = 130;
                    self.npc_flags.set_shootable(true);
                    self.npc_flags.set_invulnerable(false);
                    self.damage = 3;
                }
            }
            300 | 301 => {
                if self.action_num == 300 {
                    self.action_num = 301;
                    self.anim_num = 9;
                    self.face_player(player);
                }

                self.animate(1, 9, 11);

                self.vel_x = if self.direction == Direction::Left { -0x400 } else { 0x400 };

                if player.x > self.x - 0x800 && player.x < self.x + 0x800 {
                    self.action_num = 310;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.vel_x = 0;
                }
            }
            310 => {
                self.anim_num = 2;

                self.action_counter += 1;
                if self.action_counter > 4 {
                    self.action_num = 320;
                    self.anim_num = 12;
                    self.vel_y = -0x800;
                    state.sound_manager.play_sfx(25);
                    self.npc_flags.set_ignore_solidity(true);
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_invulnerable(true);
                    self.damage = 10;
                }
            }
            320 => {
                self.animate(1, 12, 13);

                if self.y < 0x2000 {
                    self.action_num = 330
                };
            }
            330 | 331 => {
                if self.action_num == 330 {
                    self.vel_y = 0;
                    self.action_num = 331;
                    self.action_counter = 0;
                    state.quake_counter = 16;
                    state.quake_rumble_counter = 16;
                    state.sound_manager.play_sfx(26);
                }

                self.animate(1, 12, 13);

                self.action_counter += 1;

                if self.action_counter % 6 == 0 {
                    let mut npc = NPC::create(315, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = (self.rng.range(4..16) + state.constants.game.tile_offset_x) * 0x2000;

                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.action_counter > 30 {
                    self.action_counter2 = 0;
                    self.action_num = 130;
                    self.npc_flags.set_shootable(true);
                    self.npc_flags.set_invulnerable(false);
                    self.damage = 3;
                }
            }
            500 | 501 => {
                if self.action_num == 500 {
                    self.npc_flags.set_shootable(false);
                    self.action_num = 501;
                    self.action_counter = 0;
                    self.anim_num = 8;
                    self.target_x = self.x;
                    self.damage = 0;
                    npc_list.kill_npcs_by_type(315, true, state, self);
                }
                self.vel_y += 0x20;

                self.action_counter += 1;
                self.x = self.target_x + if self.action_counter % 2 == 0 { 0x200 } else { 0 };
            }
            _ => (),
        }

        if self.action_num > 100 && self.action_num < 500 && self.action_num != 210 && self.action_num != 320 {
            // Missiles + Blade
            if bullet_manager.count_bullets_multi(
                &[13, 14, 15, 16, 17, 18, 23, 25, 26, 27, 28, 29, 30, 31, 32, 33],
                TargetPlayer::Player1,
            ) + bullet_manager.count_bullets_multi(
                &[13, 14, 15, 16, 17, 18, 23, 25, 26, 27, 28, 29, 30, 31, 32, 33],
                TargetPlayer::Player2,
            ) > 0
            {
                self.npc_flags.set_shootable(false);
                self.npc_flags.set_invulnerable(true);
            } else {
                self.npc_flags.set_shootable(true);
                self.npc_flags.set_invulnerable(false);
            }
        }

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 14 };
        self.anim_rect = state.constants.npc.n313_ma_pignon[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n314_ma_pignon_rock(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        stage: &Stage,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 100 => {
                if self.action_num == 0 {
                    self.action_num = 100;
                    self.npc_flags.set_invulnerable(true);
                    self.anim_num = self.rng.range(0..2) as u16;
                }
                self.vel_y += 0x40;

                if self.vel_y > 0x700 {
                    self.vel_y = 0x700
                };
                if self.y > 0x10000 {
                    self.npc_flags.set_ignore_solidity(false)
                };

                if self.flags.hit_bottom_wall() {
                    self.vel_y = -0x200;
                    self.action_num = 110;
                    self.npc_flags.set_ignore_solidity(true);
                    state.sound_manager.play_sfx(12);
                    state.quake_counter = 10;
                    state.quake_rumble_counter = 10;

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);

                    for _ in 0..2 {
                        npc.x = self.x + self.rng.range(-12..12) * 0x200;
                        npc.y = self.y + 0x2000;
                        npc.vel_x = self.rng.range(-0x155..0x155);
                        npc.vel_y = self.rng.range(-0x600..0);
                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }
            }
            110 => {
                self.vel_y += 0x40;

                if self.y > stage.map.height as i32 * state.tile_size.as_int() * 0x200 {
                    self.cond.set_alive(false);
                    return Ok(());
                }
            }
            _ => (),
        }

        self.animate(6, 0, 2);

        self.damage = if player.y > self.y { 10 } else { 0 };

        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n314_ma_pignon_rock[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n315_ma_pignon_clone(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        bullet_manager: &mut BulletManager,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 => {
                self.anim_num = 3;
                self.vel_y += 0x80;
                self.action_counter2 = 0;

                if self.y > 0x10000 {
                    self.action_num = 130;
                    self.npc_flags.set_ignore_solidity(false);
                }
            }
            100 | 110 => {
                if self.action_num == 100 {
                    self.action_num = 110;
                    self.action_counter = 0;
                    self.action_counter2 = 0;
                    self.npc_flags.set_shootable(true);
                }

                self.face_player(player);
                self.anim_num = 0;
                self.action_counter += 1;
                if self.action_counter > 4 {
                    self.action_num = 120;
                    self.action_counter = 0;
                }
            }
            120 => {
                self.anim_num = 1;

                self.action_counter += 1;
                if self.action_counter > 4 {
                    self.action_num = 130;
                    self.anim_num = 3;
                    self.vel_x = 2 * self.rng.range(-0x200..0x200);
                    self.vel_y = -0x800;
                    state.sound_manager.play_sfx(30);
                }
            }
            130 => {
                self.vel_y += 0x80;
                if (self.vel_x < 0 && self.flags.hit_left_wall()) || (self.vel_x > 0 && self.flags.hit_right_wall()) {
                    self.vel_x *= -1
                };
                self.face_player(player);
                self.anim_num = if self.vel_y < -0x200 {
                    2
                } else if self.vel_y > 0x200 {
                    0
                } else {
                    3
                };

                if self.flags.hit_bottom_wall() {
                    self.action_num = 140;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.vel_x = 0;
                }
            }
            140 => {
                self.anim_num = 1;

                self.action_counter += 1;
                if self.action_counter > 4 {
                    // action_counter is not reset here
                    self.action_num = 110;
                    self.npc_flags.set_shootable(true);
                }
            }
            _ => (),
        }

        if self.action_num > 100 {
            // Missiles + Blade
            if bullet_manager.count_bullets_multi(
                &[13, 14, 15, 16, 17, 18, 23, 25, 26, 27, 28, 29, 30, 31, 32, 33],
                TargetPlayer::Player1,
            ) + bullet_manager.count_bullets_multi(
                &[13, 14, 15, 16, 17, 18, 23, 25, 26, 27, 28, 29, 30, 31, 32, 33],
                TargetPlayer::Player2,
            ) > 0
            {
                self.npc_flags.set_shootable(false);
                self.npc_flags.set_invulnerable(true);
            } else {
                self.npc_flags.set_shootable(true);
                self.npc_flags.set_invulnerable(false);
            }
        }

        self.action_counter2 += 1;
        if self.action_counter2 > 300 {
            self.vanish(state);
        } else {
            self.clamp_fall_speed();

            self.x += self.vel_x;
            self.y += self.vel_y;

            let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };
            self.anim_rect = state.constants.npc.n315_ma_pignon_clone[self.anim_num as usize + dir_offset];
        }

        Ok(())
    }
}
