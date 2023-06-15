use crate::common::{CDEG_RAD, Direction};
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::Stage;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n178_core_blade_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.flags.hit_anything() {
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
            self.cond.set_alive(false);
        }

        if self.flags.in_water() {
            self.x += self.vel_x / 2;
            self.y += self.vel_y / 2;
        } else {
            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        self.animate(1, 0, 2);

        self.action_counter3 += 1;
        if self.action_counter3 > 150 {
            self.vanish(state);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        self.anim_rect = state.constants.npc.n178_core_blade_projectile[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n179_core_wisp_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.flags.hit_anything() {
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
            self.cond.set_alive(false);
        }

        self.vel_x -= 0x20;
        self.vel_y = 0;

        if self.vel_x < -0x400 {
            self.vel_x = -0x400;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.animate(1, 0, 2);

        self.action_counter3 += 1;
        if self.action_counter3 > 300 {
            self.vanish(state);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        self.anim_rect = state.constants.npc.n179_core_wisp_projectile[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n218_core_giant_ball(&mut self, state: &mut SharedGameState) -> GameResult {
        self.x += self.vel_x;
        self.y += self.vel_y;

        self.action_counter += 1;
        if self.action_counter > 200 {
            self.cond.set_alive(false);
        }

        self.animate(2, 0, 1);
        self.anim_rect = state.constants.npc.n218_core_giant_ball[self.anim_num as usize];

        Ok(())
    }
}

impl BossNPC {
    pub(crate) fn tick_b04_core(
        &mut self,
        state: &mut SharedGameState,
        mut players: [&mut Player; 2],
        npc_list: &NPCList,
        stage: &mut Stage,
    ) {
        let mut flag = false;
        // i will refactor that one day
        #[allow(mutable_transmutes)]
            let flash_counter: &mut u16 = unsafe { std::mem::transmute(&self.parts[19].action_counter3) };

        match self.parts[0].action_num {
            0 => {
                self.parts[0].action_num = 10;
                self.parts[0].exp = 1;
                self.parts[0].cond.set_alive(true);
                self.parts[0].npc_flags.0 = 0;
                self.parts[0].npc_flags.set_show_damage(true);
                self.parts[0].npc_flags.set_event_when_killed(true);
                self.parts[0].npc_flags.set_ignore_solidity(true);
                self.parts[0].npc_flags.set_invulnerable(true);
                self.parts[0].life = 650;
                self.hurt_sound[0] = 114;
                self.parts[0].x = 0x9a000;
                self.parts[0].y = 0x1c000;
                self.parts[0].vel_x = 0;
                self.parts[0].vel_y = 0;
                self.parts[0].event_num = 1000;

                self.parts[4].cond.set_alive(true);
                self.parts[4].action_num = 10;

                self.parts[5].cond.set_alive(true);
                self.parts[5].action_num = 10;

                self.parts[8].cond.set_alive(true);
                self.parts[8].npc_flags.0 = 0;
                self.parts[8].npc_flags.set_ignore_solidity(true);
                self.parts[8].npc_flags.set_invulnerable(true);
                self.parts[8].display_bounds.left = 0;
                self.parts[8].display_bounds.top = 0;
                self.parts[8].hit_bounds.right = 0x5000;
                self.parts[8].hit_bounds.top = 0x2000;
                self.parts[8].hit_bounds.bottom = 0x2000;
                self.parts[8].action_counter2 = 0;

                self.parts[9] = self.parts[8].clone();
                self.parts[9].hit_bounds.right = 0x4800;
                self.parts[9].hit_bounds.top = 0x3000;
                self.parts[9].hit_bounds.bottom = 0x3000;
                self.parts[9].action_counter2 = 1;

                self.parts[10] = self.parts[8].clone();
                self.parts[10].hit_bounds.right = 0x5800;
                self.parts[10].hit_bounds.top = 0x1000;
                self.parts[10].hit_bounds.bottom = 0x1000;
                self.parts[10].action_counter2 = 2;

                self.parts[11] = self.parts[8].clone();
                self.parts[11].cond.set_damage_boss(true);
                self.parts[11].hit_bounds.right = 0x2800;
                self.parts[11].hit_bounds.top = 0x2800;
                self.parts[11].hit_bounds.bottom = 0x2800;
                self.parts[11].action_counter2 = 3;

                self.parts[1].cond.set_alive(true);
                self.parts[1].action_num = 10;
                self.parts[1].npc_flags.set_shootable(true);
                self.parts[1].npc_flags.set_ignore_solidity(true);
                self.parts[1].npc_flags.set_invulnerable(true);
                self.parts[1].life = 1000;
                self.hurt_sound[1] = 54;
                self.parts[1].hit_bounds.right = 0x3000;
                self.parts[1].hit_bounds.top = 0x2000;
                self.parts[1].hit_bounds.bottom = 0x2000;
                self.parts[1].display_bounds.top = 0x2800;
                self.parts[1].display_bounds.left = 0x4000;
                self.parts[1].x = self.parts[0].x - 0x1000;
                self.parts[1].y = self.parts[0].y - 0x8000;

                self.parts[2] = self.parts[1].clone();
                self.parts[2].x = self.parts[0].x + 0x2000;
                self.parts[2].y = self.parts[0].y;

                self.parts[3] = self.parts[1].clone();
                self.parts[3].x = self.parts[0].x - 0x1000;
                self.parts[3].y = self.parts[0].y + 0x8000;

                self.parts[6] = self.parts[1].clone();
                self.parts[6].x = self.parts[0].x - 0x6000;
                self.parts[6].y = self.parts[0].y - 0x4000;

                self.parts[7] = self.parts[1].clone();
                self.parts[7].x = self.parts[0].x - 0x6000;
                self.parts[7].y = self.parts[0].y + 0x4000;

                for part in &mut self.parts {
                    part.prev_x = part.x;
                    part.prev_y = part.y;
                }
            }
            200 | 201 => {
                if self.parts[0].action_num == 200 {
                    self.parts[0].action_num = 201;
                    self.parts[0].action_counter = 0;
                    self.parts[11].npc_flags.set_shootable(false);
                    state.npc_super_pos.1 = 0;

                    state.sound_manager.stop_sfx(40);
                    state.sound_manager.stop_sfx(41);
                    state.sound_manager.stop_sfx(58);
                }

                let idx = self.parts[0].get_closest_player_idx_mut(&players);
                self.parts[0].target_x = players[idx].x;
                self.parts[0].target_y = players[idx].y;

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 400 {
                    self.parts[0].action_counter2 += 1;

                    state.sound_manager.play_sfx(115);

                    if self.parts[0].action_counter2 < 4 {
                        self.parts[0].action_num = 210;
                    } else {
                        self.parts[0].action_counter2 = 0;
                        self.parts[0].action_num = 220;
                    }

                    self.parts[4].anim_num = 0;
                    self.parts[5].anim_num = 0;

                    flag = true;
                }
            }
            210 | 211 => {
                if self.parts[0].action_num == 210 {
                    self.parts[0].action_num = 211;
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_counter3 = self.parts[0].life;
                    self.parts[11].npc_flags.set_shootable(true);
                }

                let idx = self.parts[0].get_closest_player_idx_mut(&players);
                self.parts[0].target_x = players[idx].x;
                self.parts[0].target_y = players[idx].y;

                if self.parts[0].shock > 0 {
                    *flash_counter += 1;
                    if (*flash_counter & 2) != 0 {
                        self.parts[4].anim_num = 0;
                        self.parts[5].anim_num = 0;
                    } else {
                        self.parts[4].anim_num = 1;
                        self.parts[5].anim_num = 1;
                    }
                } else {
                    self.parts[4].anim_num = 0;
                    self.parts[5].anim_num = 0;
                }

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter % 100 == 1 {
                    state.npc_curly_counter = self.parts[0].rng.range(80..100) as u16;
                    state.npc_curly_target = (self.parts[11].x, self.parts[11].y)
                }

                if self.parts[0].action_counter < 200 && self.parts[0].action_counter % 20 == 1 {
                    let mut npc = NPC::create(179, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x + self.parts[0].rng.range(-48..-16) * 0x200;
                    npc.y = self.parts[0].y + self.parts[0].rng.range(-64..64) * 0x200;
                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.parts[0].action_counter > 400
                    || (self.parts[0].life as i32) < self.parts[0].action_counter3 as i32 - 200
                {
                    self.parts[0].action_num = 200;
                    self.parts[4].anim_num = 2;
                    self.parts[5].anim_num = 0;

                    flag = true;
                }
            }
            220 | 221 => {
                if self.parts[0].action_num == 220 {
                    self.parts[0].action_num = 221;
                    self.parts[0].action_counter = 0;
                    self.parts[11].npc_flags.set_shootable(true);

                    state.sound_manager.loop_sfx_freq(40, 1000.0 / 2205.0);
                    state.sound_manager.loop_sfx_freq(41, 1100.0 / 2205.0);
                    state.quake_counter = 100;
                    state.quake_rumble_counter = 100;
                    state.npc_super_pos.1 = 1;
                }

                self.parts[0].action_counter += 1;

                let idx = self.parts[0].get_closest_player_idx_mut(&players);
                let mut npc = NPC::create(199, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = players[idx].x + self.parts[0].rng.range(-50..150) * 0x400;
                npc.y = players[idx].y + self.parts[0].rng.range(-160..160) * 0x200;
                let _ = npc_list.spawn(0x100, npc);

                for player in players.iter_mut() {
                    player.vel_x -= 0x20;
                    player.cond.set_increase_acceleration(true);
                }

                if self.parts[0].shock > 0 {
                    *flash_counter += 1;
                    if (*flash_counter & 2) != 0 {
                        self.parts[4].anim_num = 0;
                        self.parts[5].anim_num = 0;
                    } else {
                        self.parts[4].anim_num = 1;
                        self.parts[5].anim_num = 1;
                    }
                } else {
                    self.parts[4].anim_num = 0;
                    self.parts[5].anim_num = 0;
                }

                if [300, 350, 400].contains(&self.parts[0].action_counter) {
                    state.sound_manager.play_sfx(101);
                    let mut npc = NPC::create(218, &state.npc_table);
                    let angle = f64::atan2(
                        (self.parts[0].y - players[idx].y) as f64,
                        (self.parts[0].x - players[idx].x) as f64,
                    );
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x - 0x5000;
                    npc.y = self.parts[0].y;
                    npc.vel_x = (angle.cos() * -1536.0) as i32;
                    npc.vel_y = (angle.sin() * -1536.0) as i32;

                    let _ = npc_list.spawn(0x100, npc);
                } else if self.parts[0].action_counter > 400 {
                    self.parts[0].action_num = 200;
                    self.parts[4].anim_num = 2;
                    self.parts[5].anim_num = 0;

                    flag = true;
                }
            }
            500 | 501 => {
                if self.parts[0].action_num == 500 {
                    self.parts[0].action_num = 501;
                    self.parts[0].action_counter = 0;
                    self.parts[0].vel_x = 0;
                    self.parts[0].vel_y = 0;
                    self.parts[4].anim_num = 2;
                    self.parts[5].anim_num = 0;
                    self.parts[1].action_num = 200;
                    self.parts[2].action_num = 200;
                    self.parts[3].action_num = 200;
                    self.parts[6].action_num = 200;
                    self.parts[7].action_num = 200;

                    state.quake_counter = 20;
                    state.quake_rumble_counter = 20;

                    state.sound_manager.stop_sfx(40);
                    state.sound_manager.stop_sfx(41);
                    state.sound_manager.stop_sfx(58);

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    for _ in 0..32 {
                        npc.x = self.parts[0].x + self.parts[0].rng.range(-0x80..0x80) * 0x200;
                        npc.y = self.parts[0].y + self.parts[0].rng.range(-0x40..0x40) * 0x200;

                        npc.vel_x = self.parts[0].rng.range(-0x80..0x80) * 0x200;
                        npc.vel_y = self.parts[0].rng.range(-0x80..0x80) * 0x200;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }

                    for i in 0..12 {
                        self.parts[i].npc_flags.set_invulnerable(false);
                        self.parts[i].npc_flags.set_shootable(false);
                    }
                }

                self.parts[0].action_counter += 1;
                if (self.parts[0].action_counter & 0x0f) != 0 {
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x + self.parts[0].rng.range(-0x40..0x40) * 0x200;
                    npc.y = self.parts[0].y + self.parts[0].rng.range(-0x20..0x20) * 0x200;

                    npc.vel_x = self.parts[0].rng.range(-0x80..0x80) * 0x200;
                    npc.vel_y = self.parts[0].rng.range(-0x80..0x80) * 0x200;

                    let _ = npc_list.spawn(0x100, npc);
                }

                self.parts[0].x += if self.parts[0].action_counter & 0x02 == 0 { 0x200 } else { -0x200 };
                self.parts[0].x += if self.parts[0].x < 0x7E000 { 0x80 } else { -0x80 };
                self.parts[0].y += if self.parts[0].y < 0x16000 { 0x80 } else { -0x80 };
            }
            600 | 601 => {
                if self.parts[0].action_num == 600 {
                    self.parts[0].action_num = 601;
                    self.parts[4].action_num = 50;
                    self.parts[5].action_num = 50;
                    self.parts[8].npc_flags.set_invulnerable(false);
                    self.parts[9].npc_flags.set_invulnerable(false);
                    self.parts[10].npc_flags.set_invulnerable(false);
                    self.parts[11].npc_flags.set_invulnerable(false);
                }
                self.parts[0].action_counter += 1;

                self.parts[0].x += if self.parts[0].action_counter & 0x02 == 0 { 0x800 } else { -0x800 };
            }
            _ => {}
        }

        if flag {
            state.quake_counter = 20;
            state.quake_rumble_counter = 20;
            state.sound_manager.play_sfx(26);

            self.parts[1].action_num = 100;
            self.parts[2].action_num = 100;
            self.parts[3].action_num = 100;
            self.parts[6].action_num = 100;
            self.parts[7].action_num = 100;

            let mut npc = NPC::create(4, &state.npc_table);
            npc.cond.set_alive(true);
            npc.y = self.parts[4].y;
            for _ in 0..8 {
                npc.x = self.parts[4].x + self.parts[0].rng.range(-32..16) * 0x200;
                npc.vel_x = self.parts[0].rng.range(-0x200..0x200);
                npc.vel_y = self.parts[0].rng.range(-0x100..0x100);
                let _ = npc_list.spawn(0x100, npc.clone());
            }
        }

        if self.parts[0].action_num >= 200 && self.parts[0].action_num < 300 {
            if self.parts[0].action_counter == 80 {
                self.parts[1].action_num = 120;
            } else if self.parts[0].action_counter == 110 {
                self.parts[2].action_num = 120;
            } else if self.parts[0].action_counter == 140 {
                self.parts[3].action_num = 120;
            } else if self.parts[0].action_counter == 170 {
                self.parts[6].action_num = 120;
            } else if self.parts[0].action_counter == 200 {
                self.parts[7].action_num = 120;
            }

            if self.parts[0].x < self.parts[0].target_x + 0x14000 {
                self.parts[0].vel_x += 4;
            }

            if self.parts[0].x > self.parts[0].target_x + 0x14000 {
                self.parts[0].vel_x -= 4;
            }

            if self.parts[0].y < self.parts[0].target_y {
                self.parts[0].vel_y += 4;
            }

            if self.parts[0].y > self.parts[0].target_y {
                self.parts[0].vel_y -= 4;
            }
        }

        self.parts[0].vel_x = self.parts[0].vel_x.clamp(-0x80, 0x80);
        self.parts[0].vel_y = self.parts[0].vel_y.clamp(-0x80, 0x80);
        self.parts[0].x += self.parts[0].vel_x;
        self.parts[0].y += self.parts[0].vel_y;

        self.tick_b04_core_face(4, state);
        self.tick_b04_core_tail(5, state);
        self.tick_b04_core_small_head(1, state, &players, npc_list, stage);
        self.tick_b04_core_small_head(2, state, &players, npc_list, stage);
        self.tick_b04_core_small_head(3, state, &players, npc_list, stage);
        self.tick_b04_core_small_head(6, state, &players, npc_list, stage);
        self.tick_b04_core_small_head(7, state, &players, npc_list, stage);
        self.tick_b04_core_hitbox(8);
        self.tick_b04_core_hitbox(9);
        self.tick_b04_core_hitbox(10);
        self.tick_b04_core_hitbox(11);
    }

    fn tick_b04_core_face(&mut self, i: usize, state: &mut SharedGameState) {
        let (head, tail) = self.parts.split_at_mut(i);
        let base = &mut head[0];
        let part = &mut tail[0];

        match part.action_num {
            10 | 11 => {
                if part.action_num == 10 {
                    part.action_num = 11;
                    part.anim_num = 2;
                    part.npc_flags.set_ignore_solidity(true);
                    part.display_bounds.left = 0x4800;
                    part.display_bounds.top = 0x7000;
                }
                part.x = base.x - 0x4800;
                part.y = base.y;
            }
            50 | 51 => {
                if part.action_num == 50 {
                    part.action_num = 51;
                    part.action_counter = 112;
                }
                part.action_counter -= 1;

                if part.action_counter == 0 {
                    part.action_num = 100;
                    part.anim_num = 3;
                }
                part.x = base.x - 0x4800;
                part.y = base.y;
            }
            100 => {
                part.anim_num = 3;
            }
            _ => {}
        }

        part.anim_rect = state.constants.npc.b04_core[part.anim_num as usize];

        if part.action_num == 51 {
            part.anim_rect.bottom = part.action_counter + part.anim_rect.top;
        }
    }

    fn tick_b04_core_tail(&mut self, i: usize, state: &mut SharedGameState) {
        let (head, tail) = self.parts.split_at_mut(i);
        let base = &mut head[0];
        let part = &mut tail[0];

        match part.action_num {
            10 | 11 => {
                if part.action_num == 10 {
                    part.action_num = 11;
                    part.anim_num = 0;
                    part.npc_flags.set_ignore_solidity(true);
                    part.display_bounds.left = 0x5800;
                    part.display_bounds.top = 0x7000;
                }
                part.x = base.x + 0x5800;
                part.y = base.y;
            }
            50 | 51 => {
                if part.action_num == 50 {
                    part.action_num = 51;
                    part.action_counter = 112;
                }
                part.action_counter -= 1;

                if part.action_counter == 0 {
                    part.action_num = 100;
                    part.anim_num = 2;
                }
                part.x = base.x + 0x5800;
                part.y = base.y;
            }
            100 => {
                part.anim_num = 2;
            }
            _ => {}
        }

        part.anim_rect = state.constants.npc.b04_core[4 + part.anim_num as usize];

        if part.action_num == 51 {
            part.anim_rect.bottom = part.action_counter + part.anim_rect.top;
        }
    }

    fn tick_b04_core_small_head(
        &mut self,
        i: usize,
        state: &mut SharedGameState,
        players: &[&mut Player; 2],
        npc_list: &NPCList,
        stage: &Stage,
    ) {
        let (head, tail) = self.parts.split_at_mut(i);
        let base = &mut head[0];
        let part = &mut tail[0];

        part.life = 1000;
        match part.action_num {
            10 => {
                part.anim_num = 2;
                part.npc_flags.set_shootable(false);
            }
            100 | 101 => {
                if part.action_num == 100 {
                    part.action_num = 101;
                    part.anim_num = 2;
                    part.action_counter = 0;
                    part.target_x = base.x + (base.rng.range(-128..32) * 0x200);
                    part.target_y = base.y + (base.rng.range(-64..64) * 0x200);
                    part.npc_flags.set_shootable(true);
                }
                part.x += (part.target_x - part.x) / 16;
                part.y += (part.target_y - part.y) / 16;

                part.action_counter += 1;
                if part.action_counter > 50 {
                    part.anim_num = 0;
                }
            }
            120 | 121 => {
                if part.action_num == 120 {
                    part.action_num = 121;
                    part.action_counter = 0;
                }

                part.action_counter += 1;
                if ((part.action_counter / 2) & 1) != 0 {
                    part.anim_num = 0;
                } else {
                    part.anim_num = 1;
                }

                if part.action_counter > 20 {
                    part.action_num = 130;
                }
            }
            130 | 131 => {
                if part.action_num == 130 {
                    part.action_num = 131;
                    part.anim_num = 2;
                    part.action_counter = 0;
                    part.target_x = part.x + (part.rng.range(24..48) * 0x200);
                    part.target_y = part.y + (part.rng.range(-4..4) * 0x200);
                }

                part.x += (part.target_x - part.x) / 16;
                part.y += (part.target_y - part.y) / 16;

                part.action_counter += 1;
                if part.action_counter > 50 {
                    part.action_num = 140;
                    part.anim_num = 0;
                }

                if part.action_counter == 1 || part.action_counter == 3 {
                    let player_idx = part.get_closest_player_idx_mut(players);
                    let px = part.x - players[player_idx].x;
                    let py = part.y - players[player_idx].y;

                    let deg = f64::atan2(py as f64, px as f64) + part.rng.range(-2..2) as f64 * CDEG_RAD;

                    let mut npc = NPC::create(178, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = part.x;
                    npc.y = part.y;
                    npc.vel_x = (deg.cos() * -1024.0) as i32;
                    npc.vel_y = (deg.sin() * -1024.0) as i32;

                    let _ = npc_list.spawn(0x100, npc);

                    state.sound_manager.play_sfx(39);
                }
            }
            140 => {
                part.x += (part.target_x - part.x) / 16;
                part.y += (part.target_y - part.y) / 16;
            }
            200 | 201 => {
                if part.action_num == 200 {
                    part.action_num = 201;
                    part.anim_num = 2;
                    part.vel_x = 0;
                    part.vel_y = 0;
                }
                part.vel_x += 32;
                part.x += part.vel_x;

                if part.x > (stage.map.width as i32 * 0x2000) + 0x4000 {
                    part.cond.set_alive(false);
                }
            }
            _ => (),
        }
        if part.shock > 0 {
            part.target_x += 1024;
        }

        part.anim_rect = state.constants.npc.b04_core[7 + part.anim_num as usize];
    }

    fn tick_b04_core_hitbox(&mut self, i: usize) {
        let (head, tail) = self.parts.split_at_mut(i);
        let base = &mut head[0];
        let part = &mut tail[0];

        match part.action_counter2 {
            0 => {
                part.x = base.x;
                part.y = base.y - 0x4000;
            }
            1 => {
                part.x = base.x + 0x3800;
                part.y = base.y;
            }
            2 => {
                part.x = base.x + 0x800;
                part.y = base.y + 0x4000;
            }
            3 => {
                part.x = base.x - 0x3800;
                part.y = base.y + 0x800;
            }
            _ => (),
        }
    }
}
