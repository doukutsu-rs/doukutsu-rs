use crate::common::{Direction, Rect};
use crate::components::flash::Flash;
use crate::framework::error::GameResult;
use crate::npc::boss::BossNPC;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n340_ballos(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        flash: &mut Flash,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 => {
                self.action_num = 1;
                self.cond.set_alive(true);
                self.exp = 1;
                self.direction = Direction::Left;
                self.y -= 0xC00;
                self.damage = 0;

                let mut npc = NPC::create(341, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;
                npc.y = self.y - 0x2000;
                let _ = npc_list.spawn(0x100, npc);
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                }
                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 100;
                }
            }
            100 | 110 | 111 => {
                if self.action_num == 100 {
                    self.action_num = 110;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                    self.damage = 4;
                    self.npc_flags.set_shootable(true);
                }
                if self.action_num == 110 {
                    self.action_num = 111;
                    self.damage = 3;
                    self.target_x = self.life as i32;
                }

                self.animate(10, 1, 2);

                self.action_counter += 1;

                if (self.life as i32) < self.target_x - 50 || self.action_counter > 150 {
                    match self.action_counter3 % 5 {
                        0 | 1 | 2 | 3 => {
                            self.action_num = 200;
                        }
                        4 => {
                            self.action_num = 300;
                        }
                        _ => (),
                    }
                    self.action_counter3 += 1;
                }

                self.direction = if player.x < self.x { Direction::Left } else { Direction::Right };
            }
            200 | 201 | 202 => {
                if self.action_num == 200 {
                    self.action_num = 201;
                    self.action_counter2 = 0;
                }
                if self.action_num == 201 {
                    self.action_num = if self.vel_x == 0 { 202 } else { 203 };
                    self.action_counter = 0;
                    self.anim_num = 3;
                    self.damage = 3;
                    self.action_counter2 += 1;
                }

                self.direction = if player.x < self.x { Direction::Left } else { Direction::Right };

                self.vel_x = 8 * self.vel_x / 9;
                self.vel_y = 8 * self.vel_y / 9;

                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 210;
                }
            }
            203 => {
                self.vel_x = 8 * self.vel_x / 9;
                self.vel_y = 8 * self.vel_y / 9;

                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = if player.y < self.y + 0x1800 { 220 } else { 230 };
                }
            }
            210 | 211 => {
                if self.action_num == 210 {
                    self.action_num = 211;
                    self.action_counter = 0;
                    self.anim_num = 6;
                    self.anim_counter = 0;
                    self.damage = 10;

                    self.direction = if player.x < self.x { Direction::Left } else { Direction::Right };

                    state.sound_manager.play_sfx(25);
                }

                self.vel_x = if self.direction == Direction::Left { -0x800 } else { 0x800 };

                self.action_counter += 1;
                self.anim_num = if self.action_counter & 0x02 != 0 { 6 } else { 7 };

                if (self.direction == Direction::Left && self.flags.hit_left_wall())
                    || (self.direction == Direction::Right && self.flags.hit_right_wall())
                {
                    self.action_num = 212;
                    self.action_counter = 0;
                    self.damage = 3;
                    state.quake_counter = 10; // todo: super quake
                    state.sound_manager.play_sfx(26);
                }

                if self.action_counter2 < 4 && player.x > self.x - 0x2000 && player.x < self.x + 0x2000 {
                    self.action_num = 201;
                }
            }
            212 => {
                self.vel_x = 0;
                self.anim_num = 6;
                self.action_counter += 1;

                if self.action_counter > 30 {
                    self.action_num = if self.action_counter2 > 3 { 240 } else { 201 };
                }
            }
            220 | 221 | 230 | 231 => {
                if self.action_num == 220 || self.action_num == 230 {
                    self.action_num += 1;
                    self.action_counter = 0;
                    self.anim_num = 8;
                    self.anim_counter = 0;
                    self.vel_x = 0;
                    self.damage = 10;
                    self.direction = if self.action_num == 220 { Direction::Left } else { Direction::Right };
                    state.sound_manager.play_sfx(25);
                }
                self.vel_y = if self.action_num == 221 { -0x800 } else { 0x800 };

                self.anim_num = if self.action_counter & 0x02 != 0 { 8 } else { 9 };

                if (self.y < 0x6000 && self.action_num == 221)
                    || (self.flags.hit_bottom_wall() && self.action_num == 231)
                {
                    if self.action_num == 221 {
                        self.y = 0x6000;
                        self.vel_y = 0;
                    }

                    if self.action_num == 231 {
                        self.direction = if self.action_num == 220 { Direction::Left } else { Direction::Right };
                    }

                    self.action_num += 1;
                    self.action_counter = 0;
                    self.damage = 3;

                    for _ in 0..8 {
                        let mut npc = NPC::create(4, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.rng.range(-16..16) * 0x200;
                        npc.y = self.y - 0x1400;
                        let _ = npc_list.spawn(0x100, npc);
                    }

                    let mut npc = NPC::create(332, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x - 0x1800;
                    npc.y = self.y - 0x1800;
                    npc.direction = Direction::Left;
                    let _ = npc_list.spawn(0x100, npc);

                    let mut npc = NPC::create(332, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x + 0x1800;
                    npc.y = self.y - 0x1800;
                    npc.direction = Direction::Right;
                    let _ = npc_list.spawn(0x100, npc);

                    state.quake_counter = 10; // todo super quake
                    state.sound_manager.play_sfx(26);
                }

                if self.action_counter2 < 4 && player.y > self.y - 0x2000 && player.y < self.y + 0x2000 {
                    self.action_num = 201;
                }
            }
            222 => {
                self.action_counter += 1;
                self.vel_x = 0;
                self.anim_num = 8;

                if self.action_counter > 30 {
                    self.action_num = if self.action_counter2 > 3 { 240 } else { 201 };
                }
            }
            232 => {
                self.action_counter += 1;
                self.vel_x = 0;
                self.anim_num = 3;

                if self.action_counter > 30 {
                    self.action_num = if self.action_counter2 > 3 { 242 } else { 201 };
                }
            }
            240 | 241 => {
                if self.action_num == 240 {
                    self.action_num = 241;
                    self.direction = Direction::Left;
                }

                self.vel_y += 0x80;
                if self.vel_y > 0x5FF {
                    self.vel_y = 0x5FF;
                }

                self.anim_counter += 1;
                self.anim_num = if self.anim_counter & 0x02 != 0 { 4 } else { 5 };

                if self.flags.hit_bottom_wall() {
                    self.action_num = 242;
                    self.action_counter = 0;
                    self.anim_num = 3;

                    self.direction = if self.action_num == 220 { Direction::Left } else { Direction::Right };
                }
            }
            242 => {
                self.vel_x = (3 * self.vel_x) / 4;
                self.anim_num = 3;

                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 110;
                }
            }
            300 | 310 => {
                if self.action_num == 300 {
                    self.action_num = 310;
                    self.action_counter = 0;
                    self.vel_y = -0x600;

                    self.direction = if self.x > 0x28000 { Direction::Right } else { Direction::Left };
                    self.target_x = player.x;
                    self.target_y = 0x16000;
                    self.anim_counter = 0
                }
                self.anim_counter += 1;
                self.action_counter += 1;

                self.direction = if self.action_counter <= 200 || self.anim_counter >= 20 {
                    Direction::Right
                } else {
                    Direction::Left
                };
                self.anim_num = if self.anim_counter & 0x02 != 0 { 4 } else { 5 };
                self.vel_x += if self.x < self.target_x { 0x40 } else { -0x40 };
                self.vel_y += if self.y < self.target_y { 0x40 } else { -0x40 };

                self.vel_x = self.vel_x.clamp(-0x400, 0x400);
                self.vel_y = self.vel_y.clamp(-0x400, 0x400);

                if self.action_counter > 200 && self.action_counter % 40 == 1 {
                    self.anim_counter = 0;

                    let mut npc = NPC::create(333, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = 0x26000;
                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.action_counter > 480 {
                    self.action_num = 320;
                    self.action_counter = 0;
                }
            }
            320 => {
                self.vel_x = 0;
                self.vel_y = 0;
                self.direction = Direction::Right;

                self.action_counter += 1;
                if self.action_counter == 40 {
                    flash.set_blink();
                }

                if self.action_counter > 50 && self.action_counter % 10 == 1 {
                    let mut npc = NPC::create(333, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = ((4 * (self.action_counter as i32) - 200) / 10 + 2) * 0x2000;
                    npc.y = 0x26000;
                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.action_counter > 140 {
                    self.action_num = 240;
                }

                self.anim_counter += 1;
                self.anim_num = if self.anim_counter & 0x02 != 0 { 4 } else { 5 };
            }
            1000 | 1001 => {
                if self.action_num == 1000 {
                    self.action_num = 1001;
                    self.action_counter = 0;
                    self.anim_num = 10;
                    self.target_x = self.x;
                    self.vel_x = 0;
                    self.npc_flags.set_shootable(false);
                    npc_list.create_death_smoke(self.x, self.y, 16, 16, state, &self.rng);
                }
                self.vel_y += 0x20;
                if self.vel_y > 0x5FF {
                    self.vel_y = 0x5FF;
                }

                self.action_counter;
                self.x = self.target_x + if self.action_counter & 0x02 != 0 { 0x200 } else { -0x200 };
                if self.flags.hit_bottom_wall() {
                    self.action_num = 1002;
                    self.action_counter = 0;
                }
            }
            1002 => {
                self.action_counter += 1;
                if self.action_counter > 150 {
                    self.action_num = 1003;
                    self.action_counter = 0;
                    self.anim_num = 3;
                }

                self.x = self.target_x + if self.action_counter & 0x02 != 0 { 0x200 } else { -0x200 };
            }
            1003 => {
                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 1004;
                    self.action_counter = 0;
                    self.anim_num = 3;
                    self.vel_y -= 0xA00;
                    self.direction = Direction::Left;
                    self.npc_flags.set_ignore_solidity(true);
                }
            }
            1004 => {
                if self.y < 0 {
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.action_num = 1005;
                    self.action_counter = 0;
                    flash.set_blink();
                    state.sound_manager.play_sfx(29);
                }

                self.anim_counter += 1;
                self.anim_num = if self.anim_counter & 0x02 != 0 { 8 } else { 9 };
            }
            _ => (),
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 11 };
        self.anim_rect = state.constants.npc.n340_ballos[self.anim_num as usize + dir_offset];

        Ok(())
    }
}

impl BossNPC {
    pub(crate) fn tick_b09_ballos(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        flash: &mut Flash,
    ) {
        let player = self.parts[0].get_closest_player_mut(players);

        match self.parts[0].action_num {
            0 => {
                self.parts[0].action_num = 1;
                self.parts[0].cond.set_alive(true);
                self.parts[0].exp = 1;
                self.parts[0].direction = Direction::Left;
                self.parts[0].x = 0x28000;
                self.parts[0].y = -0x8000;
                self.hurt_sound[0] = 54;
                self.parts[0].hit_bounds = Rect { left: 0x4000, top: 0x6000, right: 0x4000, bottom: 0x6000 };
                self.parts[0].npc_flags.set_ignore_solidity(true);
                self.parts[0].npc_flags.set_solid_hard(true);
                self.parts[0].npc_flags.set_event_when_killed(true);
                self.parts[0].npc_flags.set_show_damage(true);
                self.parts[0].size = 3;
                self.parts[0].damage = 0;
                self.parts[0].event_num = 1000;
                self.parts[0].life = 800;

                self.parts[1].cond.set_alive(true);
                self.parts[1].cond.set_damage_boss(true);
                self.parts[1].direction = Direction::Left;
                self.parts[1].npc_flags.set_ignore_solidity(true);
                self.parts[1].display_bounds = Rect { left: 0x1800, top: 0, right: 0x1800, bottom: 0x2000 };
                self.parts[1].hit_bounds = Rect { left: 0x1800, top: 0, right: 0x1800, bottom: 0x2000 };

                self.parts[2] = self.parts[1].clone();
                self.parts[2].direction = Direction::Right;

                self.parts[3].cond.set_alive(true);
                self.parts[3].cond.set_damage_boss(true);
                self.parts[3].npc_flags.set_solid_hard(true); // This should be soft -- investigate bug with large soft collision boxes?
                self.parts[3].npc_flags.set_invulnerable(true);
                self.parts[3].npc_flags.set_ignore_solidity(true);
                self.parts[3].display_bounds = Rect { left: 0x7800, top: 0x7800, right: 0x7800, bottom: 0x7800 };
                self.parts[3].hit_bounds = Rect { left: 0x6000, top: 0x3000, right: 0x6000, bottom: 0x4000 };

                self.parts[4].cond.set_alive(true);
                self.parts[4].cond.set_damage_boss(true);
                self.parts[4].npc_flags.set_solid_soft(true);
                self.parts[4].npc_flags.set_invulnerable(true);
                self.parts[4].npc_flags.set_ignore_solidity(true);
                self.parts[4].hit_bounds = Rect { left: 0x4000, top: 0x1000, right: 0x4000, bottom: 0x1000 };

                self.parts[5].cond.set_alive(true);
                self.parts[5].cond.set_damage_boss(true);
                self.parts[5].npc_flags.set_solid_hard(true);
                self.parts[5].npc_flags.set_invulnerable(true);
                self.parts[5].npc_flags.set_ignore_solidity(true);
                self.parts[5].hit_bounds = Rect { left: 0x4000, top: 0, right: 0x4000, bottom: 0x6000 };
            }
            100 | 101 => {
                if self.parts[0].action_num == 100 {
                    self.parts[0].action_num = 101;
                    self.parts[0].anim_num = 0;
                    self.parts[0].x = player.x;
                    self.parts[0].action_counter = 0;

                    let mut npc = NPC::create(333, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = player.x;
                    npc.y = 0x26000;
                    npc.direction = Direction::Right;
                    let _ = npc_list.spawn(0x100, npc);
                }
                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 30 {
                    self.parts[0].action_num = 102;
                }
            }
            102 => {
                self.parts[0].vel_y += 0x40;
                if self.parts[0].vel_y > 0xC00 {
                    self.parts[0].vel_y = 0xC00
                }

                self.parts[0].y += self.parts[0].vel_y;

                if self.parts[0].y > 0x26000 - self.parts[0].hit_bounds.bottom as i32 {
                    self.parts[0].y = 0x26000 - self.parts[0].hit_bounds.bottom as i32;
                    self.parts[0].vel_y = 0;
                    self.parts[0].action_num = 103;
                    self.parts[0].action_counter = 0;
                    state.quake_counter = 30; // todo: super quake
                    state.sound_manager.play_sfx(44);

                    if player.y > self.parts[0].y + 0x6000
                        && player.x < self.parts[0].x + 0x3000
                        && player.x > self.parts[0].x - 0x3000
                    {
                        player.damage(16, state, npc_list);
                    }

                    for _ in 0..16 {
                        let mut npc = NPC::create(4, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].rng.range(-40..40) * 0x200;
                        npc.y = self.parts[0].y + 0x5000;
                        let _ = npc_list.spawn(0x100, npc);
                    }

                    if self.parts[0].flags.hit_bottom_wall() == true {
                        self.parts[0].vel_y = -0x200;
                    }
                }
            }
            103 => {
                self.parts[0].action_counter += 1;

                if self.parts[0].action_counter == 50 {
                    self.parts[0].action_num = 104;
                    self.parts[1].action_num = 100;
                    self.parts[2].action_num = 100;
                }
            }
            200 | 201 | 203 => {
                if self.parts[0].action_num == 200 {
                    self.parts[0].action_num = 201;
                    self.parts[0].action_counter2 = 0;
                }
                if self.parts[0].action_num == 201 {
                    self.parts[0].action_num = 203;
                    self.parts[0].vel_x = 0;
                    self.parts[0].hit_bounds.bottom = 0x6000;
                    self.parts[0].damage = 0;
                    self.parts[0].action_counter2 += 1;

                    if self.parts[0].action_counter2 % 3 > 0 {
                        self.parts[0].action_counter = 50;
                    } else {
                        self.parts[0].action_counter = 150;
                    }
                }

                if self.parts[0].action_counter > 0 {
                    self.parts[0].action_counter -= 1; // Underflow protection
                }

                if self.parts[0].action_counter <= 0 {
                    self.parts[0].action_num = 204;
                    self.parts[0].vel_y = -0xC00;

                    self.parts[0].vel_x = if self.parts[0].x < player.x { 0x200 } else { -0x200 };
                }
            }
            204 => {
                if self.parts[0].x < 0xA000 {
                    self.parts[0].vel_x = 0x200;
                } else if self.parts[0].x > 0x44000 {
                    self.parts[0].vel_x = -0x200;
                }

                self.parts[0].vel_y += 0x55;
                if self.parts[0].vel_y > 0xC00 {
                    self.parts[0].vel_y = 0xC00
                }

                self.parts[0].x += self.parts[0].vel_x;
                self.parts[0].y += self.parts[0].vel_y;

                if self.parts[0].y > 0x26000 - self.parts[0].hit_bounds.bottom as i32 {
                    self.parts[0].y = 0x26000 - self.parts[0].hit_bounds.bottom as i32;
                    self.parts[0].vel_y = 0;
                    self.parts[0].action_num = 201;
                    self.parts[0].action_counter = 0;
                    state.quake_counter = 30; // todo: super quake
                    state.sound_manager.play_sfx(26);
                    state.sound_manager.play_sfx(44);

                    if player.y > self.parts[0].y + 0x7000 {
                        player.damage(16, state, npc_list);
                    }

                    for sign in [-1, 1] {
                        let mut npc = NPC::create(332, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].x + 0x1800 * sign;
                        npc.y = self.parts[0].y + 0x6800;
                        npc.direction = if sign == -1 { Direction::Left } else { Direction::Right };
                        let _ = npc_list.spawn(0x100, npc);
                    }

                    for _ in 0..16 {
                        let mut npc = NPC::create(4, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].rng.range(-40..40) * 0x200;
                        npc.y = self.parts[0].y + 0x5000;
                        let _ = npc_list.spawn(0x100, npc);
                    }

                    if self.parts[0].flags.hit_bottom_wall() == true {
                        self.parts[0].vel_y = -0x200;
                    }
                }
            }
            220 | 221 => {
                if self.parts[0].action_num == 220 {
                    self.parts[0].action_num = 221;
                    self.parts[0].life = 1200;
                    self.parts[0].vel_x = 0;
                    self.parts[0].anim_num = 0;
                    self.parts[0].shock = 0;
                    self.parts[6].action_counter = 0; // flash

                    self.parts[1].action_num = 200;

                    self.parts[2].action_num = 200;
                }

                self.parts[0].vel_y += 0x40;
                if self.parts[0].vel_y > 0xC00 {
                    self.parts[0].vel_y = 0xC00;
                }

                self.parts[0].y += self.parts[0].vel_y;

                if self.parts[0].y > 0x26000 - self.parts[0].hit_bounds.bottom as i32 {
                    self.parts[0].y = 0x26000 - self.parts[0].hit_bounds.bottom as i32;
                    self.parts[0].vel_y = 0;
                    self.parts[0].action_num = 201;
                    self.parts[0].action_counter = 0;
                    state.quake_counter = 30; // todo: super quake
                    state.sound_manager.play_sfx(26);

                    for _ in 0..16 {
                        let mut npc = NPC::create(4, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].x + self.parts[0].rng.range(-40..40) * 0x200;
                        npc.y = self.parts[0].y + 0x5000;
                        let _ = npc_list.spawn(0x100, npc);
                    }

                    if self.parts[0].flags.hit_bottom_wall() {
                        self.parts[0].vel_y = -0x200;
                    }
                }
            }
            300 | 301 => {
                if self.parts[0].action_num == 300 {
                    self.parts[0].action_num = 301;
                    self.parts[0].action_counter = 0;

                    // Spawns the eye balls
                    for iter in 0..4 {
                        for side in [0, 0x220] {
                            let mut npc = NPC::create(342, &state.npc_table);
                            npc.cond.set_alive(true);
                            npc.x = self.parts[0].x;
                            npc.y = self.parts[0].y;
                            npc.parent_id = self.parts[0].id;
                            // This should be set to "direction" so we need to use something else
                            // NPC 342 sets action_counter3 to a flat value so we can extract the value passed here
                            npc.action_counter3 = (64 * iter) + side;
                            let _ = npc_list.spawn(0x5A, npc);
                        }
                    }

                    let mut npc = NPC::create(343, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x;
                    npc.y = self.parts[0].y;
                    let _ = npc_list.spawn(0x18, npc);

                    for sign in [-1, 1] {
                        let mut npc = NPC::create(344, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].x + 0x3000 * sign;
                        npc.y = self.parts[0].y + 0x4800;
                        npc.direction = if sign == -1 { Direction::Left } else { Direction::Right };
                        let _ = npc_list.spawn(0x20, npc);
                    }
                }

                self.parts[0].y += (0x1C200 - self.parts[0].y) / 8;

                self.parts[0].action_counter += 1;

                if self.parts[0].action_counter > 50 {
                    self.parts[0].action_num = 310;
                    self.parts[0].action_counter = 0;
                }
            }
            // Bouncing around the room
            311 => {
                self.parts[0].direction = Direction::Left;
                self.parts[0].vel_x = -0x3AA;
                self.parts[0].vel_y = 0;
                self.parts[0].x += self.parts[0].vel_x;

                if self.parts[0].x < 0xDE00 {
                    self.parts[0].x = 0xDE00;
                    self.parts[0].action_num = 312;
                }
            }
            312 => {
                self.parts[0].direction = Direction::Up;
                self.parts[0].vel_x = 0;
                self.parts[0].vel_y = -0x3AA;
                self.parts[0].y += self.parts[0].vel_y;

                if self.parts[0].y < 0xDE00 {
                    self.parts[0].y = 0xDE00;
                    self.parts[0].action_num = 313;
                }
            }
            313 => {
                self.parts[0].direction = Direction::Right;
                self.parts[0].vel_x = 0x3AA;
                self.parts[0].vel_y = 0;
                self.parts[0].x += self.parts[0].vel_x;

                if self.parts[0].x > 0x40200 {
                    self.parts[0].x = 0x40200;
                    self.parts[0].action_num = 314;
                }

                if self.parts[0].action_counter2 > 0 {
                    self.parts[0].action_counter2 -= 1;
                }

                if self.parts[0].action_counter2 == 0 && self.parts[0].x > 0x26000 && self.parts[0].x < 0x2A000 {
                    self.parts[0].action_num = 400;
                }
            }
            314 => {
                self.parts[0].direction = Direction::Bottom;
                self.parts[0].vel_x = 0;
                self.parts[0].vel_y = 0x3AA;
                self.parts[0].y += self.parts[0].vel_y;

                if self.parts[0].y > 0xDE00 {
                    self.parts[0].y = 0xDE00;
                    self.parts[0].action_num = 311;
                }
            }
            400 | 401 => {
                if self.parts[0].action_num == 400 {
                    self.parts[0].action_num = 401;
                    self.parts[0].action_counter = 0;
                    self.parts[0].vel_x = 0;
                    self.parts[0].vel_y = 0;
                    npc_list.kill_npcs_by_type(339, true, state);
                }

                self.parts[0].y += (0x13E00 - self.parts[0].y) / 8;

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 50 {
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_num = 410;

                    // platforms
                    for iter in 0..5 {
                        let mut npc = NPC::create(346, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].x;
                        npc.y = self.parts[0].y;
                        npc.parent_id = self.parts[0].id;
                        // Setting up action number 0 of NPC 346
                        npc.action_counter2 = 4 * 32 * iter;
                        let _ = npc_list.spawn(0x50, npc);
                    }

                    let mut npc = NPC::create(343, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x;
                    npc.y = self.parts[0].y;
                    let _ = npc_list.spawn(0x18, npc);

                    for sign in [-1, 1] {
                        let mut npc = NPC::create(344, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].x + 0x3000 * sign;
                        npc.y = self.parts[0].y + 0x4800;
                        npc.direction = if sign == -1 { Direction::Left } else { Direction::Right };
                        let _ = npc_list.spawn(0x20, npc);
                    }
                }
            }
            410 => {
                self.parts[0].action_counter += 1;

                if self.parts[0].action_counter > 50 {
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_num = 411;
                }
            }
            411 => {
                self.parts[0].action_counter += 1;

                if self.parts[0].action_counter % 30 == 1 {
                    let mut npc = NPC::create(348, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = (((self.parts[0].action_counter as i32 / 30) * 2) + 2) * 0x2000;
                    npc.y = 0x2A000;
                    let _ = npc_list.spawn(0x100, npc);
                }

                if (self.parts[0].action_counter / 3 % 2) > 0 {
                    state.sound_manager.play_sfx(26);
                }

                if self.parts[0].action_counter > 540 {
                    self.parts[0].action_num = 420;
                }
            }
            420 | 421 | 422 | 423 | 424 | 425 | 426 | 427 | 428 => {
                if self.parts[0].action_num == 420 {
                    self.parts[0].action_num = 421;
                    self.parts[0].action_counter = 0;
                    self.parts[0].anim_counter = 0;
                    state.quake_counter = 30; // todo: super quake
                    state.sound_manager.play_sfx(35);

                    self.parts[1].action_num = 102;
                    self.parts[2].action_num = 102;

                    for _ in 0..256 {
                        let mut npc = NPC::create(4, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].rng.range(-60..60) * 0x200;
                        npc.y = self.parts[0].rng.range(-60..60) * 0x200;
                        let _ = npc_list.spawn(0, npc);
                    }
                }

                self.parts[0].anim_counter += 1;

                let index = self.parts[0].action_num as usize - 421;
                let delay = [500, 200, 20, 200, 500, 200, 20, 200][index];

                if self.parts[0].anim_counter > delay {
                    self.parts[0].anim_counter = 0;
                    self.parts[0].action_num = ((index as u16 + 1) % 8) + 421;
                }
            }
            1000 | 1001 => {
                if self.parts[0].action_num == 1000 {
                    self.parts[0].action_num = 1001;
                    self.parts[0].action_counter = 0;

                    self.parts[1].action_num = 300;
                    self.parts[2].action_num = 300;

                    self.parts[0].npc_flags.set_solid_hard(false);
                    self.parts[0].npc_flags.set_solid_soft(false);
                    self.parts[3].npc_flags.set_solid_hard(false);
                    self.parts[3].npc_flags.set_solid_soft(false);
                    self.parts[4].npc_flags.set_solid_hard(false);
                    self.parts[4].npc_flags.set_solid_soft(false);
                    self.parts[5].npc_flags.set_solid_hard(false);
                    self.parts[5].npc_flags.set_solid_soft(false);
                }

                self.parts[0].action_counter += 1;

                if self.parts[0].action_counter % 12 == 0 {
                    state.sound_manager.play_sfx(44);
                }

                npc_list.create_death_smoke(
                    self.parts[0].x + self.parts[0].rng.range(-60..60) * 0x200,
                    self.parts[0].y + self.parts[0].rng.range(-60..60) * 0x200,
                    1,
                    1,
                    state,
                    &self.parts[0].rng,
                );

                if self.parts[0].action_counter > 150 {
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_num = 1002;
                    flash.set_cross(self.parts[0].x, self.parts[0].y);
                    state.sound_manager.play_sfx(35);
                }
            }
            1002 => {
                // todo super quake
                state.quake_counter = 40;

                self.parts[0].action_counter += 1;

                if self.parts[0].action_counter == 50 {
                    self.parts[0].cond.set_alive(false);
                    self.parts[1].cond.set_alive(false);
                    self.parts[2].cond.set_alive(false);
                    self.parts[3].cond.set_alive(false);
                    self.parts[4].cond.set_alive(false);
                    self.parts[5].cond.set_alive(false);

                    npc_list.kill_npcs_by_type(350, true, state);
                    npc_list.kill_npcs_by_type(348, true, state);
                }
            }
            _ => (),
        }

        if self.parts[0].action_num < 500 && 420 < self.parts[0].action_num {
            self.parts[3].npc_flags.set_shootable(true);
            self.parts[4].npc_flags.set_shootable(true);
            self.parts[5].npc_flags.set_shootable(true);

            self.parts[0].action_counter += 1;

            if self.parts[0].action_counter > 300 {
                self.parts[0].action_counter = 0;

                for _ in 0..8 {
                    let mut npc = NPC::create(350, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = ((if player.x > self.parts[0].x { 156 } else { 0 } + self.parts[0].rng.range(-4..4))
                        * 0x2000)
                        / 4;
                    npc.y = (self.parts[0].rng.range(8..68) * 0x2000) / 4;
                    let _ = npc_list.spawn(0x100, npc);
                }
            }

            match self.parts[0].action_counter {
                270 | 280 | 290 => {
                    let mut npc = NPC::create(353, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x;
                    npc.y = self.parts[0].y - 0x6800;
                    npc.direction = Direction::Up;
                    let _ = npc_list.spawn(0x100, npc);

                    state.sound_manager.play_sfx(39);

                    for _ in 0..4 {
                        let mut npc = NPC::create(4, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].x;
                        npc.y = self.parts[0].y - 0x6800;
                        let _ = npc_list.spawn(0x100, npc);
                    }
                }
                _ => (),
            }

            let limit = if self.parts[0].life > 500 { 10 } else { 4 };
            if self.parts[0].rng.range(0..limit) == 2 {
                let mut npc = NPC::create(270, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.parts[0].x + self.parts[0].rng.range(-40..40) * 0x200;
                npc.y = self.parts[0].y + self.parts[0].rng.range(0..40) * 0x200;
                npc.direction = Direction::Up;
                let _ = npc_list.spawn(0x100, npc);
            }
        }

        if self.parts[0].shock > 0 {
            self.parts[6].action_counter += 1;
            self.parts[3].anim_num = if self.parts[6].action_counter & 0x02 != 0 { 1 } else { 0 };
        } else {
            self.parts[3].anim_num = 0;
        }

        if self.parts[0].action_num > 420 {
            self.parts[3].anim_num += 2;
        }

        self.tick_b09_ballos_eye(1, state, npc_list);
        self.tick_b09_ballos_eye(2, state, npc_list);

        // Body
        self.parts[3].x = self.parts[0].x;
        self.parts[3].y = self.parts[0].y;
        self.parts[3].anim_rect = state.constants.npc.b09_ballos[self.parts[3].anim_num as usize];

        // Top
        self.parts[4].x = self.parts[0].x;
        self.parts[4].y = self.parts[0].y - 0x5800;

        // Bottom
        self.parts[5].x = self.parts[0].x;
        self.parts[5].y = self.parts[0].y;
    }

    fn tick_b09_ballos_eye(&mut self, i: usize, state: &mut SharedGameState, npc_list: &NPCList) {
        let (head, tail) = self.parts.split_at_mut(i);
        let base = &mut head[0];
        let part = &mut tail[0];

        match part.action_num {
            100 | 101 => {
                if part.action_num == 100 {
                    part.action_num = 101;
                    part.anim_num = 0;
                    part.anim_counter = 0
                }

                part.animate(2, 0, 3);

                if part.anim_num > 2 {
                    part.action_num = 102;
                }
            }
            102 => {
                part.anim_num = 3;
            }
            200 | 201 => {
                if part.action_num == 200 {
                    part.action_num = 201;
                    part.anim_num = 3;
                    part.anim_counter = 0;
                }

                // This is the animate boilerplate code except it counts backwards
                part.anim_counter += 1;
                if part.anim_counter > 2 {
                    part.anim_counter = 0;
                    part.anim_num -= 1;
                }
                if part.anim_num == 0 {
                    part.action_num = 202;
                }
            }
            300 => {
                part.action_num = 301;
                part.anim_num = 4;

                npc_list.create_death_smoke(
                    part.x + (0x800 * if part.direction == Direction::Left { -1 } else { 1 }),
                    part.y,
                    0x800,
                    10,
                    state,
                    &part.rng,
                );
            }
            _ => (),
        }

        part.x = base.x + (0x3000 * if part.direction == Direction::Left { -1 } else { 1 });
        part.y = base.y - 0x4800;

        if part.action_num < 300 {
            part.npc_flags.set_shootable(if part.anim_num != 3 { false } else { true });
        }

        let dir_offset = if part.direction == Direction::Left { 0 } else { 5 };
        part.anim_rect = state.constants.npc.b09_ballos[part.anim_num as usize + dir_offset + 4];
    }
}
