use std::hint::unreachable_unchecked;

use num_traits::clamp;

use crate::common::{CDEG_RAD, Direction};
use crate::components::flash::Flash;
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::{NPCList, NPCRefMut};
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::Stage;
use crate::util::rng::RNG;

impl NPCRefMut<'_> {
    pub(crate) fn tick_n066_misery_bubble(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    if let Some(npc) = self.unborrow_then(|token| { npc_list.iter(token).find(|npc| npc.borrow().event_num == 1000) }) {
                        let npc = npc.borrow();

                        self.action_counter2 = npc.id;
                        self.target_x = npc.x;
                        self.target_y = npc.y;

                        let angle = f64::atan2((self.y - self.target_y) as f64,  (self.x - self.target_x) as f64);
                        self.vel_x = (angle.cos() * -1024.0) as i32;
                        self.vel_y = (angle.sin() * -1024.0) as i32;
                    }

                    if self.action_counter2 == 0 {
                        self.action_num = 0xffff;
                        return Ok(());
                    }

                    self.action_num = 1;
                }

                self.animate(1, 0, 1);

                if (self.x - self.target_x).abs() < 0x600 && (self.y - self.target_y).abs() < 0x600 {
                    self.action_num = 2;
                    self.anim_num = 2;
                    state.sound_manager.play_sfx(21);

                    let npc_id = self.action_counter2 as usize;
                    self.unborrow_then(|token| {
                        if let Some(npc) = npc_list.get_npc(npc_id, token) {
                            let mut npc = npc.borrow_mut(token);

                            npc.cond.set_alive(false);
                        }
                    });
                }
            }
            2 => {
                self.vel_x -= 0x20;
                self.vel_y -= 0x20;

                self.vel_x = clamp(self.vel_x, -0x5ff, 0x5ff);
                self.vel_y = clamp(self.vel_y, -0x5ff, 0x5ff);

                if self.y < -0x1000 {
                    self.cond.set_alive(false);
                }

                self.animate(3, 2, 3);
            }
            _ => (),
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n066_misery_bubble[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n067_misery_floating(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
        flash: &mut Flash,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.target_x = self.x;
                    self.target_y = self.y;

                    state.sound_manager.play_sfx(29);
                }

                self.x = self.target_x + self.rng.range(-1..1) as i32 * 0x200;

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

                self.clamp_fall_speed();

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
                    let mut npc = NPC::create(66, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y - 0x2000;
                    npc.parent_id = self.id; // This NPC doesn't do anything with its parent...but we'll set it anyways

                    let _ = npc_list.spawn(0, npc);
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

                if self.y < -0x1000 {
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
                    flash.set_blink();
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
            _ => (),
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.action_num == 11 || self.action_num == 14 {
            let (frame1, frame2) = match self.action_num {
                11 => (0, 1),
                14 => (2, 3),
                _ => unsafe { unreachable_unchecked() },
            };

            if self.anim_counter > 0 {
                self.anim_counter -= 1;
                self.anim_num = frame2;
            } else {
                if self.rng.range(0..100) == 1 {
                    self.anim_counter = 30;
                }

                self.anim_num = frame1;
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 8 };

        self.anim_rect = state.constants.npc.n067_misery_floating[self.anim_num as usize + dir_offset];

        if self.action_num == 1 && self.anim_counter < 32 {
            self.anim_counter += 1;
            self.anim_rect.bottom = self.anim_counter / 2 + self.anim_rect.bottom - 16;
        }

        Ok(())
    }

    pub(crate) fn tick_n082_misery_standing(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
        flash: &mut Flash,
    ) -> GameResult {
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

                    let mut npc = NPC::create(66, &state.npc_table);
                    npc.x = self.x;
                    npc.y = self.y - 0x2000;
                    npc.parent_id = self.id; // This NPC doesn't do anything with its parent...but we'll set it anyways
                    npc.cond.set_alive(true);

                    let _ = npc_list.spawn(0, npc);
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

                if self.y < -0x1000 {
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
                    self.action_num = 27;
                    self.anim_num = 7;

                    state.sound_manager.play_sfx(101);
                    flash.set_blink();
                }
            }
            27 => {
                self.action_counter += 1;
                if self.action_counter == 50 {
                    self.action_num = 0;
                    self.anim_num = 0;
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

                    let mut npc = NPC::create(11, &state.npc_table);
                    npc.x = self.x + 0x1000;
                    npc.y = self.y - 0x1000;
                    npc.vel_x = 0x600;
                    npc.vel_y = self.rng.range(-0x200..0) as i32;
                    npc.cond.set_alive(true);

                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.action_counter > 50 {
                    self.action_num = 0;
                }
            }
            50 => {
                self.anim_num = 8;
            }
            _ => (),
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.action_num == 11 {
            if self.anim_counter != 0 {
                self.anim_counter -= 1;
                self.anim_num = 1;
            } else {
                if self.rng.range(0..100) == 1 {
                    self.anim_counter = 30;
                }

                self.anim_num = 0;
            }
        }

        if self.action_num == 14 {
            if self.action_counter != 0 {
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

    pub(crate) fn tick_n247_misery_boss(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.y += 0xc00;
                    self.target_y = 0x8000;
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
            20 => {
                self.vel_x = 0;
                self.vel_y += 0x40;

                if self.flags.hit_bottom_wall() {
                    self.action_num = 21;
                    self.anim_num = 2;
                }
            }
            21 => {
                if self.rng.range(0..120) == 10 {
                    self.action_num = 22;
                    self.action_counter = 0;
                    self.anim_num = 3;
                }
            }
            22 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 21;
                    self.anim_num = 2;
                }
            }
            100 | 101 => {
                if self.action_num == 100 {
                    self.action_num = 101;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.vel_x = 0;
                    self.npc_flags.set_shootable(true);
                    self.action_counter2 = self.life;
                }

                let player = self.get_closest_player_ref(&players);

                if player.x >= self.x {
                    self.direction = Direction::Right;
                } else {
                    self.direction = Direction::Left;
                }

                self.vel_y += if self.y >= self.target_y { -0x20 } else { 0x20 };
                self.vel_y = self.vel_y.clamp(-0x200, 0x200);

                self.action_counter += 1;
                if self.action_counter > 200 || (self.life + 80) <= self.action_counter2 {
                    self.action_counter = 0;
                    self.action_num = 110;
                }
            }
            110 | 111 => {
                if self.action_num == 110 {
                    self.action_num = 111;
                    self.action_counter = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.npc_flags.set_shootable(false);
                }

                self.action_counter += 1;
                self.anim_num = if (self.action_counter & 1) != 0 { 5 } else { 6 };

                if self.action_counter > 30 {
                    self.action_counter3 += 1;
                    self.action_num = if self.action_counter3 % 3 == 0 { 113 } else { 112 };

                    self.action_counter = 0;
                    self.anim_num = 4;
                }
            }
            112 => {
                self.action_counter += 1;
                if self.action_counter % 6 == 0 {
                    let player = self.get_closest_player_ref(&players);
                    let angle = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64)
                        + self.rng.range(-4..4) as f64 * CDEG_RAD;

                    let mut npc = NPC::create(248, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y + 0x800;
                    npc.vel_x = (angle.cos() * -2048.0) as i32;
                    npc.vel_y = (angle.sin() * -2048.0) as i32;

                    let _ = npc_list.spawn(0x100, npc);
                    state.sound_manager.play_sfx(34);
                }

                if self.action_counter > 30 {
                    self.action_num = 150;
                    self.action_counter = 0;
                }
            }
            113 => {
                self.action_counter += 1;
                if self.action_counter == 10 {
                    let player = self.get_closest_player_ref(&players);

                    let mut npc = NPC::create(279, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = player.x;
                    npc.y = player.y - 0x8000;
                    npc.direction = Direction::Up;
                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.action_counter > 30 {
                    self.action_num = 150;
                    self.action_counter = 0;
                }
            }
            150 | 151 => {
                if self.action_num == 150 {
                    self.action_num = 151;
                    self.action_counter = 0;
                    self.anim_num = 7;

                    let mut npc = NPC::create(249, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    let _ = npc_list.spawn(0x100, npc.clone());

                    npc.direction = Direction::Right;
                    let _ = npc_list.spawn(0x100, npc);

                    self.target_x = self.rng.range(9..31) * 0x2000;
                    self.target_y = self.rng.range(5..7) * 0x2000;

                    state.sound_manager.play_sfx(29);
                }

                self.action_counter += 1;
                if self.action_counter == 42 {
                    let mut npc = NPC::create(249, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.target_x + 0x2000;
                    npc.y = self.target_y;
                    let _ = npc_list.spawn(0x100, npc.clone());

                    npc.x = self.target_x - 0x2000;
                    npc.direction = Direction::Right;
                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.action_counter > 50 {
                    self.action_counter = 0;
                    self.vel_y = -0x200;
                    self.npc_flags.set_shootable(true);
                    self.x = self.target_x;
                    self.y = self.target_y;

                    if self.life < 340 {
                        let mut npc = NPC::create(252, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.parent_id = self.id;
                        let _ = npc_list.spawn(0x100, npc.clone());

                        npc.tsc_direction = 0x80;
                        let _ = npc_list.spawn(0x100, npc);
                    }

                    if self.life < 180 {
                        let mut npc = NPC::create(252, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.parent_id = self.id;
                        npc.tsc_direction = 0x40;
                        let _ = npc_list.spawn(0x100, npc.clone());

                        npc.tsc_direction = 0xc0;
                        let _ = npc_list.spawn(0x100, npc);
                    }

                    let player = self.get_closest_player_ref(&players);
                    self.action_num = if player.x >= self.x - 0xe000 && player.x <= self.x + 0xe000 { 100 } else { 160 };
                }
            }
            160 | 161 => {
                if self.action_num == 160 {
                    self.action_num = 161;
                    self.action_counter = 0;
                    self.anim_num = 4;

                    let player = self.get_closest_player_ref(&players);
                    if player.x >= self.x {
                        self.direction = Direction::Right;
                    } else {
                        self.direction = Direction::Left;
                    }
                }

                self.vel_y += if self.y >= self.target_y { -0x20 } else { 0x20 };
                self.vel_y = self.vel_y.clamp(-0x200, 0x200);

                self.action_counter += 1;
                if self.action_counter % 24 == 0 {
                    let mut npc = NPC::create(250, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y + 0x800;
                    let _ = npc_list.spawn(0x100, npc);

                    state.sound_manager.play_sfx(34);
                }

                if self.action_counter > 72 {
                    self.action_counter = 0;
                    self.action_num = 100;
                }
            }

            1000 | 1001 => {
                if self.action_num == 1000 {
                    self.npc_flags.set_shootable(false);
                    self.action_num = 1001;
                    self.action_counter = 0;
                    self.anim_num = 4;

                    self.target_x = self.x;
                    self.target_y = self.y;

                    self.vel_x = 0;
                    self.vel_y = 0;

                    self.unborrow_then(|token| {
                        npc_list.kill_npcs_by_type(252, true, state, token);
                    });

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    for _ in 0..3 {
                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }

                self.action_counter += 1;
                self.x = if (self.action_counter & 0x02) != 0 { self.target_x + 0x200 } else { self.target_x };
            }
            1010 => {
                self.vel_y += 0x10;

                if self.flags.hit_bottom_wall() {
                    self.action_num = 1020;
                    self.anim_num = 8;
                }
            }
            _ => (),
        }

        self.vel_x = self.vel_x.clamp(-0x200, 0x200);
        self.vel_y = self.vel_y.clamp(-0x400, 0x400);

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 9 };

        self.anim_rect = state.constants.npc.n247_misery_boss[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n248_misery_boss_vanishing(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.flags.hit_anything() {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        self.y += self.vel_y;
        self.x += self.vel_x;

        self.animate(1, 0, 2);
        self.anim_rect = state.constants.npc.n248_misery_boss_vanishing[self.anim_num as usize];

        self.action_counter3 += 1;
        if self.action_counter3 > 300 {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        Ok(())
    }

    pub(crate) fn tick_n249_misery_boss_appearing(&mut self, state: &mut SharedGameState) -> GameResult {
        self.action_counter2 += 1;
        if self.action_counter2 > 8 {
            self.cond.set_alive(false);
        }

        if self.direction == Direction::Left {
            self.x -= 0x400;
            self.anim_rect = state.constants.npc.n249_misery_boss_appearing[0];
        } else {
            self.x += 0x400;
            self.anim_rect = state.constants.npc.n249_misery_boss_appearing[1];
        }

        Ok(())
    }

    pub(crate) fn tick_n250_misery_boss_lightning_ball(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.target_y = self.y;
                    self.vel_x = 0;
                    self.vel_y = -0x200;
                }

                let player = self.get_closest_player_ref(&players);

                self.vel_x += if self.x < player.x { 0x10 } else { -0x10 };
                self.vel_y += if self.y < self.target_y { 0x20 } else { -0x20 };

                self.vel_x = self.vel_x.clamp(-0x200, 0x200);
                self.vel_y = self.vel_y.clamp(-0x200, 0x200);

                self.x += self.vel_x;
                self.y += self.vel_y;

                self.animate(2, 0, 1);

                if player.x > self.x - 0x1000 && player.x < self.x + 0x1000 && player.y > self.y {
                    self.action_num = 10;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 10 {
                    let mut npc = NPC::create(251, &state.npc_table);
                    npc.cond.set_alive(true);

                    npc.x = self.x;
                    npc.y = self.y;

                    let _ = npc_list.spawn(0x100, npc);

                    state.sound_manager.play_sfx(101);
                    self.cond.set_alive(false);

                    return Ok(());
                }

                self.anim_num = if (self.action_counter & 2) != 0 { 2 } else { 1 };
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n250_misery_boss_lightning_ball[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n251_misery_boss_lightning(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
        }

        if self.action_num == 1 {
            self.anim_num = (self.anim_num + 1) & 1;
            self.y += 0x1000;

            if self.flags.hit_anything() {
                npc_list.create_death_smoke(self.x, self.y, self.display_bounds.right as usize, 3, state, &mut self.rng);
                self.cond.set_alive(false);
            }
        }

        self.anim_rect = state.constants.npc.n251_misery_boss_lightning[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n252_misery_boss_bats(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.action_counter2 = self.tsc_direction;
                }

                self.action_counter2 += 2;
                self.action_counter2 &= 0xff;

                if self.action_counter < 192 {
                    self.action_counter += 1;
                }

                if let Some(parent) = self.get_parent_ref(npc_list) {
                    let parent = parent.borrow();
                    
                    self.x = parent.x
                        + self.action_counter as i32 * ((self.action_counter2 as f64 * CDEG_RAD).cos() * 512.0) as i32
                        / 4;
                    self.y = parent.y
                        + self.action_counter as i32 * ((self.action_counter2 as f64 * CDEG_RAD).sin() * 512.0) as i32
                        / 4;

                    if parent.action_num == 151 {
                        self.action_num = 10;
                        self.anim_num = 0;
                    }
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.npc_flags.set_shootable(true);
                    self.npc_flags.set_invulnerable(false);
                    self.npc_flags.set_ignore_solidity(false);

                    let player = self.get_closest_player_ref(&players);
                    let deg = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64);
                    let deg = deg + self.rng.range(-3..3) as f64 * CDEG_RAD;

                    self.vel_x = (deg.cos() * -512.0) as i32;
                    self.vel_y = (deg.sin() * -512.0) as i32;

                    self.anim_num = 1;
                    self.anim_counter = 0;

                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
                }

                self.x += self.vel_x;
                self.y += self.vel_y;

                if self.flags.hit_anything() {
                    let mut npc = NPC::create(4, &state.npc_table);

                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;

                    let _ = npc_list.spawn(0x100, npc.clone());
                    let _ = npc_list.spawn(0x100, npc.clone());
                    let _ = npc_list.spawn(0x100, npc);

                    self.cond.set_alive(false);
                }

                self.animate(4, 1, 3);
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.npc.n252_misery_boss_bats[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n283_misery_possessed(
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
                    self.y -= 0x1000;
                    state.sound_manager.play_sfx(29);
                }

                self.action_counter += 1;
                self.anim_num = if self.action_counter & 2 != 0 { 9 } else { 0 };
            }
            10 => {
                self.action_num = 11;
                self.anim_num = 9;
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.anim_counter = 0;

                    state.npc_super_pos.0 = 0;
                }

                self.vel_x = 7 * self.vel_x / 8;
                self.vel_y = 7 * self.vel_y / 8;

                self.animate(20, 0, 1);

                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 30;
                }

                let player = self.get_closest_player_ref(&players);

                self.direction = if player.x > self.x { Direction::Right } else { Direction::Left };
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.action_counter3 = self.life;
                }

                self.animate(1, 2, 3);

                if self.flags.hit_bottom_wall() {
                    self.vel_y = -0x200;
                }

                let player = self.get_closest_player_ref(&players);

                self.vel_x += if self.x > boss.parts[0].x { -0x20 } else { 0x20 };
                self.vel_y += if self.y > player.y { -0x10 } else { 0x10 };

                self.vel_x = self.vel_x.clamp(-0x200, 0x200);
                self.vel_y = self.vel_y.clamp(-0x200, 0x200);

                self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };

                self.action_counter += 1;
                if self.action_counter > 150 && (self.life + 20 < self.action_counter3 || state.npc_super_pos.0 != 0) {
                    state.npc_super_pos.0 = 0;
                    self.action_num = 40;
                }

                if boss.parts[0].anim_num != 0 && self.action_counter > 250 {
                    self.action_num = 50;
                }
            }
            40 | 41 => {
                if self.action_num == 40 {
                    self.action_num = 41;
                    self.action_counter = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;

                    state.sound_manager.play_sfx(103);

                    let player = self.get_closest_player_ref(&players);
                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
                    self.action_counter3 = if player.y >= 0x14000 { 289 } else { 290 };
                }

                self.action_counter += 1;
                self.anim_num = if self.action_counter & 2 != 0 { 4 } else { 5 };

                if self.action_counter % 6 == 1 {
                    state.sound_manager.play_sfx(39);

                    let mut npc = NPC::create(self.action_counter3, &state.npc_table);
                    npc.cond.set_alive(true);

                    if self.action_counter3 == 289 {
                        npc.x = self.x + self.rng.range(-64..64) * 0x200;
                        npc.y = self.y + self.rng.range(-32..32) * 0x200;
                    } else {
                        npc.x = self.x + self.rng.range(-32..32) * 0x200;
                        npc.y = self.y + self.rng.range(-64..64) * 0x200;
                    }

                    npc.x = npc.x.clamp(
                        2 * state.tile_size.as_int() * 0x200,
                        (stage.map.width as i32 - 2) * state.tile_size.as_int() * 0x200,
                    );
                    npc.y = npc.y.clamp(
                        2 * state.tile_size.as_int() * 0x200,
                        (stage.map.height as i32 - 2) * state.tile_size.as_int() * 0x200,
                    );

                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.action_counter > 50 {
                    self.action_num = 42;
                    self.action_counter = 0;

                    let player = self.get_closest_player_ref(&players);
                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
                }
            }
            42 => {
                self.action_counter += 1;
                self.anim_num = 6;
                if self.action_counter > 50 {
                    self.action_num = 30;
                    self.vel_y = -0x200;
                    self.vel_x = self.direction.opposite().vector_x() * 0x200;
                }
            }
            50 | 51 => {
                let player = self.get_closest_player_ref(&players);

                if self.action_num == 50 {
                    self.action_num = 51;
                    self.action_counter = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };

                    state.sound_manager.play_sfx(103);
                }

                self.action_counter += 1;
                self.anim_num = if self.action_counter & 2 != 0 { 4 } else { 5 };

                let period = if player.equip.has_booster_2_0() { 10 } else { 24 };
                if self.action_counter % period == 1 {
                    state.sound_manager.play_sfx(39);

                    let mut npc = NPC::create(301, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x + 0x1400 * self.direction.opposite().vector_x();
                    npc.y = self.y;
                    npc.tsc_direction = match ((self.action_counter / 6) & 3, self.direction) {
                        (0, Direction::Left) => 0xD8,
                        (1, Direction::Left) => 0xEC,
                        (2, Direction::Left) => 0x14,
                        (3, Direction::Left) => 0x28,
                        (0, _) => 0x58,
                        (1, _) => 0x6C,
                        (2, _) => 0x94,
                        (3, _) => 0xA8,
                        _ => unsafe {
                            unreachable_unchecked();
                        },
                    };

                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.action_counter > 50 {
                    self.action_num = 42;
                    self.action_counter = 0;

                    let player = self.get_closest_player_ref(&players);
                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
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
                    self.vel_y = -0x200;
                    self.shock += 50;
                    self.hit_bounds.bottom = 0x1800;
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

        let dir_offset = if self.direction == Direction::Left { 0 } else { 11 };

        self.anim_rect = state.constants.npc.n283_misery_possessed[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n289_critter_orange(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        stage: &mut Stage,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 2;

                    let player = self.get_closest_player_ref(&players);
                    self.direction = if player.x > self.x { Direction::Left } else { Direction::Right };
                }

                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_num = 10;
                    self.display_bounds.top = 0x1000;
                    self.display_bounds.bottom = 0x1000;
                    self.damage = 2;
                    self.npc_flags.set_shootable(true);
                }
            }
            10 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 11;
                    self.anim_num = 0;
                    self.action_counter = 0;
                    self.vel_x = 0;

                    let player = self.get_closest_player_ref(&players);
                    self.direction = if player.x > self.x { Direction::Left } else { Direction::Right };
                }
            }
            11 => {
                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.anim_num = 2;
                    self.action_counter2 += 1;
                    self.action_num = if self.action_counter2 > 4 { 12 } else { 10 };

                    state.sound_manager.play_sfx(30);
                    self.vel_x = self.direction.vector_x() * 0x200;
                    self.vel_y = -0x600;
                }
            }
            12 => {
                self.npc_flags.set_ignore_solidity(true);

                if self.y > stage.map.height as i32 * state.tile_size.as_int() * 0x200 {
                    self.vanish(state);
                    return Ok(());
                }
            }
            _ => (),
        }

        if self.action_num >= 10 {
            self.vel_y += 0x40;
        }

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n289_critter_orange[self.anim_num as usize + dir_offset];

        if self.action_num == 1 {
            self.anim_rect.top += 8 - self.action_counter / 2;
            self.anim_rect.bottom -= 8 + self.action_counter / 2;
            self.display_bounds.top = (self.action_counter as u32 * 0x200) / 2;
            self.display_bounds.bottom = (self.action_counter as u32 * 0x200) / 2;
        }

        Ok(())
    }

    pub(crate) fn tick_n290_bat_misery(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        stage: &mut Stage,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 2;

                    let player = self.get_closest_player_ref(&players);
                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
                }

                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_num = 10;
                    self.display_bounds.top = 0x1000;
                    self.display_bounds.bottom = 0x1000;
                    self.damage = 2;
                    self.npc_flags.set_shootable(true);
                    self.target_y = self.y;
                    self.vel_y = 0x400;
                }
            }
            10 => {
                self.animate(2, 0, 2);

                self.vel_y += if self.y >= self.target_y { -0x40 } else { 0x40 };
                self.vel_x += if self.direction == Direction::Left { -0x10 } else { 0x10 };

                if self.x < 0
                    || self.y < 0
                    || self.x > (stage.map.width as i32 * state.tile_size.as_int() * 0x200)
                    || self.y > (stage.map.height as i32 * state.tile_size.as_int() * 0x200)
                {
                    self.vanish(state);
                    return Ok(());
                }
            }
            _ => (),
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n290_bat_misery[self.anim_num as usize + dir_offset];

        if self.action_num == 1 {
            self.anim_rect.top += 8 - self.action_counter / 2;
            self.anim_rect.bottom -= 8 + self.action_counter / 2;
            self.display_bounds.top = (self.action_counter as u32 * 0x200) / 2;
            self.display_bounds.bottom = (self.action_counter as u32 * 0x200) / 2;
        }

        Ok(())
    }

    pub(crate) fn tick_n301_misery_fish_missile(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter2 = self.tsc_direction;
                }

                let radians = self.action_counter2 as f64 * CDEG_RAD;
                self.vel_x = 2 * (radians.cos() * 512.0) as i32;
                self.vel_y = 2 * (radians.sin() * 512.0) as i32;
                self.x += self.vel_x;
                self.y += self.vel_y;

                let player = self.get_closest_player_mut(players);
                let direction = f64::atan2(-(self.y - player.y) as f64, -(self.x - player.x) as f64);

                if direction < radians {
                    if radians - direction < std::f64::consts::PI {
                        self.action_counter2 = self.action_counter2.wrapping_sub(1) & 0xff;
                    } else {
                        self.action_counter2 = (self.action_counter2 + 1) & 0xff;
                    }
                } else if direction - radians < std::f64::consts::PI {
                    self.action_counter2 = (self.action_counter2 + 1) & 0xff;
                } else {
                    self.action_counter2 = self.action_counter2.wrapping_sub(1) & 0xff;
                }
            }
            _ => (),
        }

        self.anim_counter += 1;
        if self.anim_counter > 2 {
            self.anim_counter = 0;
            state.create_caret(self.x, self.y, CaretType::Exhaust, Direction::FacingPlayer);
        }

        self.anim_num = (self.action_counter2 + 0x10) / 0x20;

        if self.anim_num > 7 {
            self.anim_num = 7;
        }

        self.anim_rect = state.constants.npc.n301_misery_fish_missile[self.anim_num as usize];

        Ok(())
    }
}
