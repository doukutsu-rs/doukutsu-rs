use ggez::GameResult;
use num_traits::{abs, clamp};

use crate::caret::CaretType;
use crate::common::{CDEG_RAD, Direction, Rect};
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n002_behemoth(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.flags.hit_left_wall() {
            self.direction = Direction::Right;
        } else if self.flags.hit_right_wall() {
            self.direction = Direction::Left;
        }

        match self.action_num {
            0 => {
                self.vel_x = match self.direction {
                    Direction::Left => { -0x100 }
                    Direction::Right => { 0x100 }
                    _ => { 0 }
                };

                self.anim_counter += 1;
                if self.anim_counter > 8 {
                    self.anim_counter = 0;
                    self.anim_num = (self.anim_num + 1) % 3;
                    self.anim_rect = state.constants.npc.n002_behemoth[self.anim_num as usize + if self.direction == Direction::Right { 7 } else { 0 }];
                }

                if self.shock > 0 {
                    self.action_counter = 0;
                    self.action_num = 1;
                    self.anim_num = 4;
                    self.anim_rect = state.constants.npc.n002_behemoth[self.anim_num as usize + if self.direction == Direction::Right { 7 } else { 0 }];
                }
            }
            1 => {
                self.vel_x = (self.vel_x * 7) / 8;

                self.action_counter += 1;
                if self.action_counter > 40 {
                    if self.shock > 0 {
                        self.action_counter = 0;
                        self.action_num = 2;
                        self.anim_num = 6;
                        self.anim_counter = 0;
                        self.damage = 5;
                        self.anim_rect = state.constants.npc.n002_behemoth[self.anim_num as usize + if self.direction == Direction::Right { 7 } else { 0 }];
                    } else {
                        self.action_num = 0;
                        self.anim_counter = 0;
                    }
                }
            }
            2 => {
                self.vel_x = match self.direction {
                    Direction::Left => { -0x400 }
                    Direction::Right => { 0x400 }
                    _ => { 0 }
                };

                self.action_counter += 1;
                if self.action_counter > 200 {
                    self.action_num = 0;
                    self.damage = 1;
                }

                self.anim_counter += 1;
                if self.anim_counter > 5 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 6 {
                        self.anim_num = 5;
                        state.sound_manager.play_sfx(26);
                        state.quake_counter = 8;
                    }

                    self.anim_rect = state.constants.npc.n002_behemoth[self.anim_num as usize + if self.direction == Direction::Right { 7 } else { 0 }];
                }
            }
            _ => {}
        }

        self.vel_y += 0x40;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n005_green_critter(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.y += 3 * 0x200;
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                }

                let player = self.get_closest_player_mut(players);

                if self.x > player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }

                if self.target_x < 100 {
                    self.target_x += 1;
                }

                if self.action_counter >= 8
                    && self.x - (112 * 0x200) < player.x
                    && self.x + (112 * 0x200) > player.x
                    && self.y - (80 * 0x200) < player.y
                    && self.y + (80 * 0x200) > player.y {
                    if self.anim_num != 1 {
                        self.anim_num = 1;
                        self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                } else {
                    if self.action_counter < 8 {
                        self.action_counter += 1;
                    }

                    if self.anim_num != 0 {
                        self.anim_num = 0;
                        self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                }

                if self.shock > 0 {
                    self.action_num = 2;
                    self.action_counter = 0;

                    if self.anim_num != 0 {
                        self.anim_num = 0;
                        self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                }

                if self.action_counter >= 8
                    && self.target_x >= 100
                    && self.x - (64 * 0x200) < player.x
                    && self.x + (64 * 0x200) > player.x
                    && self.y - (80 * 0x200) < player.y
                    && self.y + (80 * 0x200) > player.y {
                    self.action_num = 2;
                    self.action_counter = 0;

                    if self.anim_num != 0 {
                        self.anim_num = 0;
                        self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 3;

                    if self.anim_num != 2 {
                        self.anim_num = 2;
                        self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }

                    self.vel_y = -0x5ff;
                    state.sound_manager.play_sfx(30);

                    if self.direction == Direction::Left {
                        self.vel_x = -0x100;
                    } else {
                        self.vel_x = 0x100;
                    }
                }
            }
            3 => {
                if self.flags.hit_bottom_wall() {
                    self.vel_x = 0;
                    self.action_counter = 0;
                    self.action_num = 1;

                    state.sound_manager.play_sfx(23);

                    if self.anim_num != 0 {
                        self.anim_num = 0;
                        self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                }
            }
            _ => {}
        }

        self.vel_y += 0x40;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n006_green_beetle(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;

                match self.direction {
                    Direction::Left => { self.action_num = 1; }
                    Direction::Right => { self.action_num = 3; }
                    _ => {}
                }
            }
            1 => {
                self.vel_x -= 0x10;

                if self.vel_x < -0x400 {
                    self.vel_x = -0x400;
                }

                if self.shock > 0 {
                    self.x += self.vel_x / 2;
                } else {
                    self.x += self.vel_x;
                }

                self.anim_counter += 1;
                if self.anim_counter > 1 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num > 2 {
                        self.anim_num = 1;
                    }
                }

                if self.flags.hit_left_wall() {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.vel_x = 0;
                    self.direction = Direction::Right;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 60 {
                    self.action_num = 3;
                    self.anim_counter = 0;
                    self.anim_num = 1;
                }
            }
            3 => {
                self.vel_x += 0x10;

                if self.vel_x > 0x400 {
                    self.vel_x = 0x400;
                }

                if self.shock > 0 {
                    self.x += self.vel_x / 2;
                } else {
                    self.x += self.vel_x;
                }

                self.anim_counter += 1;
                if self.anim_counter > 1 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num > 2 {
                        self.anim_num = 1;
                    }
                }

                if self.flags.hit_right_wall() {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.vel_x = 0;
                    self.direction = Direction::Left;
                }
            }
            4 => {
                self.action_counter += 1;
                if self.action_counter > 60 {
                    self.action_num = 1;
                    self.anim_counter = 0;
                    self.anim_num = 1;
                }
            }
            _ => {}
        }

        self.anim_rect = state.constants.npc.n006_green_beetle[self.anim_num as usize + if self.direction == Direction::Right { 5 } else { 0 }];

        Ok(())
    }

    pub(crate) fn tick_n007_basil(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 => {
                let player = self.get_closest_player_mut(players);
                self.x = player.x;

                if self.direction == Direction::Left {
                    self.action_num = 1;
                } else {
                    self.action_num = 2;
                }
            }
            1 => {
                self.vel_x -= 0x40;

                let player = self.get_closest_player_mut(players);
                if self.x < (player.x - 192 * 0x200) {
                    self.action_num = 2;
                }

                if self.flags.hit_left_wall() {
                    self.vel_x = 0;
                    self.action_num = 2;
                }
            }
            2 => {
                self.vel_x += 0x40;

                let player = self.get_closest_player_mut(players);
                if self.x > (player.x + 192 * 0x200) {
                    self.action_num = 1;
                }

                if self.flags.hit_right_wall() {
                    self.vel_x = 0;
                    self.action_num = 1;
                }
            }
            _ => {}
        }

        if self.vel_x < 0 {
            self.direction = Direction::Left;
        } else {
            self.direction = Direction::Right;
        }

        self.vel_x = clamp(self.vel_x, -0x5ff, 0x5ff);

        self.x += self.vel_x;

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num = (self.anim_num + 1) % 2;
        }

        if self.anim_counter == 1 {
            self.anim_rect = state.constants.npc.n007_basil[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
        }

        Ok(())
    }

    pub(crate) fn tick_n008_blue_beetle(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 => {
                let player = self.get_closest_player_mut(players);

                if player.x < self.x + 16 * 0x200 && player.x > self.x - 16 * 0x200 {
                    self.npc_flags.set_shootable(true);
                    self.vel_y = -0x100;
                    self.target_y = self.y;
                    self.action_num = 1;
                    self.damage = 2;

                    match self.direction {
                        Direction::Left => {
                            self.x = player.x + 256 * 0x200;
                            self.vel_x = -0x2ff;
                        }
                        Direction::Right => {
                            self.x = player.x - 256 * 0x200;
                            self.vel_x = 0x2ff;
                        }
                        _ => {}
                    }
                } else {
                    self.npc_flags.set_shootable(false);
                    self.anim_rect.left = 0;
                    self.anim_rect.right = 0;
                    self.damage = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;

                    return Ok(());
                }
            }
            1 => {
                let player = self.get_closest_player_mut(players);

                if self.x > player.x {
                    self.direction = Direction::Left;
                    self.vel_x -= 0x10;
                } else {
                    self.direction = Direction::Right;
                    self.vel_x += 0x10;
                }

                self.vel_y += if self.y < self.target_y { 8 } else { -8 };

                self.vel_x = clamp(self.vel_x, -0x2ff, 0x2ff);
                self.vel_y = clamp(self.vel_y, -0x100, 0x100);

                if self.shock > 0 {
                    self.x += self.vel_x / 2;
                    self.y += self.vel_y / 2;
                } else {
                    self.x += self.vel_x;
                    self.y += self.vel_y;
                }
            }
            _ => {}
        }

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num += 1;

            if self.anim_num > 1 {
                self.anim_num = 0;
            }
        }

        if self.anim_counter == 1 {
            self.anim_rect = state.constants.npc.n008_blue_beetle[self.anim_num as usize + if self.direction == Direction::Right { 2 } else { 0 }];
        }

        Ok(())
    }

    pub(crate) fn tick_n025_lift(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.x += 8 * 0x200;
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 1;
                }

                self.action_counter += 1;
                if self.action_counter > 150 {
                    self.action_num = 2;
                    self.action_counter = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 64 {
                    self.action_num = 3;
                    self.action_counter = 0;
                } else {
                    self.y -= 0x200;
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 150 {
                    self.action_num = 4;
                    self.action_counter = 0;
                }
            }
            4 => {
                self.action_counter += 1;
                if self.action_counter > 64 {
                    self.action_num = 5;
                    self.action_counter = 0;
                } else {
                    self.y -= 0x200;
                }
            }
            5 => {
                self.action_counter += 1;
                if self.action_counter > 150 {
                    self.action_num = 6;
                    self.action_counter = 0;
                }
            }
            6 => {
                self.action_counter += 1;
                if self.action_counter > 64 {
                    self.action_num = 7;
                    self.action_counter = 0;
                } else {
                    self.y += 0x200;
                }
            }
            7 => {
                self.action_counter += 1;
                if self.action_counter > 150 {
                    self.action_num = 8;
                    self.action_counter = 0;
                }
            }
            8 => {
                self.action_counter += 1;
                if self.action_counter > 64 {
                    self.action_num = 1;
                    self.action_counter = 0;
                } else {
                    self.y += 0x200;
                }
            }
            _ => {}
        }

        if [2, 4, 6, 8].contains(&self.action_num) {
            self.anim_counter += 1;
            if self.anim_counter > 1 {
                self.anim_num += 1;
                if self.anim_num > 1 {
                    self.anim_num = 0;
                }
            }
        }

        if self.anim_counter == 1 {
            self.anim_rect = state.constants.npc.n025_lift[self.anim_num as usize];
        }

        Ok(())
    }

    pub(crate) fn tick_n058_basu(&mut self, state: &mut SharedGameState, players: [&mut Player; 2], npc_list: &NPCList) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 => {
                if player.x < self.x + 16 * 0x200 && player.x > self.x - 16 * 0x200 {
                    self.target_x = self.x;
                    self.target_y = self.y;
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.action_counter2 = 0;
                    self.damage = 6;
                    self.vel_y = -0x100;
                    self.tsc_direction = self.direction as u16;
                    self.npc_flags.set_shootable(true);

                    self.x = player.x + self.direction.vector_x() * 16 * 16 * 0x200;
                    self.vel_x = self.direction.vector_x() * 0x2ff;
                } else {
                    self.anim_rect = Rect::new(0, 0, 0, 0);
                    self.damage = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.npc_flags.set_shootable(false);
                }

                return Ok(());
            }
            1 => {
                if self.x > player.x {
                    self.direction = Direction::Left;
                    self.vel_x -= 0x10;
                } else {
                    self.direction = Direction::Right;
                    self.vel_x += 0x10;
                }

                if self.flags.hit_left_wall() {
                    self.vel_x = 0x200;
                }

                if self.flags.hit_right_wall() {
                    self.vel_x = -0x200;
                }

                self.vel_y += ((self.target_y - self.y).signum() | 1) * 0x08;

                self.vel_x = clamp(self.vel_x, -0x2ff, 0x2ff);
                self.vel_y = clamp(self.vel_y, -0x100, 0x100);

                if self.shock > 0 {
                    self.x += self.vel_x / 2;
                    self.y += self.vel_y / 2;
                } else {
                    self.x += self.vel_x;
                    self.y += self.vel_y;
                }

                if player.x > self.x + 400 * 0x200 || player.x < self.x - 400 * 0x200 {
                    self.action_num = 0;
                    self.vel_x = 0;
                    self.x = self.target_x;
                    self.damage = 0;
                    self.direction = Direction::from_int_facing(self.tsc_direction as usize)
                        .unwrap_or(Direction::Left);
                    self.anim_rect = Rect::new(0, 0, 0, 0);
                    return Ok(());
                }
            }
            _ => {}
        }

        if self.action_counter < 150 {
            self.action_counter += 1;
        } else {
            self.action_counter2 += 1;
            if (self.action_counter2 % 8) == 0 && abs(self.x - player.x) < 160 * 0x200 {
                let angle = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64)
                    + self.rng.range(-6..6) as f64 * CDEG_RAD;

                let mut npc = NPC::create(84, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;
                npc.y = self.y;
                npc.vel_x = (angle.cos() * -1024.0) as i32;
                npc.vel_y = (angle.sin() * -1024.0) as i32;

                let _ = npc_list.spawn(0x100, npc);
                state.sound_manager.play_sfx(39);
            }

            if self.action_counter2 > 8 {
                self.action_counter = 0;
                self.action_counter2 = 0;
            }
        }

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > 1 {
                self.anim_num = 0;
            }
        }

        if self.action_counter > 120 && self.action_counter / 2 % 2 == 1 && self.anim_num == 1 {
            self.anim_num = 2;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n058_basu[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n084_basu_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_counter += 1;
        if self.anim_counter > 2 {
            self.anim_counter = 0;
            self.anim_num += 1;

            if self.anim_num > 3 {
                self.anim_num = 0;
            }
        }

        self.anim_rect = state.constants.npc.n084_basu_projectile[self.anim_num as usize];

        self.action_counter2 += 1;
        if self.flags.0 != 0 || self.action_counter2 > 300 {
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
            self.cond.set_alive(false);
        }

        Ok(())
    }

    pub(crate) fn tick_n207_counter_bomb_countdown(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = self.tsc_direction;
                    state.sound_manager.play_sfx(43);
                }

                self.x += 0x200;

                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 2;
                    self.action_counter = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.cond.set_alive(false);
                }
            }
            _ => {}
        }

        self.anim_rect = state.constants.npc.n207_counter_bomb_countdown[self.anim_num as usize % 5];

        Ok(())
    }

    pub(crate) fn tick_n208_basu_destroyed_egg_corridor(&mut self, state: &mut SharedGameState, players: [&mut Player; 2], npc_list: &NPCList) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 => {
                if player.x < self.x + 16 * 0x200 && player.x > self.x - 16 * 0x200 {
                    self.target_x = self.x;
                    self.target_y = self.y;
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.action_counter2 = 0;
                    self.damage = 6;
                    self.vel_y = -0x100;
                    self.tsc_direction = self.direction as u16;
                    self.npc_flags.set_shootable(true);

                    self.x = player.x + self.direction.vector_x() * 16 * 16 * 0x200;
                    self.vel_x = self.direction.vector_x() * 0x2ff;
                } else {
                    self.anim_rect = Rect::new(0, 0, 0, 0);
                    self.damage = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.npc_flags.set_shootable(false);
                }

                return Ok(());
            }
            1 => {
                if self.x > player.x {
                    self.direction = Direction::Left;
                    self.vel_x -= 0x10;
                } else {
                    self.direction = Direction::Right;
                    self.vel_x += 0x10;
                }

                if self.flags.hit_left_wall() {
                    self.vel_x = 0x200;
                }

                if self.flags.hit_right_wall() {
                    self.vel_x = -0x200;
                }

                self.vel_y += ((self.target_y - self.y).signum() | 1) * 0x08;

                self.vel_x = clamp(self.vel_x, -0x2ff, 0x2ff);
                self.vel_y = clamp(self.vel_y, -0x100, 0x100);

                if self.shock > 0 {
                    self.x += self.vel_x / 2;
                    self.y += self.vel_y / 2;
                } else {
                    self.x += self.vel_x;
                    self.y += self.vel_y;
                }

                if player.x > self.x + 400 * 0x200 || player.x < self.x - 400 * 0x200 {
                    self.action_num = 0;
                    self.vel_x = 0;
                    self.x = self.target_x;
                    self.damage = 0;
                    self.direction = Direction::from_int_facing(self.tsc_direction as usize)
                        .unwrap_or(Direction::Left);
                    self.anim_rect = Rect::new(0, 0, 0, 0);
                    return Ok(());
                }
            }
            _ => {}
        }

        if self.action_counter < 150 {
            self.action_counter += 1;
        } else {
            self.action_counter2 += 1;
            if (self.action_counter2 % 8) == 0 && abs(self.x - player.x) < 160 * 0x200 {
                let angle = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64)
                    + self.rng.range(-6..6) as f64 * CDEG_RAD;

                let mut npc = NPC::create(209, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;
                npc.y = self.y;
                npc.vel_x = (angle.cos() * -1536.0) as i32;
                npc.vel_y = (angle.sin() * -1536.0) as i32;

                let _ = npc_list.spawn(0x100, npc);
                state.sound_manager.play_sfx(39);
            }

            if self.action_counter2 > 16 {
                self.action_counter = 0;
                self.action_counter2 = 0;
            }
        }

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > 1 {
                self.anim_num = 0;
            }
        }

        if self.action_counter > 120 && self.action_counter / 2 % 2 == 1 && self.anim_num == 1 {
            self.anim_num = 2;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n208_basu_destroyed_egg_corridor[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n209_basu_projectile_destroyed_egg_corridor(&mut self, state: &mut SharedGameState) -> GameResult {
        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_counter += 1;
        if self.anim_counter > 2 {
            self.anim_counter = 0;
            self.anim_num += 1;

            if self.anim_num > 3 {
                self.anim_num = 0;
            }
        }

        self.anim_rect = state.constants.npc.n209_basu_projectile_destroyed_egg_corridor[self.anim_num as usize];

        self.action_counter2 += 1;
        if self.flags.0 != 0 || self.action_counter2 > 300 {
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
            self.cond.set_alive(false);
        }

        Ok(())
    }
}
