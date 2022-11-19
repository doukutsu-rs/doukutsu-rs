use crate::common::{CDEG_RAD, Direction, Rect};
use crate::framework::error::GameResult;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::Stage;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n139_doctor(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.y += -0x1000;
                }

                if !self.flags.hit_bottom_wall() {
                    self.anim_num = 2;
                } else {
                    self.anim_num = 0;
                }

                self.vel_y += 0x40;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 0xb;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                    self.action_counter3 = 0;
                }

                self.anim_counter += 1;
                if 6 < self.anim_counter {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }

                if 1 < self.anim_num {
                    self.anim_num = 0;
                    self.action_counter3 += 1;
                }

                if 8 < self.action_counter3 {
                    self.anim_num = 0;
                    self.action_num = 1;
                }
            }
            0x14 | 0x15 => {
                if self.action_num == 0x14 {
                    self.action_num = 0x15;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.target_y = self.y + -0x4000;
                }

                if self.y < self.target_y {
                    self.vel_y += 0x20;
                } else {
                    self.vel_y += -0x20;
                }

                self.vel_y = self.vel_y.clamp(-0x200, 0x200);
            }
            0x1e | 0x1f => {
                if self.action_num == 0x1e {
                    self.action_num = 0x1f;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.action_counter = (self.anim_rect.bottom - self.anim_rect.top) * 2;
                    state.sound_manager.play_sfx(0x1d);
                }

                self.action_counter = self.action_counter.saturating_sub(1);
                self.anim_num = 0;

                if self.action_counter == 0 {
                    self.cond.set_alive(false);
                }
            }
            0x28 | 0x29 => {
                if self.action_num == 0x28 {
                    self.action_num = 0x29;
                    self.action_counter = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    state.sound_manager.play_sfx(0x1d);
                }
                self.anim_num = 2;
                self.action_counter += 1;
                if 0x3f < self.action_counter {
                    self.action_num = 0x14;
                }
            }
            _ => (),
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n139_doctor[self.anim_num as usize + dir_offset];

        if self.action_num == 31 || self.action_num == 41 {
            self.anim_rect.bottom = self.action_counter / 2 + self.anim_rect.top;
            if ((self.action_counter / 2) & 1) != 0 {
                self.anim_rect.left += 1;
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n256_doctor_facing_away(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.y -= 0x1000;

                    state.npc_super_pos.0 = 0;
                }

                self.anim_num = 0;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.action_counter2 = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 5 {
                    self.anim_counter = 0;

                    self.anim_num += 1;
                    if self.anim_num > 1 {
                        self.anim_num = 0;

                        self.action_counter2 += 1;
                        if self.action_counter2 > 5 {
                            self.action_num = 1;
                        }
                    }
                }
            }
            20 | 21 => {
                self.action_num = 21;
                self.anim_num = 2;
            }
            40 | 41 => {
                if self.action_num == 40 {
                    self.action_num = 41;

                    let mut npc = NPC::create(257, &state.npc_table);
                    npc.cond.set_alive(true);

                    npc.x = self.x - 0x1c00;
                    npc.y = self.y - 0x2000;

                    let _ = npc_list.spawn(0x100, npc.clone());

                    npc.direction = Direction::Right;
                    let _ = npc_list.spawn(0xaa, npc);
                }

                self.anim_num = 4;
            }
            50 | 51 => {
                if self.action_num == 50 {
                    self.action_num = 51;
                    self.anim_num = 4;
                    self.anim_counter = 0;
                    self.action_counter2 = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 5 {
                    self.anim_counter = 0;

                    self.anim_num += 1;
                    if self.anim_num > 5 {
                        self.anim_num = 4;

                        self.action_counter2 += 1;
                        if self.action_counter2 > 5 {
                            self.action_num = 41;
                        }
                    }
                }
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n256_doctor_facing_away[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n257_red_crystal(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
        }

        if self.action_num == 1 {
            if state.npc_super_pos.0 != 0 {
                self.action_num = 10;
            }
        } else if self.action_num == 10 {
            if self.x < state.npc_super_pos.0 {
                self.vel_x += 0x55;
            }
            if self.x > state.npc_super_pos.0 {
                self.vel_x -= 0x55;
            }
            if self.y < state.npc_super_pos.1 {
                self.vel_y += 0x55;
            }
            if self.y > state.npc_super_pos.1 {
                self.vel_y -= 0x55;
            }

            self.vel_x = self.vel_x.clamp(-0x400, 0x400);
            self.vel_y = self.vel_y.clamp(-0x400, 0x400);

            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        self.animate(3, 0, 1);

        if self.direction == Direction::Left && self.vel_x > 0 {
            self.anim_num = 2;
        }
        if self.direction == Direction::Right && self.vel_x < 0 {
            self.anim_num = 2;
        }

        self.anim_rect = state.constants.npc.n257_red_crystal[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n263_doctor_boss(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 3;
                self.y += 0x1000;
            }
            2 => {
                self.action_counter += 1;
                self.anim_num = if (self.action_counter & 2) != 0 { 0 } else { 3 };

                if self.action_counter > 50 {
                    self.action_num = 10;
                }
            }
            10 => {
                self.vel_y += 0x80;
                self.npc_flags.set_shootable(true);
                self.damage = 3;

                if self.flags.hit_bottom_wall() {
                    self.action_num = 20;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.action_counter3 = self.life;

                    let player = self.get_closest_player_ref(&players);
                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
                }
            }
            20 => {
                self.action_counter += 1;
                if self.action_counter < 50 && (20 + self.life) < self.action_counter3 {
                    self.action_counter = 50;
                }

                if self.action_counter == 50 {
                    let player = self.get_closest_player_ref(&players);
                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
                    self.anim_num = 4;
                }

                if self.action_counter == 80 {
                    self.anim_num = 5;
                    state.sound_manager.play_sfx(25);

                    let mut npc = NPC::create(264, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.y = self.y;

                    if self.direction == Direction::Left {
                        npc.x = self.x - 0x2000;
                        npc.tsc_direction = 0x0;

                        let _ = npc_list.spawn(0x100, npc.clone());

                        npc.tsc_direction = 0x400;
                        let _ = npc_list.spawn(0x100, npc.clone());
                    } else {
                        npc.x = self.x + 0x2000;
                        npc.tsc_direction = 0x2;

                        let _ = npc_list.spawn(0x100, npc.clone());

                        npc.tsc_direction = 0x402;
                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }

                if self.action_counter == 120 {
                    self.anim_num = 0;
                }

                if self.action_counter > 130 && (50 + self.life) < self.action_counter3 {
                    self.action_counter = 161;
                }

                if self.action_counter > 160 {
                    self.action_num = 100;
                    self.anim_num = 0;
                }
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 0;
                    self.anim_num = 6;
                    self.target_x = self.x;
                    self.npc_flags.set_shootable(true);
                }

                self.action_counter += 1;
                self.x = if (self.action_counter & 2) != 0 { self.target_x } else { self.target_x + 0x200 };

                if self.action_counter > 50 {
                    self.action_num = 32;
                    self.action_counter = 0;
                    self.anim_num = 7;

                    state.sound_manager.play_sfx(101);

                    let mut npc = NPC::create(266, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;

                    for i in (8..256).step_by(16) {
                        npc.vel_x = ((i as f64 * CDEG_RAD).cos() * -1024.0) as i32;
                        npc.vel_y = ((i as f64 * CDEG_RAD).sin() * -1024.0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }
            }
            32 => {
                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 100;
                    self.anim_num = 0;
                }
            }
            100 | 101 => {
                if self.action_num == 100 {
                    self.action_num = 101;
                    self.npc_flags.set_shootable(false);
                    self.damage = 0;
                    self.action_counter = 0;

                    state.sound_manager.play_sfx(29);
                }

                self.action_counter += 2;
                if self.action_counter > 16 {
                    self.action_num = 102;
                    self.action_counter = 0;
                    self.anim_num = 3;
                    self.target_x = self.rng.range(5..35) * 0x2000;
                    self.target_y = self.rng.range(5..7) * 0x2000;
                }
            }
            102 => {
                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.action_num = 103;
                    self.action_counter = 16;
                    self.anim_num = 2;
                    self.vel_y = 0;
                    self.x = self.target_x;
                    self.y = self.target_y;

                    let player = self.get_closest_player_ref(&players);
                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
                }
            }
            103 => {
                self.action_counter = self.action_counter.saturating_sub(2);
                if self.action_counter == 0 {
                    self.npc_flags.set_shootable(true);
                    self.damage = 3;
                    if self.action_counter2 < 4 {
                        self.action_num = 10;
                        self.action_counter2 += 1;
                    } else {
                        self.action_num = 30;
                        self.action_counter2 = 0;
                    }
                }
            }
            500 => {
                self.npc_flags.set_shootable(false);
                self.anim_num = 6;
                self.vel_y += 0x10;

                if self.flags.hit_bottom_wall() {
                    self.action_num = 501;
                    self.action_counter = 0;
                    self.target_x = self.x;

                    let player = self.get_closest_player_ref(&players);
                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
                }
            }
            501 => {
                let player = self.get_closest_player_ref(&players);
                self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };

                self.anim_num = 8;
                self.action_counter += 1;
                self.x = if (self.action_counter & 2) != 0 { self.target_x } else { self.target_x + 0x200 };
            }
            _ => (),
        }

        if self.action_num == 102 {
            state.npc_super_pos = (self.target_x, self.target_y);
        } else if self.action_num >= 10 {
            state.npc_super_pos = (self.x, self.y);
        }

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 9 };

        self.anim_rect = state.constants.npc.n263_doctor_boss[self.anim_num as usize + dir_offset];

        if self.action_num == 101 || self.action_num == 103 {
            self.anim_rect.top += self.action_counter;
            self.anim_rect.bottom -= self.action_counter;
            self.display_bounds.top = (16 - self.action_counter as u32) * 0x200;
        } else {
            self.display_bounds.top = 0x2000;
        }

        Ok(())
    }

    pub(crate) fn tick_n264_doctor_boss_red_projectile(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
        stage: &mut Stage,
    ) -> GameResult {
        if self.x < 0 || self.x > stage.map.width as i32 * state.tile_size.as_int() * 0x200 {
            self.vanish(state);
            return Ok(());
        }

        if self.action_num == 0 {
            self.action_num = 1;
            self.target_x = self.x;
            self.target_y = self.y;
            self.action_counter2 = self.tsc_direction / 8;
            self.tsc_direction %= 8;
        }

        if self.action_num == 1 {
            self.action_counter2 += 6;
            self.action_counter2 &= 0xff;

            if self.action_counter < 128 {
                self.action_counter += 1;
            }

            self.vel_x += if self.tsc_direction != 0 { 0x15 } else { -0x15 };
            self.target_x += self.vel_x;

            let angle = self.action_counter2 as f64 * CDEG_RAD;
            self.x = self.target_x + self.action_counter as i32 * (angle.cos() * -512.0) as i32 / 8;
            self.y = self.target_y + self.action_counter as i32 * (angle.sin() * -512.0) as i32 / 2;

            let mut npc = NPC::create(265, &state.npc_table);
            npc.cond.set_alive(true);

            npc.x = self.x;
            npc.y = self.y;

            let _ = npc_list.spawn(0x100, npc);
        }

        self.anim_rect = state.constants.npc.n264_doctor_boss_red_projectile;

        Ok(())
    }

    pub(crate) fn tick_n265_doctor_boss_red_projectile_trail(&mut self, state: &mut SharedGameState) -> GameResult {
        self.anim_counter += 1;
        if self.anim_counter > 3 {
            self.anim_counter = 0;
            self.anim_num += 1;
        }

        if self.anim_num > 2 {
            self.cond.set_alive(false);
        } else {
            self.anim_rect = state.constants.npc.n265_doctor_boss_red_projectile_trail[self.anim_num as usize];
        }

        Ok(())
    }

    pub(crate) fn tick_n266_doctor_boss_red_projectile_bouncing(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        if self.flags.hit_left_wall() {
            self.vel_x = -self.vel_x;
        }

        if self.flags.hit_right_wall() {
            self.vel_x = -self.vel_x;
        }

        if self.flags.hit_top_wall() {
            self.vel_y = 0x200;
        }

        if self.flags.hit_bottom_wall() {
            self.vel_y = -0x200;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_num += 1;
        if self.anim_num > 1 {
            self.anim_num = 0;
        }

        self.anim_rect = state.constants.npc.n266_doctor_boss_red_projectile_bouncing[self.anim_num as usize];

        self.action_counter += 1;
        if self.action_counter % 4 == 1 {
            let mut npc = NPC::create(265, &state.npc_table);
            npc.cond.set_alive(true);

            npc.x = self.x;
            npc.y = self.y;

            let _ = npc_list.spawn(0x100, npc);
        }

        if self.action_counter > 250 {
            self.vanish(state);
        }

        Ok(())
    }

    pub(crate) fn tick_n267_muscle_doctor(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 | 2 => {
                if self.action_num == 0 {
                    let player = self.get_closest_player_ref(&players);
                    self.direction = if state.npc_super_pos.0 > player.x { Direction::Left } else { Direction::Right };

                    self.x = state.npc_super_pos.0 + self.direction.vector_x() * 0xc00;
                    self.y = state.npc_super_pos.1;
                }

                if self.action_num == 1 {
                    self.action_num = 2;
                }

                self.vel_y += 0x80;
                self.action_counter += 1;
                self.anim_num = if (self.action_counter & 2) != 0 { 0 } else { 3 };
            }
            5 | 6 => {
                if self.action_num == 5 {
                    self.action_num = 6;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }

                self.vel_y += 0x80;
                self.animate(40, 1, 2);
            }
            7 | 8 => {
                if self.action_num == 7 {
                    self.action_num = 8;
                    self.action_counter = 0;
                    self.anim_num = 3;
                }

                self.vel_y += 0x40;
                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.action_num = 10;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.npc_flags.set_invulnerable(true);
                    self.vel_x = 0;
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                    self.action_counter3 = self.life;
                }

                self.vel_y += 0x80;
                let player = self.get_closest_player_mut(players);
                self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };

                if self.flags.hit_bottom_wall() {
                    if self.life + 20 > self.action_counter3 {
                        self.animate(10, 1, 2);
                    } else if player.flags.hit_bottom_wall() && player.x > self.x - 0x6000 && player.x < self.x + 0x6000
                    {
                        self.anim_num = 6;
                        state.quake_counter = 10;
                        state.quake_rumble_counter = 10;
                        state.sound_manager.play_sfx(26);

                        player.damage(5, state, npc_list);
                        player.vel_y = -0x400;
                        player.vel_x = if self.x > player.x { -0x5FF } else { 0x5FF };

                        let mut npc = NPC::create(270, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.direction = Direction::Bottom;

                        for _ in 0..100 {
                            npc.x = self.x + (self.rng.range(-16..16) as i32) * 0x200;
                            npc.y = self.y + (self.rng.range(-16..16) as i32) * 0x200;
                            npc.vel_x = 3 * self.rng.range(-512..512);
                            npc.vel_y = 3 * self.rng.range(-512..512);

                            let _ = npc_list.spawn(0xaa, npc.clone());
                        }
                    }
                } else {
                    self.anim_num = 4;
                }

                self.action_counter += 1;
                if self.action_counter > 30 || self.life + 20 < self.action_counter3 {
                    self.action_counter2 += 1;
                    if self.action_counter2 > 10 {
                        self.action_counter2 = 0;
                    }

                    match self.action_counter2 {
                        1 | 9 => {
                            self.action_num = 40;
                        }
                        2 | 7 => {
                            self.action_num = 100;
                        }
                        3 | 6 => {
                            self.action_num = 30;
                        }
                        8 => {
                            self.action_num = 20;
                        }
                        _ => {
                            self.action_num = 15;
                            self.action_counter = 0;
                        }
                    }
                }
            }
            15 => {
                self.anim_num = 3;
                self.action_counter += 1;

                let player = self.get_closest_player_ref(&players);
                self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };

                if self.action_counter > 20 {
                    self.action_num = 16;
                    self.anim_num = 4;
                    self.anim_counter = 0;

                    self.vel_x = self.direction.vector_x() * 0x400;
                    self.vel_y = -0x600;
                }
            }
            16 => {
                self.vel_y += 0x40;
                self.animate(1, 4, 5);

                let player = self.get_closest_player_ref(&players);
                self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };

                if self.vel_y > 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 17;
                }
            }
            17 | 18 => {
                if self.action_num == 17 {
                    self.action_num = 18;
                    self.action_counter = 0;

                    state.quake_counter = 10;
                    state.quake_rumble_counter = 10;
                    state.sound_manager.play_sfx(26);
                }

                self.anim_num = 3;
                self.action_counter += 1;
                self.vel_x = 7 * self.vel_x / 8;
                self.vel_y += 0x80;

                if self.action_counter > 10 {
                    self.action_num = 10;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                }

                self.anim_num = 6;
                self.action_counter += 1;
                if self.action_counter > 20 && self.action_counter % 3 == 1 {
                    let mut npc = NPC::create(269, &state.npc_table);

                    npc.cond.set_alive(true);
                    npc.x = self.x + if self.direction == Direction::Left { -0x1000 } else { 0x1000 };
                    npc.y = self.y - 0x800;
                    npc.vel_x = 4 * self.rng.range(256..512) * self.direction.vector_x();
                    npc.vel_y = self.rng.range(-512..512);
                    npc.direction = self.direction;

                    let _ = npc_list.spawn(0x100, npc);
                    state.sound_manager.play_sfx(39);
                }

                if self.action_counter > 90 {
                    self.action_num = 10;
                }
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 0;

                    self.npc_flags.set_solid_soft(true);
                    self.npc_flags.set_shootable(false);
                }

                self.anim_num = 3;
                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 32;
                    self.action_counter = 0;
                    self.anim_num = 7;

                    self.npc_flags.set_rear_and_top_not_hurt(true);
                    self.damage = 10;
                    self.vel_x = self.direction.vector_x() * 0x5FF;

                    state.sound_manager.play_sfx(25);
                }
            }
            32 => {
                self.action_counter += 1;
                self.vel_y = 0;

                self.anim_num = if self.action_counter & 2 != 0 { 7 } else { 8 };

                if self.action_counter > 30 {
                    self.action_num = 18;
                    self.action_counter = 0;
                    self.damage = 5;

                    self.npc_flags.set_rear_and_top_not_hurt(false);
                    self.npc_flags.set_solid_soft(false);
                    self.npc_flags.set_shootable(true);
                }

                if self.flags.hit_left_wall() || self.flags.hit_right_wall() {
                    self.action_num = 15;
                    self.action_counter = 0;
                    self.damage = 5;

                    self.npc_flags.set_rear_and_top_not_hurt(false);
                    self.npc_flags.set_solid_soft(false);
                    self.npc_flags.set_shootable(true);
                }
            }
            40 => {
                self.anim_num = 3;
                self.action_counter += 1;
                let player = self.get_closest_player_ref(&players);
                self.face_player(player);

                if self.action_counter > 20 {
                    self.action_num = 41;
                    self.action_counter = 0;
                    self.anim_num = 4;
                    self.vel_y = -0x800;
                    self.vel_x = self.direction.vector_x() * 0x400;
                }
            }
            41 => {
                self.vel_y += 0x40;
                self.animate(1, 4, 5);

                let player = self.get_closest_player_ref(&players);
                if player.y > self.y && player.x > self.x - 0x1000 && player.x < self.x + 0x1000 {
                    self.action_num = 16;
                    self.vel_y = 0x5FF;
                    self.vel_x = 0;
                }

                if self.vel_y > 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 17;
                }
            }
            100 | 101 => {
                if self.action_num == 100 {
                    self.action_num = 101;
                    self.action_counter = 0;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_invulnerable(false);
                    self.damage = 0;
                    state.sound_manager.play_sfx(29);
                }

                self.action_counter += 2;
                if self.action_counter > 28 {
                    self.action_num = 102;
                    self.action_counter = 0;
                    self.anim_num = 0;

                    let player = self.get_closest_player_ref(&players);
                    self.target_x = player.x;
                    self.target_y = player.y - 0x4000;

                    self.target_x = self.target_x.clamp(0x8000, 0x48000);
                    if self.target_y < 0x8000 {
                        self.target_y = 0x8000;
                    }
                }
            }
            102 => {
                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.action_num = 103;
                    self.action_counter = 28;
                    self.anim_num = 4;
                    self.vel_y = 0;
                    self.x = self.target_x;
                    self.y = self.target_y;

                    let player = self.get_closest_player_ref(&players);
                    self.face_player(player);
                }
            }
            103 => {
                if self.action_counter > 0 {
                    self.action_counter -= 2;
                } else {
                    self.action_num = 16;
                    self.vel_x = 0;
                    self.vel_y = -0x200;

                    self.npc_flags.set_shootable(true);
                    self.npc_flags.set_invulnerable(true);
                    self.damage = 5;
                }
            }
            500 => {
                npc_list.kill_npcs_by_type(269, true, state);

                self.npc_flags.set_shootable(false);
                self.anim_num = 4;
                self.vel_y += 0x20;
                self.vel_x = 0;

                if self.flags.hit_bottom_wall() {
                    self.action_num = 501;
                    self.action_counter = 0;
                    self.target_x = self.x;

                    let player = self.get_closest_player_ref(&players);
                    self.face_player(player);
                }
            }
            501 => {
                self.anim_num = 9;

                self.action_counter += 1;
                if self.action_counter / 2 % 2 != 0 {
                    self.x = self.target_x;
                } else {
                    self.x = self.x.wrapping_add(0x200);
                }
            }
            510 | 511 => {
                if self.action_num == 510 {
                    self.action_num = 511;
                    self.action_counter = 0;
                    self.anim_num = 9;
                    self.target_x = self.x;
                    self.y += 0x2000;
                    self.npc_flags.set_ignore_solidity(true);
                }

                state.quake_counter = 2;
                state.quake_rumble_counter = 2;
                self.action_counter += 1;
                if self.action_counter % 6 == 3 {
                    state.sound_manager.play_sfx(25);
                }

                self.x = if self.action_counter & 2 != 0 { self.target_x } else { self.target_x + 512 };

                if self.action_counter > 352 {
                    self.action_num = 512;
                    self.anim_num = 0;
                }
            }
            520 => {
                self.damage = 0;
                state.npc_super_pos.1 = -0x4000;
            }
            _ => (),
        }

        if self.action_num > 10 && self.action_num <= 500 {
            if self.action_num == 102 {
                state.npc_super_pos = (self.target_x, self.target_y);
            } else {
                state.npc_super_pos = (self.x, self.y);
            }
        }

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.action_num < 512 {
            if self.action_num >= 510 {
                let mut npc = NPC::create(270, &state.npc_table);
                npc.cond.set_alive(true);

                for i in 0..4 {
                    npc.x = self.x + self.rng.range(-0x10..0x10) * 0x200;
                    npc.y = self.y - ((0x150 - self.action_counter as i32) / 8) * 0x200;
                    npc.vel_y = 2 * self.rng.range(-0x200..0);
                    npc.vel_x = if i >= 2 { 0 } else { self.rng.range(-0x200..0x200) };
                    npc.direction = Direction::Bottom;
                    let _ = npc_list.spawn(0xAA, npc.clone());
                }
            } else if self.action_num != 102 && self.action_num != 103 && self.rng.range(0..3) == 2 {
                let mut npc = NPC::create(270, &state.npc_table);
                npc.cond.set_alive(true);

                npc.x = self.x + self.rng.range(-0x10..0x10) * 0x200;
                npc.y = self.y + self.rng.range(-8..4) * 0x200;
                npc.vel_x = self.vel_x;
                npc.vel_y = 0;
                npc.direction = Direction::Bottom;
                let _ = npc_list.spawn(0x100, npc.clone());
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 10 };

        self.anim_rect = state.constants.npc.n267_muscle_doctor[self.anim_num as usize + dir_offset];

        if self.action_num == 511 {
            self.anim_rect.top += self.action_counter / 8;
            self.display_bounds.top = (44u32).saturating_sub(self.action_counter as u32 / 8) * 0x200;
            self.display_bounds.bottom = 0x800;
        } else if self.action_num == 101 || self.action_num == 103 {
            self.anim_rect.top += self.action_counter;
            self.anim_rect.bottom -= self.action_counter;
            self.display_bounds.top = (28u32).saturating_sub(self.action_counter as u32) * 0x200;
        } else {
            self.display_bounds.top = 0x3800;
        }

        Ok(())
    }

    pub(crate) fn tick_n269_red_bat_bouncing(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.vel_x2 = self.vel_x;
            self.vel_y2 = self.vel_y;
        }

        if self.action_num == 1 {
            if self.vel_x2 < 0 && self.flags.hit_left_wall() {
                self.direction = Direction::Right;
                self.vel_x2 = -self.vel_x2;
            } else if self.vel_x2 > 0 && self.flags.hit_right_wall() {
                self.direction = Direction::Left;
                self.vel_x2 = -self.vel_x2;
            } else if self.vel_y2 < 0 && self.flags.hit_top_wall() {
                self.vel_y2 = -self.vel_y2;
            } else if self.vel_y2 > 0 && self.flags.hit_bottom_wall() {
                self.vel_y2 = -self.vel_y2;
            }

            self.x += self.vel_x2;
            self.y += self.vel_y2;
            self.animate(2, 0, 2);
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n269_red_bat_bouncing[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n270_doctor_red_energy(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        if self.direction == Direction::Bottom || self.direction == Direction::Up {
            self.vel_y += self.direction.vector_y() * 0x40;

            self.action_counter += 1;
            self.clamp_fall_speed();

            self.x += self.vel_x;
            self.y += self.vel_y;
            if self.action_counter > 50 || self.flags.any_flag() {
                self.cond.set_alive(false)
            }
        } else if self.direction == Direction::Right {
            if self.action_num == 0 {
                self.action_num = 1;
                self.npc_flags.set_ignore_solidity(true);
                self.vel_x = 3 * self.rng.range(-0x200..0x200);
                self.vel_y = 3 * self.rng.range(-0x200..0x200);
                self.action_counter2 = self.rng.range(0x10..0x33) as u16;
                self.action_counter3 = self.rng.range(0x80..0x100) as u16;
            }

            if let Some(parent) = self.get_parent_ref_mut(npc_list) {
                if self.x < parent.x {
                    self.vel_x += 0x200 / self.action_counter2 as i32;
                }
                if self.x > parent.x {
                    self.vel_x -= 0x200 / self.action_counter2 as i32;
                }
                if self.y < parent.y {
                    self.vel_y += 0x200 / self.action_counter2 as i32;
                }
                if self.y > parent.y {
                    self.vel_y -= 0x200 / self.action_counter2 as i32;
                }
                if self.vel_x > 2 * self.action_counter3 as i32 {
                    self.vel_x = 2 * self.action_counter3 as i32;
                }
                if self.vel_x < -2 * self.action_counter3 as i32 {
                    self.vel_x = -2 * self.action_counter3 as i32;
                }
                if self.vel_y > 3 * self.action_counter3 as i32 {
                    self.vel_y = 3 * self.action_counter3 as i32;
                }
                if self.vel_y < -3 * self.action_counter3 as i32 {
                    self.vel_y = -3 * self.action_counter3 as i32;
                }
                self.x += self.vel_x;
                self.y += self.vel_y;
            }
        }

        self.anim_rect = state.constants.npc.n270_doctor_red_energy[self.rng.range(0..1) as usize];

        Ok(())
    }

    pub(crate) fn tick_n281_doctor_energy_form(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.action_counter = 0;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                }

                self.action_counter += 1;

                let mut npc = NPC::create(270, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;
                npc.y = self.y + 0x10000;
                npc.direction = Direction::Right;
                npc.parent_id = self.id;

                let _ = npc_list.spawn(0x100, npc);

                if self.action_counter > 150 {
                    self.action_num = 12;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 250 {
                    self.action_num = 22;
                    npc_list.remove_by_type(270, state);
                }
            }
            _ => (),
        }

        self.anim_rect = Rect::new(0, 0, 0, 0);

        Ok(())
    }
}
