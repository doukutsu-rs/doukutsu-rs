use crate::npc::boss::BossNPC;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl BossNPC {
    pub(crate) fn tick_b04_core(&mut self, state: &mut SharedGameState, players: [&mut Player; 2], npc_list: &NPCList) {
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
                self.parts[2].x = self.parts[0].y;

                self.parts[3] = self.parts[1].clone();
                self.parts[3].x = self.parts[0].x - 0x1000;
                self.parts[3].x = self.parts[0].y + 0x8000;

                self.parts[6] = self.parts[1].clone();
                self.parts[6].x = self.parts[0].x - 0x6000;
                self.parts[6].x = self.parts[0].y - 0x4000;

                self.parts[7] = self.parts[1].clone();
                self.parts[7].x = self.parts[0].x - 0x6000;
                self.parts[7].x = self.parts[0].y + 0x4000;
            }
            200 => {
                self.parts[0].action_num = 201;
                self.parts[0].action_counter = 0;
                self.parts[11].npc_flags.set_shootable(false);
                state.npc_super_pos.1 = 0;

                state.sound_manager.stop_sfx(40);
                state.sound_manager.stop_sfx(41);
                state.sound_manager.stop_sfx(58);
            }
            201 => {
                self.parts[0].target_x = self.parts[0].x;
                self.parts[0].target_y = self.parts[0].y;

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 400 {
                    self.parts[0].action_counter2 += 2;

                    state.sound_manager.play_sfx(115);

                    if self.parts[0].action_counter2 < 4 {
                        self.parts[0].action_num = 210;
                    } else {
                        self.parts[0].action_counter2 = 0;
                        self.parts[0].action_num = 220;
                    }
                }

                self.parts[4].anim_num = 0;
                self.parts[5].anim_num = 0;

                flag = true;
            }
            210 | 211 => {
                if self.parts[0].action_num == 210 {
                    self.parts[0].action_num = 211;
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_counter2 = self.parts[0].life;
                    self.parts[11].npc_flags.set_shootable(true);
                }

                let player = self.parts[0].get_closest_player_mut(players);
                self.parts[0].target_x = player.x;
                self.parts[0].target_y = player.y;

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
                    || (self.parts[0].life as i32) < self.parts[0].action_counter2 as i32 - 200
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

                    // todo <SSS1000 equivalent!!!!
                    state.quake_counter = 100;
                    state.npc_super_pos.1 = 1;
                }

                self.parts[0].action_counter += 1;

                let idx = self.parts[0].get_closest_player_idx_mut(&players);
                let mut npc = NPC::create(199, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = players[idx].x + self.parts[0].rng.range(-50..150) * 0x400;
                npc.y = players[idx].y + self.parts[0].rng.range(-160..160) * 0x200;
                let _ = npc_list.spawn(0x100, npc);

                for player in players {
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
                    let angle = 0.0f64;
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
            _ => {}
        }

        if flag {
            state.quake_counter = 20;
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
                npc_list.spawn(0x100, npc.clone());
            }
        }

        if self.parts[0].action_num >= 200 && self.parts[0].action_num < 300 {
            if self.parts[0].action_counter == 140 {
                self.parts[3].action_num = 120;
            } else if self.parts[0].action_counter == 170 {
                self.parts[6].action_num = 120;
            } else if self.parts[0].action_counter == 200 {
                self.parts[7].action_num = 120;
            } else if self.parts[0].action_counter == 80 {
                self.parts[1].action_num = 120;
            } else if self.parts[0].action_counter == 110 {
                self.parts[2].action_num = 120;
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

        self.parts[0].vel_x = self.parts[0].vel_x.clamp(-0x100, 0x100);
        self.parts[0].vel_y = self.parts[0].vel_y.clamp(-0x100, 0x100);
        self.parts[0].x += self.parts[0].vel_x;
        self.parts[0].y += self.parts[0].vel_y;
    }
}
