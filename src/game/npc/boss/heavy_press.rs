use crate::common::{Direction, Rect};
use crate::framework::error::GameResult;
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::{NPCAccessToken, NPCList};
use crate::game::npc::NPC;
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::Stage;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n325_heavy_press_lightning(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    state.sound_manager.play_sfx(29);
                }

                self.animate(0, 0, 2);

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 10;
                    self.anim_counter = 0;
                    self.anim_num = 3;
                    self.damage = 10;
                    self.display_bounds.left = 0x1000;
                    self.display_bounds.top = 0x1800;
                    state.sound_manager.play_sfx(101);
                    npc_list.create_death_smoke(self.x, self.y + 0xA800, 0, 3, state, &self.rng);
                }
            }
            10 => {
                self.animate(2, 3, 7);

                if self.anim_num > 6 {
                    self.cond.set_alive(false);
                    return Ok(());
                };
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n325_heavy_press_lightning[self.anim_num as usize];

        Ok(())
    }
}

impl BossNPC {
    pub(crate) fn tick_b08_heavy_press(&mut self, state: &mut SharedGameState, npc_list: &NPCList, stage: &mut Stage, token: &mut NPCAccessToken) {
        match self.parts[0].action_num {
            0 => {
                self.parts[0].action_num = 10;
                self.parts[0].cond.set_alive(true);
                self.parts[0].exp = 1;
                self.parts[0].direction = Direction::Right;
                self.parts[0].x = 0;
                self.parts[0].y = 0;
                self.parts[0].display_bounds = Rect { left: 0x5000, top: 0x7800, right: 0x5000, bottom: 0x7800 };
                self.parts[0].hit_bounds = Rect { left: 0x6200, top: 0x7800, right: 0x5000, bottom: 0x6000 };
                self.hurt_sound[0] = 54;
                self.parts[0].npc_flags.set_ignore_solidity(true);
                self.parts[0].npc_flags.set_solid_hard(true);
                self.parts[0].npc_flags.set_event_when_killed(true);
                self.parts[0].npc_flags.set_show_damage(true);
                self.parts[0].size = 3;
                self.parts[0].damage = 10;
                self.parts[0].event_num = 1000;
                self.parts[0].life = 700;
            }
            5 => {
                self.parts[0].action_num = 6;
                self.parts[0].x = 0;
                self.parts[0].y = 0;
                self.parts[1].cond.set_alive(false);
                self.parts[2].cond.set_alive(false);
            }
            10 => {
                self.parts[0].action_num = 11;
                self.parts[0].x = 0x14000;
                self.parts[0].y = 0x9400;
            }
            20 | 21 => {
                if self.parts[0].action_num == 20 {
                    self.parts[0].action_num = 21;
                    self.parts[0].damage = 0;
                    self.parts[0].x = 0x14000 + (state.constants.game.tile_offset_x * 0x2000);
                    self.parts[0].y = 0x33A00;
                    self.parts[0].npc_flags.set_solid_hard(false);
                    self.parts[1].cond.set_alive(false);
                    self.parts[2].cond.set_alive(false);
                }
                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter % 16 == 0 {
                    npc_list.create_death_smoke(
                        self.parts[0].x + self.parts[0].rng.range(-40..40) * 0x200,
                        self.parts[0].y + self.parts[0].rng.range(-60..60) * 0x200,
                        1,
                        1,
                        state,
                        &self.parts[0].rng,
                    );
                }
            }
            30 | 31 => {
                if self.parts[0].action_num == 30 {
                    self.parts[0].action_num = 31;
                    self.parts[0].anim_num = 2;
                    self.parts[0].x = 0x14000 + (state.constants.game.tile_offset_x * 0x2000);
                    self.parts[0].y = 0x8000;
                }
                self.parts[0].y += 0x800;

                if self.parts[0].y >= 0x33A00 {
                    self.parts[0].y = 0x33A00;
                    self.parts[0].anim_num = 0;
                    self.parts[0].action_num = 20;
                    state.sound_manager.play_sfx(44);

                    for _ in 0..5 {
                        let mut npc = NPC::create(4, &state.npc_table);
                        npc.x = self.parts[0].x + self.parts[0].rng.range(-40..40) * 0x200;
                        npc.y = self.parts[0].y + 0x7800;
                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }
            }
            100 | 101 => {
                if self.parts[0].action_num == 100 {
                    self.parts[0].action_num = 101;
                    self.parts[0].action_counter3 = 9;
                    self.parts[0].action_counter = 0; // This should be -100

                    self.parts[1].cond.set_alive(true);
                    self.parts[1].npc_flags.set_invulnerable(true);
                    self.parts[1].npc_flags.set_ignore_solidity(true);
                    self.parts[1].hit_bounds = Rect { left: 0x1C00, top: 0x1000, right: 0x1C00, bottom: 0x1000 };

                    self.parts[2] = self.parts[1].clone();

                    self.parts[3].cond.set_alive(true);
                    self.parts[3].cond.set_damage_boss(true);
                    self.parts[3].npc_flags.set_shootable(true);
                    self.parts[3].hit_bounds = Rect { left: 0xC00, top: 0x1000, right: 0xC00, bottom: 0x1000 };

                    let mut npc = NPC::create(325, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x;
                    npc.y = self.parts[0].y + 0x7800;

                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.parts[0].action_counter3 > 1 && self.parts[0].life < self.parts[0].action_counter3 * 70 {
                    self.parts[0].action_counter3 -= 1;

                    // This relies heavily on the map not being changed
                    // Need to calculate offset from the default starting location
                    for i in 0..5 {
                        let extra_smoke = if stage.change_tile(i + 8, self.parts[0].action_counter3 as usize, 0) { 3 } else { 0 };
                        npc_list.create_death_smoke(
                            (i as i32 + 8) * 0x2000,
                            self.parts[0].action_counter3 as i32 * 0x2000,
                            0,
                            4 + extra_smoke,
                            state,
                            &self.parts[0].rng,
                        );
                        state.sound_manager.play_sfx(12);
                    }
                }

                self.parts[0].action_counter += 1;

                // All of these checks are +100 to account for no negative values
                if self.parts[0].action_counter == 181 || self.parts[0].action_counter == 341 {
                    let mut npc = NPC::create(323, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = 0x6000;
                    npc.y = 0x1E000;
                    npc.direction = Direction::Up;
                    let _ = npc_list.spawn(0x100, npc);
                } else if self.parts[0].action_counter == 101 || self.parts[0].action_counter == 261 {
                    let mut npc = NPC::create(323, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = 0x22000;
                    npc.y = 0x1E000;
                    npc.direction = Direction::Up;
                    let _ = npc_list.spawn(0x100, npc);
                } else if self.parts[0].action_counter >= 400 {
                    self.parts[0].action_counter = 100; // Should be 0

                    let mut npc = NPC::create(325, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x;
                    npc.y = self.parts[0].y + 0x7800;
                    let _ = npc_list.spawn(0x100, npc);
                }
            }
            500 | 501 => {
                if self.parts[0].action_num == 500 {
                    self.parts[3].npc_flags.set_shootable(false);

                    self.parts[0].action_num = 501;
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_counter2 = 0;

                    npc_list.kill_npcs_by_type(325, true, state, token);
                    npc_list.kill_npcs_by_type(330, true, state, token);
                }

                self.parts[0].action_counter += 1;

                if self.parts[0].action_counter % 16 == 0 {
                    state.sound_manager.play_sfx(12);
                    npc_list.create_death_smoke(
                        self.parts[0].x + self.parts[0].rng.range(-40..40) * 0x200,
                        self.parts[0].y + self.parts[0].rng.range(-60..60) * 0x200,
                        1,
                        1,
                        state,
                        &self.parts[0].rng,
                    );
                }

                if self.parts[0].action_counter == 95 {
                    self.parts[0].anim_num = 1;
                } else if self.parts[0].action_counter == 98 {
                    self.parts[0].anim_num = 2;
                } else if self.parts[0].action_counter > 100 {
                    self.parts[0].action_num = 510;
                }
            }
            510 => {
                self.parts[0].vel_y += 64;
                self.parts[0].damage = 127;
                self.parts[0].y += self.parts[0].vel_y;

                if self.parts[0].action_counter2 == 0 && self.parts[0].y > 0x14000 {
                    self.parts[0].action_counter2 = 1;
                    self.parts[0].vel_y = -0x200;
                    self.parts[0].damage = 0;

                    // This relies heavily on the map not being changed
                    // Need to calculate offset from the default starting location
                    for i in 0..7 {
                        stage.change_tile(i + 7, 14, 0);
                        // This should be called with an amount of 0, but change_tile also needs to make smoke
                        npc_list.create_death_smoke((i as i32 + 7) * 0x2000, 0x1C000, 0, 3, state, &self.parts[0].rng);
                        state.sound_manager.play_sfx(12);
                    }
                }

                if self.parts[0].y > 0x3C000 {
                    self.parts[0].action_num = 520;
                }
            }
            _ => (),
        }

        self.parts[1].x = self.parts[0].x - 0x3000;
        self.parts[1].y = self.parts[0].y + 0x6800;

        self.parts[2].x = self.parts[0].x + 0x3000;
        self.parts[2].y = self.parts[0].y + 0x6800;

        self.parts[3].x = self.parts[0].x;
        self.parts[3].y = self.parts[0].y + 0x5000;

        let mut anim_offset = 0;
        if self.parts[0].shock != 0 {
            self.parts[4].action_counter += 1;
            if self.parts[4].action_counter & 0x02 == 0 {
                anim_offset = 3;
            }
        }

        self.parts[0].anim_rect = state.constants.npc.b08_heavy_press[self.parts[0].anim_num as usize + anim_offset];
    }
}
