use crate::framework::context::Context;
use crate::framework::error::GameResult;
use num_traits::clamp;

use crate::caret::CaretType;
use crate::common::{CDEG_RAD, Direction};
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;
use crate::stage::Stage;

impl NPC {
    pub(crate) fn tick_n009_balrog_falling_in(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 2;
                }

                self.vel_y += 0x20;

                if self.action_counter2 > 39 {
                    self.npc_flags.set_ignore_solidity(false);
                    self.npc_flags.set_solid_soft(true);
                } else {
                    self.action_counter2 += 1;
                }

                if self.flags.hit_bottom_wall() {
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;

                    for _ in 0..3 {
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }

                    self.action_num = 2;
                    self.anim_num = 1;
                    self.action_counter = 0;

                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_counter = 3;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }
            }
            _ => {}
        }

        self.vel_y = clamp(self.vel_y, -0x5ff, 0x5ff);

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n009_balrog_falling_in[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n010_balrog_shooting(&mut self, state: &mut SharedGameState, players: [&mut Player; 2], npc_list: &NPCList) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                }

                self.action_counter += 1;
                if self.action_counter > 12 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.action_counter2 = 3;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_counter2 -= 1;
                    self.action_counter = 0;

                    let mut npc = NPC::create(11, &state.npc_table);

                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y + 4 * 0x200; // 4.0fix9

                    let mut angle = ((self.y + 4 * 0x200 - player.y) as f64 / (self.x - player.y) as f64).atan();
                    angle += self.rng.range(-16..16) as f64 * std::f64::consts::FRAC_PI_8;
                    npc.vel_x = (angle.cos() * 512.0) as i32; // 1.0fix9
                    npc.vel_y = (angle.sin() * 512.0) as i32;

                    let _ = npc_list.spawn(0x100, npc);

                    state.sound_manager.play_sfx(39);

                    if self.action_counter2 == 0 {
                        self.action_num = 3;
                        self.action_counter = 0;
                    }
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 3 {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.vel_x = (player.x - self.x) / 0x64;
                    self.vel_y = -3 * 0x200;
                    self.anim_num = 3;
                }
            }
            4 => {
                if self.flags.hit_left_wall() || self.flags.hit_right_wall() {
                    self.vel_x = 0;
                }

                self.damage = if self.y + 16 * 0x200 < player.y { 5 } else { 0 };

                if self.flags.hit_bottom_wall() {
                    self.action_num = 5;
                    self.action_counter = 5;
                    self.anim_num = 2;
                    self.damage = 0;

                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;
                }
            }
            5 => {
                self.vel_x = 0;

                self.action_counter += 1;
                if self.action_counter > 3 {
                    self.action_num = 1;
                    self.action_counter = 0;
                }
            }
            _ => {}
        }

        self.vel_y += 0x20;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.npc.n010_balrog_shooting[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n011_balrogs_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.flags.0 != 0 {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;

            if self.anim_num > 2 {
                self.anim_num = 0;
            }
        }

        self.anim_rect = state.constants.npc.n011_balrog_energy_shot[self.anim_num as usize];

        self.action_counter2 += 1;
        if self.action_counter2 > 150 {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        Ok(())
    }

    pub(crate) fn tick_n012_balrog_cutscene(&mut self, state: &mut SharedGameState, players: [&mut Player; 2], npc_list: &NPCList, stage: &mut Stage) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    if self.direction == Direction::FacingPlayer {
                        let player = self.get_closest_player_mut(players);

                        if self.x <= player.x {
                            self.direction = Direction::Right;
                        } else {
                            self.direction = Direction::Left;
                        }
                    }

                    self.action_num = 1;
                    self.anim_num = 0;
                }

                if self.rng.range(0..100) == 0 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    if self.direction == Direction::FacingPlayer {
                        let player = self.get_closest_player_mut(players);

                        if self.x <= player.x {
                            self.direction = Direction::Right;
                        } else {
                            self.direction = Direction::Left;
                        }
                    }

                    self.action_num = 11;
                    self.anim_num = 2;
                    self.action_counter = 0;
                    self.target_x = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 12;
                    self.action_counter = 0;
                    self.anim_num = 3;
                    self.vel_y = -4 * 0x200;
                    self.npc_flags.set_ignore_solidity(true);
                }
            }
            12 => {
                if self.flags.hit_left_wall() || self.flags.hit_right_wall() {
                    self.vel_x = 0;
                }

                if self.y < 0 {
                    self.npc_type = 0;
                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    if self.direction == Direction::FacingPlayer {
                        let player = self.get_closest_player_mut(players);

                        if self.x <= player.x {
                            self.direction = Direction::Right;
                        } else {
                            self.direction = Direction::Left;
                        }
                    }

                    self.action_num = 21;
                    self.anim_num = 5;
                    self.action_counter = 0;
                    self.action_counter2 = 0;

                    let mut npc = NPC::create(4, &state.npc_table);

                    for _ in 0..3 {
                        npc.cond.set_alive(true);
                        npc.direction = Direction::Left;
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }

                    state.sound_manager.play_sfx(72);
                }

                self.target_x = 1; // ???

                if self.flags.hit_bottom_wall() {
                    self.action_counter += 1;
                }

                self.action_counter2 += 1;
                self.x += if self.action_counter2 / 2 % 2 != 0 {
                    0x200
                } else {
                    -0x200
                };

                if self.action_counter > 100 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 2;
                }

                self.vel_y += 0x20;
                if self.vel_y > 0x5ff {
                    self.vel_y = 0x5ff;
                }
            }
            30 => {
                self.anim_num = 4;
                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 0;
                    self.anim_num = 0;
                }
            }
            40 | 41 => {
                if self.action_num == 40 {
                    if self.direction == Direction::FacingPlayer {
                        let player = self.get_closest_player_mut(players);

                        if self.x <= player.x {
                            self.direction = Direction::Right;
                        } else {
                            self.direction = Direction::Left;
                        }
                    }

                    self.action_num = 41;
                    self.action_counter = 0;
                    self.anim_num = 5;
                }

                self.anim_counter += 1;
                self.anim_num = if self.anim_counter / 2 % 2 != 0 {
                    5
                } else {
                    6
                };
            }
            42 | 43 => {
                if self.action_num == 42 {
                    if self.direction == Direction::FacingPlayer {
                        let player = self.get_closest_player_mut(players);

                        if self.x <= player.x {
                            self.direction = Direction::Right;
                        } else {
                            self.direction = Direction::Left;
                        }
                    }

                    self.action_num = 43;
                    self.anim_num = 6;
                    self.action_counter = 0;
                }

                self.anim_counter += 1;
                self.anim_num = if self.anim_counter / 2 % 2 != 0 {
                    7
                } else {
                    6
                };
            }
            50 => {
                self.anim_num = 8;
                self.vel_x = 0;
            }
            60 | 61 => {
                if self.action_num == 60 {
                    self.action_num = 61;
                    self.anim_num = 9;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_counter = 0;

                    self.anim_num += 1;
                    if self.anim_num == 10 || self.anim_num == 11 {
                        state.sound_manager.play_sfx(23);
                    }

                    if self.anim_num > 12 {
                        self.anim_num = 9;
                    }
                }

                self.vel_x = match self.direction {
                    Direction::Left => -0x200,
                    Direction::Right => 0x200,
                    _ => 0,
                };
            }
            70 | 71 => {
                if self.action_num == 70 {
                    self.action_num = 71;
                    self.action_counter = 64;
                    self.anim_num = 13;
                    state.sound_manager.play_sfx(29);
                }

                self.action_counter -= 1;
                if self.action_counter == 0 {
                    self.cond.set_alive(false);
                }
            }
            80 | 81 => {
                if self.action_num == 80 {
                    self.action_num = 81;
                    self.action_counter2 = 0;
                }

                self.action_counter2 += 1;
                self.x += if self.action_counter2 / 2 % 2 != 0 {
                    0x200
                } else {
                    -0x200
                };

                self.anim_num = 5;
                self.vel_x = 0;
                self.vel_y += 0x20;
            }
            100 | 101 => {
                if self.action_num == 100 {
                    self.action_num = 101;
                    self.action_counter = 0;
                    self.anim_num = 2;
                }

                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 102;
                    self.action_counter = 0;
                    self.anim_num = 3;
                    self.vel_y = -4 * 0x200;
                    self.npc_flags.set_ignore_solidity(true);

                    for npc in npc_list.iter_alive()
                        .filter(|npc| npc.npc_type == 117 || npc.npc_type == 150) {
                        npc.cond.set_alive(false)
                    }

                    let mut npc = NPC::create(355, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.parent_id = self.id;

                    let _ = npc_list.spawn(0x100, npc.clone());
                    npc.direction = Direction::Up;
                    let _ = npc_list.spawn(0x100, npc);
                }
            }
            102 => {
                let x = clamp(self.x / (16 * 0x200), 0, stage.map.width as i32) as usize;
                let y = clamp(self.y / (16 * 0x200), 0, stage.map.height as i32) as usize;

                if y <= 34 && stage.change_tile(x, y, 0) {
                    state.sound_manager.play_sfx(44);
                    state.quake_counter = 10;

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = x as i32 * 16 * 0x200;
                    npc.y = y as i32 * 16 * 0x200;

                    let _ = npc_list.spawn(0x100, npc.clone());
                    let _ = npc_list.spawn(0x100, npc.clone());

                    if x > 0 && stage.change_tile(x - 1, y, 0) {
                        npc.x = (x - 1) as i32 * 16 * 0x200;
                        let _ = npc_list.spawn(0x100, npc.clone());
                        let _ = npc_list.spawn(0x100, npc.clone());
                    }

                    if x < stage.map.width as usize && stage.change_tile(x + 1, y, 0) {
                        npc.x = (x + 1) as i32 * 16 * 0x200;
                        let _ = npc_list.spawn(0x100, npc.clone());
                        let _ = npc_list.spawn(0x100, npc);
                    }
                }

                println!("y: {}", self.y as f64 / 512.0);
                if self.y < -32 * 0x200 {
                    self.npc_type = 0;
                    state.quake_counter = 30;
                }
            }
            _ => {}
        }

        if self.target_x != 0 && self.rng.range(0..10) == 0 {
            let mut npc = NPC::create(4, &state.npc_table);
            npc.cond.set_alive(true);
            npc.direction = Direction::Left;
            npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
            npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
            npc.vel_x = self.rng.range(-0x155..0x155) as i32;
            npc.vel_y = self.rng.range(-0x600..0) as i32;

            let _ = npc_list.spawn(0x100, npc);
        }

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 14 };

        self.anim_rect = state.constants.npc.n012_balrog_cutscene[self.anim_num as usize + dir_offset];

        if self.action_num == 71 {
            self.anim_rect.bottom = self.anim_rect.top + self.action_counter as u16 / 2;
            if self.action_counter % 2 == 0 {
                self.anim_rect.left += 1;
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n019_balrog_bust_in(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;

                    for _ in 0..16 {
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }

                    state.sound_manager.play_sfx(12);
                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;

                    self.y += 10 * 0x200;
                    self.action_num = 1;
                    self.anim_num = 3;
                    self.vel_y = -0x100;
                }

                self.vel_y += 0x10;

                if self.vel_y > 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 2;
                    self.anim_num = 2;
                    self.action_counter = 0;

                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_num = 3;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }
            }
            3 => {
                if self.rng.range(0..100) == 0 {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            4 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_num = 3;
                    self.anim_num = 0;
                }
            }
            _ => {}
        }

        self.vel_y = clamp(self.vel_y, -0x5ff, 0x5ff);

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.npc.n019_balrog_bust_in[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n033_balrog_bouncing_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.flags.hit_left_wall() || self.flags.hit_right_wall() {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        } else if self.flags.hit_bottom_wall() {
            self.vel_y = -2 * 0x200;
        }

        self.vel_y += 0x2a;

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_counter += 1;
        if self.anim_counter > 2 {
            self.anim_counter = 0;

            self.anim_num += 1;
            if self.anim_num > 1 {
                self.anim_num = 0;
            }
        }

        self.anim_rect = state.constants.npc.n033_balrog_bouncing_projectile[self.anim_num as usize];

        self.action_counter += 1;
        if self.action_counter > 250 {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        Ok(())
    }

    pub(crate) fn tick_n036_balrog_hover(&mut self, state: &mut SharedGameState, players: [&mut Player; 2], npc_list: &NPCList) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                }

                self.action_counter += 1;
                if self.action_counter > 12 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.vel_x2 = 3;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.vel_x2 -= 1;
                    self.action_counter = 0;

                    let angle = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64)
                        + self.rng.range(-16..16) as f64 * CDEG_RAD;

                    let mut npc = NPC::create(11, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.vel_x = (angle.cos() * -512.0) as i32;
                    npc.vel_y = (angle.sin() * -512.0) as i32;
                    npc.x = self.x;
                    npc.y = self.y + 4 * 0x200;

                    let _ = npc_list.spawn(0x100, npc);
                    state.sound_manager.play_sfx(39);

                    if self.vel_x2 == 0 {
                        self.action_num = 3;
                        self.action_counter = 0;
                    }
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 3 {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.anim_num = 3;

                    self.vel_x = (player.x - self.x) / 100;
                    self.vel_y = -3 * 0x200;
                }
            }
            4 => {
                if self.vel_y > -0x200 {
                    if self.life > 60 {
                        self.action_num = 5;
                        self.action_counter = 0;
                        self.anim_num = 4;
                        self.anim_counter = 0;
                        self.target_y = self.y;
                    } else {
                        self.action_num = 6;
                    }
                }
            }
            5 => {
                self.anim_counter += 1;
                if self.anim_counter > 1 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 5 {
                        self.anim_num = 4;
                        state.sound_manager.play_sfx(47);
                    }
                }

                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 6;
                    self.anim_num = 3;
                }

                self.vel_y += ((self.target_y - self.y).signum() | 1) * 0x40;
                self.vel_y = clamp(self.vel_y, -0x200, 0x200);
            }
            6 => {
                if self.y + 16 * 0x200 < player.y {
                    self.damage = 10;
                } else {
                    self.damage = 0;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 7;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.damage = 0;

                    state.sound_manager.play_sfx(25);
                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;

                    let mut npc_smoke = NPC::create(4, &state.npc_table);
                    npc_smoke.cond.set_alive(true);
                    npc_smoke.direction = Direction::Left;

                    let mut npc_proj = NPC::create(33, &state.npc_table);
                    npc_proj.cond.set_alive(true);
                    npc_proj.direction = Direction::Left;

                    for _ in 0..8 {
                        npc_smoke.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc_smoke.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc_smoke.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc_smoke.vel_y = self.rng.range(-0x600..0) as i32;
                        let _ = npc_list.spawn(0x100, npc_smoke.clone());

                        npc_proj.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc_proj.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc_proj.vel_x = self.rng.range(-0x400..0x400) as i32;
                        npc_proj.vel_y = self.rng.range(-0x400..0) as i32;

                        let _ = npc_list.spawn(0x100, npc_proj.clone());
                    }
                }
            }
            7 => {
                self.vel_x = 0;

                self.action_counter += 1;
                if self.action_counter > 3 {
                    self.action_num = 1;
                    self.action_counter = 0;
                }
            }
            _ => {}
        }

        if self.action_num != 5 {
            self.vel_y += 0x33;
            self.direction = if self.x < player.x { Direction::Right } else { Direction::Left };
        }

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n036_balrog_hover[self.anim_num as usize + dir_offset];

        Ok(())
    }

    /// note: vel_y2 stores currently caught player
    pub(crate) fn tick_n068_balrog_running(&mut self, state: &mut SharedGameState, mut players: [&mut Player; 2], npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.action_counter = 30;

                    let player = self.get_closest_player_mut(players);
                    if self.x > player.x {
                        self.direction = Direction::Left;
                    } else {
                        self.direction = Direction::Right;
                    }
                }

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 2;
                    self.action_counter2 += 1;
                }
            }
            2 | 3 => {
                if self.action_num == 2 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num == 2 || self.anim_num == 4 {
                        state.sound_manager.play_sfx(23);
                    }

                    if self.anim_num > 4 {
                        self.anim_num = 1;
                    }
                }

                self.vel_x += 0x10 * self.direction.vector_x(); // 0.03125fix9

                let pi = self.get_closest_player_idx_mut(&players);
                if self.action_counter >= 8 && (players[pi].x - self.x).abs() < 12 * 0x200 // 12.0fix9
                    && self.y - 12 * 0x200 < players[pi].y && self.y + 8 * 0x200 > players[pi].y { // 12.0fix9 / 8.0fix9
                    self.action_num = 10;
                    self.anim_num = 5;
                    self.vel_y2 = pi as i32;
                    players[pi].cond.set_hidden(true);
                    players[pi].damage(2, state, npc_list);
                } else {
                    self.action_counter += 1;

                    if (self.flags.hit_left_wall() || self.flags.hit_right_wall()) || self.action_counter > 75 {
                        self.action_num = 9;
                        self.anim_num = 0;
                    } else if (self.action_counter2 % 3 == 0) && self.action_counter > 25 {
                        self.action_num = 4;
                        self.anim_num = 7;
                        self.vel_y = -2 * 0x200; // -2.0fix9
                    }
                }
            }
            4 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 9;
                    self.anim_num = 8;
                    state.quake_counter = 30;
                    state.sound_manager.play_sfx(26);
                }

                let pi = self.get_closest_player_idx_mut(&players);
                if self.action_counter >= 8 && (players[pi].x - self.x).abs() < 12 * 0x200
                    && self.y - 12 * 0x200 < players[pi].y && self.y + 8 * 0x200 > players[pi].y {
                    self.action_num = 10;
                    self.anim_num = 5;
                    self.vel_y2 = pi as i32;
                    players[pi].cond.set_hidden(true);
                    players[pi].damage(2, state, npc_list);
                }
            }
            9 => {
                self.vel_x = self.vel_x * 4 / 5;

                if self.vel_x == 0 {
                    self.action_num = 0;
                }
            }
            10 => {
                let player = &mut players[self.vel_y2 as usize];
                player.x = self.x;
                player.y = self.y;

                self.vel_x = self.vel_x * 4 / 5;

                if self.vel_x == 0 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 5;
                    self.anim_counter = 0;
                }
            }
            11 => {
                let player = &mut players[self.vel_y2 as usize];
                player.x = self.x;
                player.y = self.y;

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num > 6 {
                        self.anim_num = 5;
                    }
                }

                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 20;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    let player = &mut players[self.vel_y2 as usize];
                    state.sound_manager.play_sfx(25);
                    player.cond.set_hidden(false);

                    self.direction = self.direction.opposite();

                    player.direction = self.direction;
                    player.x += 4 * 0x200 * self.direction.vector_x();
                    player.y -= 8 * 0x200;
                    player.vel_x = 0x5ff * self.direction.vector_x();
                    player.vel_y = -0x200;

                    self.action_num = 21;
                    self.action_counter = 0;
                    self.anim_num = 7;
                }

                self.action_counter += 1;
                if self.action_counter >= 50 {
                    self.action_num = 0;
                }
            }
            _ => {}
        }

        self.vel_x = clamp(self.vel_x, -0x400, 0x400);
        self.vel_y += 0x20;

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 9 };

        self.anim_rect = state.constants.npc.n068_balrog_running[self.anim_num as usize + dir_offset];

        Ok(())
    }
}

