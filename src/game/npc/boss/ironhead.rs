use crate::common::{Direction, Rect};
use crate::framework::error::GameResult;
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::{NPCAccessToken, NPCList};
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n196_ironhead_wall(&mut self, state: &mut SharedGameState) -> GameResult {
        self.x -= 0xC00;
        if self.x <= if !state.constants.is_switch { 0x26000 } else { 0x1E000 } {
            self.x += if !state.constants.is_switch { 0x2C000 } else { 0x3B400 };
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 1 };

        self.anim_rect = state.constants.npc.n196_ironhead_wall[dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n197_porcupine_fish(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 10 => {
                if self.action_num == 0 {
                    self.action_num = 10;
                    self.anim_counter = 0;
                    self.vel_y = self.rng.range(-0x200..0x200);
                    self.vel_x = 0x800;
                }

                self.animate(2, 0, 1);

                if self.vel_x < 0 {
                    self.damage = 3;
                    self.action_num = 20;
                }
            }
            20 => {
                self.damage = 3;
                self.animate(0, 2, 3);

                if self.x <= 0x5fff {
                    // npc->destroy_voice = 0; // todo
                    self.cond.set_explode_die(true);
                }
            }
            _ => (),
        }

        if self.flags.hit_top_wall() {
            self.vel_y = 0x200;
        }
        if self.flags.hit_bottom_wall() {
            self.vel_y = -0x200;
        }

        self.vel_x -= 0xC;
        self.x += self.vel_x;
        self.y += self.vel_y;
        self.anim_rect = state.constants.npc.n197_porcupine_fish[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n198_ironhead_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_counter += 1;
            if self.action_counter > 20 {
                self.action_num = 1;
                self.vel_x = 0;
                self.vel_y = 0;
                self.action_counter3 = 0;
            }
        } else if self.action_num == 1 {
            self.vel_x += 0x20;
        }

        self.animate(0, 0, 2);
        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n198_ironhead_projectile[self.anim_num as usize];

        self.action_counter3 += 1;
        if self.action_counter3 > 100 {
            self.cond.set_alive(false);
        }

        if self.action_counter3 % 4 == 1 {
            state.sound_manager.play_sfx(46);
        }

        Ok(())
    }

    pub(crate) fn tick_n335_ikachan(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = self.rng.range(3..20) as u16;
                }

                self.action_counter -= 1;
                if self.action_counter <= 0 {
                    self.action_num = 2;
                    self.action_counter = self.rng.range(10..50) as u16;
                    self.anim_num = 1;
                    self.vel_x = 0x600;
                }
            }
            2 => {
                self.action_counter -= 1;
                if self.action_counter <= 0 {
                    self.action_num = 3;
                    self.action_counter = self.rng.range(40..50) as u16;
                    self.anim_num = 2;
                    self.vel_y = self.rng.range(-0x100..0x100);
                }
            }
            3 => {
                self.action_counter -= 1;
                if self.action_counter <= 0 {
                    self.action_num = 1;
                    self.action_counter = 1; // Should be 0 but causes underflow as unsigned
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        self.vel_x -= 0x10;

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n335_ikachan[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n336_ikachan_generator(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 => {
                for player in players {
                    if player.shock_counter > 0 {
                        self.cond.set_alive(false);
                    }
                }
            }
            10 => {
                self.action_counter += 1;
                if self.action_counter % 4 == 1 {
                    let mut npc = NPC::create(335, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.rng.range(0..13) * 0x2000;
                    let _ = npc_list.spawn(0x100, npc);
                }
            }
            _ => (),
        }

        Ok(())
    }
}

impl BossNPC {
    pub(crate) fn tick_b05_ironhead(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        token: &mut NPCAccessToken
    ) {
        match self.parts[0].action_num {
            0 => {
                self.parts[0].cond.set_alive(true);
                self.parts[0].exp = 1;
                self.parts[0].direction = Direction::Right;
                self.parts[0].action_num = 100;
                self.parts[0].x = 0x14000;
                self.parts[0].y = 0x10000;
                self.hurt_sound[0] = 54;
                self.parts[0].npc_flags.set_event_when_killed(true);
                self.parts[0].npc_flags.set_shootable(true);
                self.parts[0].npc_flags.set_ignore_solidity(true);
                self.parts[0].npc_flags.set_show_damage(true);
                self.parts[0].size = 3;
                self.parts[0].damage = 10;
                self.parts[0].event_num = 1000;
                self.parts[0].life = 400;
                self.parts[0].display_bounds = Rect::new(0x5000, 0x1800, 0x3000, 0x1800);
                self.parts[0].hit_bounds = Rect::new(0x2000, 0x1400, 0x2000, 0x1400);
            }
            100 | 101 => {
                if self.parts[0].action_num == 100 {
                    self.parts[0].action_num = 101;
                    self.parts[0].action_counter = 0;
                    self.parts[0].npc_flags.set_shootable(false);
                }

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 50 {
                    self.parts[0].action_num = 250;
                    self.parts[0].action_counter = 0;
                }

                if self.parts[0].action_counter % 4 == 0 {
                    let mut npc = NPC::create(197, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].rng.range(15..18) * state.tile_size.as_int() * 0x200;
                    npc.y = self.parts[0].rng.range(2..13) * state.tile_size.as_int() * 0x200;

                    let _ = npc_list.spawn(0x100, npc);
                }
            }
            250 | 251 => {
                let player = self.parts[0].get_closest_player_ref(&players);
                let switch_buffer = state.constants.game.tile_offset_x * 0x2000; // Buffer to stop Ironhead's teleport to be visible
                if self.parts[0].action_num == 250 {
                    self.parts[0].action_num = 251;
                    if self.parts[0].direction == Direction::Right {
                        self.parts[0].x = 0x1E000 - switch_buffer;
                        self.parts[0].y = player.y;
                    } else {
                        self.parts[0].x = 0x5A000 + switch_buffer;
                        self.parts[0].y = self.parts[0].rng.range(2..13) * state.tile_size.as_int() * 0x200;
                    }

                    self.parts[0].target_x = self.parts[0].x;
                    self.parts[0].target_y = self.parts[0].y;
                    self.parts[0].vel_y = self.parts[0].rng.range(-0x200..0x200);
                    self.parts[0].vel_x = self.parts[0].rng.range(-0x200..0x200);
                    self.parts[0].npc_flags.set_shootable(true);
                }

                if self.parts[0].direction == Direction::Right {
                    self.parts[0].target_x += 0x400;
                } else {
                    self.parts[0].target_x -= 0x200;
                    if self.parts[0].target_y >= player.y {
                        self.parts[0].target_y -= 0x200;
                    } else {
                        self.parts[0].target_y += 0x200;
                    }
                }

                self.parts[0].vel_x += if self.parts[0].x >= self.parts[0].target_x { -8 } else { 8 };
                self.parts[0].vel_y += if self.parts[0].y >= self.parts[0].target_y { -8 } else { 8 };
                self.parts[0].vel_y = self.parts[0].vel_y.clamp(-0x200, 0x200);
                self.parts[0].x += self.parts[0].vel_x;
                self.parts[0].y += self.parts[0].vel_y;

                if self.parts[0].direction == Direction::Right {
                    if self.parts[0].x > 0x5A000 + switch_buffer {
                        self.parts[0].direction = Direction::Left;
                        self.parts[0].action_num = 100;
                    }
                } else if self.parts[0].x < 0x22000 - switch_buffer {
                    self.parts[0].direction = Direction::Right;
                    self.parts[0].action_num = 100;
                }

                if self.parts[0].direction == Direction::Left {
                    self.parts[0].action_counter += 1;
                    if [300, 310, 320].contains(&self.parts[0].action_counter) {
                        state.sound_manager.play_sfx(39);

                        let mut npc = NPC::create(198, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].x + 0x1400;
                        npc.y = self.parts[0].y + 0x200;
                        npc.vel_x = self.parts[0].rng.range(-3..0) * 0x200;
                        npc.vel_y = self.parts[0].rng.range(-3..3) * 0x200;
                        npc.direction = Direction::Right;

                        let _ = npc_list.spawn(0x100, npc);
                    }
                }

                self.parts[0].animate(2, 0, 7);
            }
            1000 | 1001 => {
                if self.parts[0].action_num == 1000 {
                    self.parts[0].action_num = 1001;
                    self.parts[0].anim_num = 8;
                    self.parts[0].npc_flags.set_shootable(false);
                    self.parts[0].damage = 0;
                    self.parts[0].target_x = self.parts[0].x;
                    self.parts[0].target_y = self.parts[0].y;
                    state.quake_counter = 20;
                    state.quake_rumble_counter = 20;
                    for _ in 0..32 {
                        let mut npc = NPC::create(4, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].x + self.parts[0].rng.range(-0x80..0x80) * 0x200;
                        npc.y = self.parts[0].y + self.parts[0].rng.range(-0x40..0x40) * 0x200;
                        npc.vel_x = self.parts[0].rng.range(-0x80..0x80) * 0x200;
                        npc.vel_y = self.parts[0].rng.range(-0x80..0x80) * 0x200;
                        npc.direction = Direction::Left;

                        let _ = npc_list.spawn(0x100, npc);
                    }

                    npc_list.kill_npcs_by_type(197, true, state, token);
                    npc_list.kill_npcs_by_type(271, true, state, token);
                    npc_list.kill_npcs_by_type(272, true, state, token);
                }

                self.parts[0].target_x -= 0x200;
                self.parts[0].x = self.parts[0].target_x + self.parts[0].rng.range(-1..1) * 0x200;
                self.parts[0].y = self.parts[0].target_y + self.parts[0].rng.range(-1..1) * 0x200;

                if self.parts[0].action_counter % 4 == 0 {
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x + self.parts[0].rng.range(-0x80..0x80) * 0x200;
                    npc.y = self.parts[0].y + self.parts[0].rng.range(-0x40..0x40) * 0x200;
                    npc.vel_x = self.parts[0].rng.range(-0x80..0x80) * 0x200;
                    npc.vel_y = self.parts[0].rng.range(-0x80..0x80) * 0x200;
                    npc.direction = Direction::Left;

                    let _ = npc_list.spawn(0x100, npc);
                }
            }
            _ => (),
        }

        let offset = if self.parts[0].shock != 0 {
            self.parts[19].action_counter += 1;
            if self.parts[19].action_counter & 2 != 0 {
                0
            } else {
                9
            }
        } else {
            0
        };

        self.parts[0].anim_rect = state.constants.npc.b05_ironhead[self.parts[0].anim_num as usize + offset];
    }
}
