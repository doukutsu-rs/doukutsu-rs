use num_traits::{abs, clamp};

use crate::caret::CaretType;
use crate::common::{Direction, Rect, CDEG_RAD};
use crate::components::flash::Flash;
use crate::framework::error::GameResult;
use crate::npc::boss::BossNPC;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n158_fish_missile(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;

                    self.action_counter2 = match self.direction {
                        Direction::Left => 0xa0,
                        Direction::Up => 0xe0,
                        Direction::Right => 0x20,
                        Direction::Bottom => 0x60,
                        Direction::FacingPlayer => 0,
                    };
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
                } else {
                    if direction - radians < std::f64::consts::PI {
                        self.action_counter2 = (self.action_counter2 + 1) & 0xff;
                    } else {
                        self.action_counter2 = self.action_counter2.wrapping_sub(1) & 0xff;
                    }
                }
            }
            _ => {}
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

        self.anim_rect = state.constants.npc.n158_fish_missile[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n159_monster_x_defeated(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;

                    for _ in 0..8 {
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }
                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 2;
                    self.vel_x = -256;
                }
                self.x = if ((self.action_counter / 2) & 1) != 0 { self.x + 0x200 } else { self.x - 0x200 }
            }
            2 => {
                self.action_counter += 1;
                self.vel_y += 0x40;
                if self.y > 0x50000 {
                    self.cond.set_alive(false);
                }
            }
            _ => {}
        }

        self.y += self.vel_y;
        self.x += self.vel_x;

        self.anim_rect = state.constants.npc.n159_monster_x_defeated;

        if self.action_counter % 8 == 1 {
            let mut npc = NPC::create(4, &state.npc_table);
            npc.cond.set_alive(true);
            npc.direction = Direction::Left;

            npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
            npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
            npc.vel_x = self.rng.range(-0x155..0x155) as i32;
            npc.vel_y = self.rng.range(-0x600..0) as i32;

            let _ = npc_list.spawn(0x100, npc);
        }
        Ok(())
    }
}

impl BossNPC {
    pub(crate) fn tick_b03_monster_x(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        flash: &mut Flash,
    ) {
        match self.parts[0].action_num {
            0 => {
                self.parts[0].life = 1;
                self.parts[0].x = -320 * 0x200;
            }
            1 => {
                self.parts[0].life = 700;
                self.parts[0].exp = 1;
                self.parts[0].action_num = 2;
                self.parts[0].anim_num = 0;
                self.parts[0].x = 2048 * 0x200;
                self.parts[0].y = 200 * 0x200;
                self.parts[0].size = 3;
                self.parts[0].event_num = 1000;
                self.parts[0].hit_bounds =
                    Rect { left: 24 * 0x200, top: 24 * 0x200, right: 24 * 0x200, bottom: 24 * 0x200 };
                self.parts[0].npc_flags.set_ignore_solidity(true);
                self.parts[0].npc_flags.set_event_when_killed(true);
                self.parts[0].npc_flags.set_show_damage(true);
                self.hurt_sound[0] = 54;

                self.parts[1].cond.set_alive(true);
                self.parts[1].size = 3;
                self.parts[1].direction = Direction::Left;
                self.parts[1].display_bounds =
                    Rect { left: 24 * 0x200, top: 24 * 0x200, right: 24 * 0x200, bottom: 24 * 0x200 };
                self.parts[1].npc_flags.set_ignore_solidity(true);

                self.parts[2] = self.parts[1].clone();
                self.parts[2].direction = Direction::Right;

                self.parts[3].cond.set_alive(true);
                self.parts[3].life = 60;
                self.parts[3].size = 2;
                self.parts[3].target_x = 0;
                self.parts[3].display_bounds =
                    Rect { left: 0x1000, top: 0x1000, right: 0x1000, bottom: 0x1000 };
                self.parts[3].hit_bounds =
                    Rect { left: 5 * 0x200, top: 5 * 0x200, right: 5 * 0x200, bottom: 5 * 0x200 };
                self.parts[3].npc_flags.set_ignore_solidity(true);
                self.hurt_sound[3] = 54;
                self.death_sound[3] = 71;

                self.parts[4] = self.parts[3].clone();
                self.parts[3].target_x = 1;

                self.parts[5] = self.parts[3].clone();
                self.parts[6] = self.parts[3].clone();
                self.parts[5].target_x = 2;
                self.parts[6].target_x = 3;
                self.parts[5].life = 100;
                self.parts[6].life = 100;

                self.parts[7].cond.set_alive(true);
                self.parts[7].x = self.parts[0].x;
                self.parts[7].y = self.parts[0].y;
                self.parts[7].size = 3;
                self.parts[7].anim_num = 0;
                self.parts[7].display_bounds =
                    Rect { left: 52 * 0x200, top: 24 * 0x200, right: 52 * 0x200, bottom: 24 * 0x200 };
                self.parts[7].hit_bounds =
                    Rect { left: 0x1000, top: 24 * 0x200, right: 0x1000, bottom: 0x2000 };
                self.parts[7].npc_flags.set_ignore_solidity(true);

                self.parts[9].cond.set_alive(true);
                self.parts[9].x = self.parts[0].x - 64 * 0x200;
                self.parts[9].y = self.parts[0].y - 56 * 0x200;
                self.parts[9].size = 3;
                self.parts[9].action_num = 0;
                self.parts[9].direction = Direction::Up;
                self.parts[9].display_bounds =
                    Rect { left: 36 * 0x200, top: 0x1000, right: 36 * 0x200, bottom: 24 * 0x200 };
                self.parts[9].hit_bounds =
                    Rect { left: 28 * 0x200, top: 0x1000, right: 28 * 0x200, bottom: 0x2000 };
                self.hurt_sound[9] = 52;
                self.parts[9].npc_flags.set_rear_and_top_not_hurt(true);
                self.parts[9].npc_flags.set_ignore_solidity(true);
                self.parts[9].npc_flags.set_invulnerable(true);
                self.parts[9].npc_flags.set_solid_soft(true);

                self.parts[10] = self.parts[9].clone();
                self.parts[10].x = self.parts[0].x + 64 * 0x200;

                self.parts[11] = self.parts[9].clone();
                self.parts[11].x = self.parts[0].x - 64 * 0x200;
                self.parts[11].y = self.parts[0].y + 56 * 0x200;
                self.parts[11].direction = Direction::Bottom;
                self.parts[11].display_bounds.top = 24 * 0x200;
                self.parts[11].display_bounds.bottom = 0x1000;
                self.parts[11].hit_bounds.top = 0x2000;
                self.parts[11].hit_bounds.bottom = 0x1000;

                self.parts[12] = self.parts[11].clone();
                self.parts[12].x = self.parts[0].x + 64 * 0x200;

                self.parts[13] = self.parts[9].clone();
                self.parts[13].display_bounds =
                    Rect { left: 30 * 0x200, top: 0x2000, right: 42 * 0x200, bottom: 0x2000 };
                self.parts[13].action_counter2 = 9;
                self.parts[13].anim_num = 0;
                self.parts[13].npc_flags.0 = 0;
                self.parts[13].npc_flags.set_ignore_solidity(true);

                self.parts[14] = self.parts[13].clone();
                self.parts[14].action_counter2 = 10;
                self.parts[14].anim_num = 1;
                self.parts[14].display_bounds.left = 42 * 0x200;
                self.parts[14].display_bounds.right = 30 * 0x200;

                self.parts[15] = self.parts[13].clone();
                self.parts[15].action_counter2 = 11;
                self.parts[15].anim_num = 2;
                self.parts[15].display_bounds.top = 0x2000;
                self.parts[15].display_bounds.bottom = 0x2000;

                self.parts[16] = self.parts[15].clone();
                self.parts[16].action_counter2 = 12;
                self.parts[16].anim_num = 3;
                self.parts[16].display_bounds.left = 42 * 0x200;
                self.parts[16].display_bounds.right = 30 * 0x200;

                for npc in self.parts.iter_mut() {
                    npc.init_rng();
                }
            }
            10 | 11 => {
                if self.parts[0].action_num == 10 {
                    self.parts[0].action_num = 11;
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_counter2 = 0;
                }

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 100 {
                    self.parts[0].action_counter = 0;

                    let player_idx = self.parts[0].get_closest_player_idx_mut(&players);
                    self.parts[0].action_num = if self.parts[0].x > players[player_idx].x { 100 } else { 200 };
                }
            }
            100 | 101 => {
                if self.parts[0].action_num == 100 {
                    self.parts[0].action_num = 101;
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_counter2 += 1;
                }

                self.parts[0].action_counter += 1;

                match self.parts[0].action_counter {
                    4 => self.parts[9].action_num = 100,
                    8 => self.parts[10].action_num = 100,
                    10 => self.parts[11].action_num = 100,
                    12 => self.parts[12].action_num = 100,
                    _ => {}
                }

                if self.parts[0].action_counter > 120 && self.parts[0].action_counter2 > 2 {
                    self.parts[0].action_num = 300;
                }

                let player_idx = self.parts[0].get_closest_player_idx_mut(&players);
                if self.parts[0].action_counter > 121 && players[player_idx].x > self.parts[0].x {
                    self.parts[0].action_num = 200;
                }
            }
            200 | 201 => {
                if self.parts[0].action_num == 200 {
                    self.parts[0].action_num = 201;
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_counter2 += 1;
                }

                self.parts[0].action_counter += 1;

                match self.parts[0].action_counter {
                    4 => self.parts[9].action_num = 200,
                    8 => self.parts[10].action_num = 200,
                    10 => self.parts[11].action_num = 200,
                    12 => self.parts[12].action_num = 200,
                    _ => {}
                }

                if self.parts[0].action_counter > 120 && self.parts[0].action_counter2 > 2 {
                    self.parts[0].action_num = 400;
                }

                let player_idx = self.parts[0].get_closest_player_idx_mut(&players);
                if self.parts[0].action_counter > 121 && players[player_idx].x < self.parts[0].x {
                    self.parts[0].action_num = 100;
                }
            }
            300 | 301 => {
                if self.parts[0].action_num == 300 {
                    self.parts[0].action_num = 301;
                    self.parts[0].action_counter = 0;
                }

                self.parts[0].action_counter += 1;

                match self.parts[0].action_counter {
                    4 => self.parts[9].action_num = 300,
                    8 => self.parts[10].action_num = 300,
                    10 => self.parts[11].action_num = 300,
                    12 => self.parts[12].action_num = 300,
                    _ => {}
                }

                if self.parts[0].action_counter > 50 {
                    if !self.parts[3].cond.alive()
                        && !self.parts[4].cond.alive()
                        && !self.parts[5].cond.alive()
                        && !self.parts[6].cond.alive()
                    {
                        self.parts[0].action_num = 600;
                    } else {
                        self.parts[0].action_num = 500;
                    }
                }
            }
            400 | 401 => {
                if self.parts[0].action_num == 400 {
                    self.parts[0].action_num = 401;
                    self.parts[0].action_counter = 0;
                }

                self.parts[0].action_counter += 1;

                match self.parts[0].action_counter {
                    4 => self.parts[9].action_num = 400,
                    8 => self.parts[10].action_num = 400,
                    10 => self.parts[11].action_num = 400,
                    12 => self.parts[12].action_num = 400,
                    _ => {}
                }

                if self.parts[0].action_counter > 50 {
                    if !self.parts[3].cond.alive()
                        && !self.parts[4].cond.alive()
                        && !self.parts[5].cond.alive()
                        && !self.parts[6].cond.alive()
                    {
                        self.parts[0].action_num = 600;
                    } else {
                        self.parts[0].action_num = 500;
                    }
                }
            }
            500 | 501 => {
                if self.parts[0].action_num == 500 {
                    self.parts[0].action_num = 501;
                    self.parts[0].action_counter = 0;
                    self.parts[1].action_num = 10;
                    self.parts[2].action_num = 10;
                }

                self.parts[0].action_counter += 1;

                if self.parts[0].action_counter > 300 {
                    self.parts[0].action_num = 502;
                    self.parts[0].action_counter = 0;
                }

                if !self.parts[3].cond.alive()
                    && !self.parts[4].cond.alive()
                    && !self.parts[5].cond.alive()
                    && !self.parts[6].cond.alive()
                {
                    self.parts[0].action_num = 502;
                    self.parts[0].action_counter = 0;
                }
            }
            502 | 503 => {
                if self.parts[0].action_num == 502 {
                    self.parts[0].action_num = 503;
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_counter2 = 0;
                    self.parts[1].action_num = 20;
                    self.parts[2].action_num = 20;
                }

                self.parts[0].action_counter += 1;

                if self.parts[0].action_counter > 50 {
                    let player_idx = self.parts[0].get_closest_player_idx_mut(&players);
                    self.parts[0].action_num = if self.parts[0].x > players[player_idx].x { 100 } else { 200 };
                }
            }
            600 | 601 => {
                if self.parts[0].action_num == 600 {
                    self.parts[0].action_num = 601;
                    self.parts[0].action_counter = 0;
                    self.parts[0].vel_y2 = self.parts[0].life as i32;
                    self.parts[1].action_num = 30;
                    self.parts[2].action_num = 30;
                }

                self.parts[0].action_counter += 1;
                if (self.parts[0].life as i32) < self.parts[0].vel_y2.saturating_sub(200)
                    || self.parts[0].action_counter > 300
                {
                    self.parts[0].action_num = 602;
                    self.parts[0].action_counter = 0;
                }
            }
            602 | 603 => {
                if self.parts[0].action_num == 602 {
                    self.parts[0].action_num = 603;
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_counter2 = 0;
                    self.parts[1].action_num = 40;
                    self.parts[2].action_num = 40;
                }

                self.parts[0].action_counter += 1;

                if self.parts[0].action_counter > 50 {
                    let player_idx = self.parts[0].get_closest_player_idx_mut(&players);
                    self.parts[0].action_num = if self.parts[0].x > players[player_idx].x { 100 } else { 200 };
                }
            }
            1000 => {
                state.quake_counter = 2;

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter % 8 == 0 {
                    state.sound_manager.play_sfx(52);
                }

                let mut npc = NPC::create(4, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.parts[0].x + self.parts[0].rng.range(-72..72) as i32 * 0x200;
                npc.y = self.parts[0].y + self.parts[0].rng.range(-64..64) as i32 * 0x200;
                let _ = npc_list.spawn(0x100, npc);

                if self.parts[0].action_counter > 100 {
                    self.parts[0].action_num = 1001;
                    self.parts[0].action_counter = 0;
                    flash.set_cross(self.parts[0].x, self.parts[0].y);
                    state.sound_manager.play_sfx(35);
                }
            }
            1001 => {
                state.quake_counter = 40;
                self.parts[0].action_counter += 1;

                if self.parts[0].action_counter > 50 {
                    for part in self.parts.iter_mut() {
                        part.cond.set_alive(false);
                    }

                    for npc in npc_list.iter_alive() {
                        if npc.npc_type == 158 {
                            npc.cond.set_alive(false);
                        }
                    }

                    let mut npc = NPC::create(159, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x;
                    npc.y = self.parts[1].y - 24 * 0x200;

                    let _ = npc_list.spawn(0, npc);
                }
            }
            _ => {}
        }

        self.tick_b03_monster_x_track(9, state, &players);
        self.tick_b03_monster_x_track(10, state, &players);
        self.tick_b03_monster_x_track(11, state, &players);
        self.tick_b03_monster_x_track(12, state, &players);

        self.parts[0].x +=
            (((self.parts[9].x + self.parts[10].x + self.parts[11].x + self.parts[12].x) / 4) - self.parts[0].x) / 16;
        self.tick_b03_monster_x_face(7, state);

        self.tick_b03_monster_x_frame(13, state, npc_list);
        self.tick_b03_monster_x_frame(14, state, npc_list);
        self.tick_b03_monster_x_frame(15, state, npc_list);
        self.tick_b03_monster_x_frame(16, state, npc_list);

        self.tick_b03_monster_x_shield(1, state);
        self.tick_b03_monster_x_shield(2, state);

        if self.parts[3].cond.alive() {
            self.tick_b03_monster_x_eye(3, state, &players, npc_list);
        }
        if self.parts[4].cond.alive() {
            self.tick_b03_monster_x_eye(4, state, &players, npc_list);
        }
        if self.parts[5].cond.alive() {
            self.tick_b03_monster_x_eye(5, state, &players, npc_list);
        }
        if self.parts[6].cond.alive() {
            self.tick_b03_monster_x_eye(6, state, &players, npc_list);
        }

        if self.parts[0].life == 0 && self.parts[0].action_num < 1000 {
            self.parts[0].action_num = 1000;
            self.parts[0].action_counter = 0;
            self.parts[0].shock = 150;

            self.parts[9].action_num = 300;
            self.parts[10].action_num = 300;
            self.parts[11].action_num = 300;
            self.parts[12].action_num = 300;
        }
    }

    fn tick_b03_monster_x_face(&mut self, i: usize, state: &mut SharedGameState) {
        match self.parts[i].action_num {
            0 => {
                self.parts[0].npc_flags.set_shootable(false);
                self.parts[i].anim_num = 2;
            }
            10 | 11 => {
                if self.parts[i].action_num == 10 {
                    self.parts[i].action_num = 11;
                    self.parts[i].action_counter = (self.parts[i].target_x * 10 + 40) as u16;
                    self.parts[i].anim_num = 2;
                    self.parts[0].npc_flags.set_shootable(true);
                }

                if self.parts[0].shock > 0 {
                    self.parts[i].action_counter2 += 1;
                    if self.parts[i].action_counter2 / 2 % 2 != 0 {
                        self.parts[i].anim_num = 1;
                    } else {
                        self.parts[i].anim_num = 0;
                    }
                } else {
                    self.parts[i].anim_num = 0;
                }
            }
            _ => {}
        }

        self.parts[7].x = self.parts[0].x;
        self.parts[7].y = self.parts[0].y;

        self.parts[i].anim_rect = state.constants.npc.b03_monster_x[self.parts[i].anim_num as usize];
    }

    fn tick_b03_monster_x_track(&mut self, i: usize, state: &mut SharedGameState, players: &[&mut Player; 2]) {
        match self.parts[i].action_num {
            10 => {
                self.parts[i].anim_num = 0;
                self.parts[i].npc_flags.set_bouncy(false);
            }
            100 | 101 => {
                if self.parts[i].action_num == 100 {
                    self.parts[i].action_num = 101;
                    self.parts[i].action_counter = 0;
                    self.parts[i].anim_num = 2;
                    self.parts[i].anim_counter = 0;
                    self.parts[i].npc_flags.set_bouncy(true);
                }

                self.parts[i].action_counter += 1;
                if self.parts[i].action_counter > 30 {
                    self.parts[i].action_num = 102;
                }

                self.parts[i].animate(0, 2, 3);
                self.parts[i].vel_x -= 0x20;
            }
            102 | 103 => {
                if self.parts[i].action_num == 102 {
                    self.parts[i].action_num = 103;
                    self.parts[i].anim_num = 0;
                    self.parts[i].anim_counter = 0;
                    self.parts[i].npc_flags.set_bouncy(false);
                }

                self.parts[i].action_counter += 1;
                self.parts[i].animate(1, 0, 1);
                self.parts[i].vel_x -= 0x20;
            }
            200 | 201 => {
                if self.parts[i].action_num == 200 {
                    self.parts[i].action_num = 201;
                    self.parts[i].action_counter = 0;
                    self.parts[i].anim_num = 4;
                    self.parts[i].anim_counter = 0;
                    self.parts[i].npc_flags.set_bouncy(true);
                    self.parts[i].npc_flags.set_rear_and_top_not_hurt(true);
                }

                self.parts[i].action_counter += 1;
                if self.parts[i].action_counter > 30 {
                    self.parts[i].action_num = 202;
                }

                self.parts[i].animate(0, 4, 5);
                self.parts[i].vel_x += 0x20;
            }
            202 | 203 => {
                if self.parts[i].action_num == 202 {
                    self.parts[i].action_num = 203;
                    self.parts[i].anim_num = 0;
                    self.parts[i].anim_counter = 0;
                    self.parts[i].npc_flags.set_bouncy(false);
                }

                self.parts[i].action_counter += 1;

                self.parts[i].animate(1, 0, 1);
                self.parts[i].vel_x += 0x20;
            }
            300 | 301 => {
                if self.parts[i].action_num == 300 {
                    self.parts[i].action_num = 301;
                    self.parts[i].anim_num = 4;
                    self.parts[i].anim_counter = 0;
                    self.parts[i].npc_flags.set_bouncy(true);
                }

                self.parts[i].animate(0, 4, 5);

                self.parts[i].vel_x += 0x20;
                if self.parts[i].vel_x > 0 {
                    self.parts[i].vel_x = 0;
                    self.parts[i].action_num = 10;
                }
            }
            400 | 401 => {
                if self.parts[i].action_num == 400 {
                    self.parts[i].action_num = 401;
                    self.parts[i].anim_num = 2;
                    self.parts[i].anim_counter = 0;
                    self.parts[i].npc_flags.set_bouncy(true);
                }

                self.parts[i].animate(0, 2, 3);

                self.parts[i].vel_x -= 0x20;
                if self.parts[i].vel_x < 0 {
                    self.parts[i].vel_x = 0;
                    self.parts[i].action_num = 10;
                }
            }
            _ => {}
        }

        if self.parts[i].action_counter % 2 == 1 && [101, 201, 301, 401].contains(&self.parts[i].action_num) {
            state.sound_manager.play_sfx(112);
        }

        if self.parts[i].action_counter % 4 == 1 && [103, 203].contains(&self.parts[i].action_num) {
            state.sound_manager.play_sfx(111);
        }

        let player_idx = self.parts[i].get_closest_player_idx_mut(players);
        if self.parts[i].action_num >= 100 && abs(players[player_idx].y - self.parts[i].y) < 0x800 {
            self.parts[i].damage = 10;
            self.parts[i].npc_flags.set_rear_and_top_not_hurt(true);
        } else {
            self.parts[i].damage = 0;
            self.parts[i].npc_flags.set_rear_and_top_not_hurt(false);
        }

        self.parts[i].vel_x = clamp(self.parts[i].vel_x, -0x400, 0x400);
        self.parts[i].x += self.parts[i].vel_x;

        let dir_offset = if self.parts[i].direction == Direction::Up { 3 } else { 9 };

        self.parts[i].anim_rect = state.constants.npc.b03_monster_x[self.parts[i].anim_num as usize + dir_offset];
    }

    fn tick_b03_monster_x_frame(&mut self, i: usize, state: &mut SharedGameState, npc_list: &NPCList) {
        match self.parts[i].action_num {
            10 | 11 => {
                if self.parts[i].action_num == 10 {
                    self.parts[i].action_num = 11;
                    self.parts[i].action_counter = (self.parts[i].anim_num * 30 + 30) as u16;
                }

                if self.parts[i].action_counter > 0 {
                    self.parts[i].action_counter -= 1;
                } else {
                    self.parts[i].action_counter = 120;
                    state.sound_manager.play_sfx(39);

                    let mut npc = NPC::create(158, &state.npc_table);
                    npc.cond.set_alive(true);

                    match self.parts[i].anim_num {
                        0 => {
                            npc.direction = Direction::Bottom;
                            npc.x = self.parts[i].x + -30 * 0x200;
                            npc.y = self.parts[i].y + 0xc00;
                        }
                        1 => {
                            npc.direction = Direction::Right;
                            npc.x = self.parts[i].x + 30 * 0x200;
                            npc.y = self.parts[i].y + 0xc00;
                        }
                        2 => {
                            npc.direction = Direction::Left;
                            npc.x = self.parts[i].x - 30 * 0x200;
                            npc.y = self.parts[i].y - 0xc00;
                        }
                        3 => {
                            npc.direction = Direction::Up;
                            npc.x = self.parts[i].x + 30 * 0x200;
                            npc.y = self.parts[i].y - 0xc00;
                        }
                        _ => {}
                    }

                    let _ = npc_list.spawn(0x100, npc);
                }
            }
            _ => {}
        }

        self.parts[i].x = (self.parts[0].x + self.parts[self.parts[i].action_counter2 as usize].x) / 2;
        self.parts[i].y = (self.parts[0].y + self.parts[self.parts[i].action_counter2 as usize].y) / 2;

        self.parts[i].anim_rect = state.constants.npc.b03_monster_x[15 + self.parts[i].anim_num as usize];
    }

    fn tick_b03_monster_x_shield(&mut self, i: usize, state: &mut SharedGameState) {
        match self.parts[i].action_num {
            10 => {
                self.parts[i].target_x += 0x200;

                if self.parts[i].target_x > 32 * 0x200 {
                    self.parts[i].target_x = 32 * 0x200;
                    self.parts[i].action_num = 0;
                    self.parts[3].action_num = 10;
                    self.parts[4].action_num = 10;
                    self.parts[5].action_num = 10;
                    self.parts[6].action_num = 10;
                }
            }
            20 => {
                self.parts[i].target_x -= 0x200;

                if self.parts[i].target_x < 0 {
                    self.parts[i].target_x = 0;
                    self.parts[i].action_num = 0;
                    self.parts[3].action_num = 0;
                    self.parts[4].action_num = 0;
                    self.parts[5].action_num = 0;
                    self.parts[6].action_num = 0;
                }
            }
            30 => {
                self.parts[i].target_x += 0x200;

                if self.parts[i].target_x > 20 * 0x200 {
                    self.parts[i].target_x = 20 * 0x200;
                    self.parts[i].action_num = 0;
                    self.parts[7].action_num = 10;
                    self.parts[13].action_num = 10;
                    self.parts[14].action_num = 10;
                    self.parts[15].action_num = 10;
                    self.parts[16].action_num = 10;
                }
            }
            40 => {
                self.parts[i].target_x -= 0x200;

                if self.parts[i].target_x < 0 {
                    self.parts[i].target_x = 0;
                    self.parts[i].action_num = 0;
                    self.parts[7].action_num = 0;
                    self.parts[13].action_num = 0;
                    self.parts[14].action_num = 0;
                    self.parts[15].action_num = 0;
                    self.parts[16].action_num = 0;
                }
            }
            _ => {}
        }

        self.parts[i].x = self.parts[0].x + self.parts[i].direction.vector_x() * (24 * 0x200 + self.parts[i].target_x);
        self.parts[i].y = self.parts[0].y;

        let dir_offset = if self.parts[i].direction == Direction::Left { 19 } else { 20 };

        self.parts[i].anim_rect = state.constants.npc.b03_monster_x[dir_offset];
    }

    fn tick_b03_monster_x_eye(
        &mut self,
        i: usize,
        state: &mut SharedGameState,
        players: &[&mut Player; 2],
        npc_list: &NPCList,
    ) {
        match self.parts[i].action_num {
            0 => {
                self.parts[i].npc_flags.set_shootable(false);
                self.parts[i].anim_num = 0;
            }
            10 | 11 => {
                if self.parts[i].action_num == 10 {
                    self.parts[i].action_num = 11;
                    self.parts[i].action_counter = (self.parts[i].target_x * 10 + 40) as u16;
                    self.parts[i].npc_flags.set_shootable(true);
                }

                self.parts[i].anim_num =
                    if self.parts[i].action_counter < 16 && self.parts[i].action_counter / 2 % 2 != 0 { 1 } else { 0 };

                if self.parts[i].action_counter > 0 {
                    self.parts[i].action_counter -= 1;
                } else {
                    let player_idx = self.parts[i].get_closest_player_idx_mut(players);
                    let px = self.parts[i].x - players[player_idx].x;
                    let py = self.parts[i].y - players[player_idx].y;

                    let deg = f64::atan2(py as f64, px as f64) + self.parts[i].rng.range(-2..2) as f64 * CDEG_RAD;

                    let mut npc = NPC::create(156, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[i].x;
                    npc.y = self.parts[i].y;
                    npc.vel_x = (deg.cos() * -1536.0) as i32;
                    npc.vel_y = (deg.sin() * -1536.0) as i32;

                    let _ = npc_list.spawn(0x100, npc);

                    state.sound_manager.play_sfx(39);
                    self.parts[i].action_counter = 40;
                }
            }
            _ => {}
        }

        match self.parts[i].target_x {
            0 => {
                self.parts[i].x = self.parts[0].x - 22 * 0x200;
                self.parts[i].y = self.parts[0].y - 0x2000;
            }
            1 => {
                self.parts[i].x = self.parts[0].x + 28 * 0x200;
                self.parts[i].y = self.parts[0].y - 0x2000;
            }
            2 => {
                self.parts[i].x = self.parts[0].x - 15 * 0x200;
                self.parts[i].y = self.parts[0].y + 14 * 0x200;
            }
            3 => {
                self.parts[i].x = self.parts[0].x + 17 * 0x200;
                self.parts[i].y = self.parts[0].y + 14 * 0x200;
            }
            _ => {}
        }

        self.parts[i].anim_rect = state.constants.npc.b03_monster_x
            [21 + self.parts[i].target_x as usize + 4 * self.parts[i].anim_num as usize];
    }
}
