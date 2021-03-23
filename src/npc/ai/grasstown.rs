use num_traits::abs;
use num_traits::clamp;

use crate::caret::CaretType;
use crate::common::{Direction, Rect};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::npc::{NPCList, NPC};
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n024_power_critter(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.y += 3 * 0x200;
                    self.action_num = 1;
                }

                if self.action_counter >= 8
                    && abs(self.x - player.x) < (128 * 0x200)
                    && self.y - 128 * 0x200 < player.y
                    && self.y + 48 * 0x200 > player.y
                {
                    if self.x > player.x {
                        self.direction = Direction::Left;
                    } else {
                        self.direction = Direction::Right;
                    }

                    self.anim_num = 1;
                } else {
                    if self.action_counter < 8 {
                        self.action_counter += 1;
                    }

                    self.anim_num = 0;
                }

                if self.shock > 0 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }

                if self.action_counter >= 8
                    && abs(self.x - player.x) < 96 * 0x200
                    && self.y - 96 * 0x200 < player.y
                    && self.y + 48 * 0x200 > player.y
                {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    if self.x > player.x {
                        self.direction = Direction::Left;
                    } else {
                        self.direction = Direction::Right;
                    }

                    self.action_num = 3;
                    self.anim_num = 2;
                    self.vel_x = self.direction.vector_x() * 0x100;
                    self.vel_y = -0x5ff;
                    state.sound_manager.play_sfx(108);
                }
            }
            3 => {
                if self.vel_y > 0x200 {
                    self.target_y = self.y;
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.anim_num = 3;
                }
            }
            4 => {
                if self.x >= player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }

                self.action_counter += 1;

                if (self.flags.hit_left_wall() || self.flags.hit_right_wall() || self.flags.hit_top_wall())
                    || self.action_counter > 100
                {
                    self.action_num = 5;
                    self.anim_num = 2;
                    self.vel_x /= 2;
                    self.damage = 12;
                } else {
                    if self.action_counter % 4 == 1 {
                        state.sound_manager.play_sfx(110);
                    }

                    self.animate(0, 3, 5);
                }
            }
            5 => {
                if self.flags.hit_bottom_wall() {
                    self.vel_x = 0;
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.damage = 2;

                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;
                }
            }
            _ => {}
        }

        if self.action_num != 4 {
            self.vel_y += 0x40;
            if self.vel_y > 0x5ff {
                self.vel_y = 0x5ff;
            }
        } else {
            self.vel_x = clamp(self.vel_x + if self.x < player.x { 0x20 } else { -0x20 }, -0x200, 0x200);
            self.vel_y = clamp(self.vel_y + if self.y > self.target_y { -0x10 } else { 0x10 }, -0x200, 0x200);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };
        self.anim_rect = state.constants.npc.n024_power_critter[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n026_bat_flying(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    let angle = self.rng.range(0..0xff);
                    self.vel_x = ((angle as f64 * 1.40625).cos() * 512.0) as i32;
                    self.target_x =
                        self.x + ((angle as f64 * 1.40625 + std::f64::consts::FRAC_2_PI).cos() * 8.0 * 512.0) as i32;

                    let angle = self.rng.range(0..0xff);
                    self.vel_y = ((angle as f64 * 1.40625).sin() * 512.0) as i32;
                    self.target_y =
                        self.y + ((angle as f64 * 1.40625 + std::f64::consts::FRAC_2_PI).sin() * 8.0 * 512.0) as i32;

                    self.action_num = 1;
                    self.action_counter2 = 120;
                }

                if self.x > player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }

                self.vel_x = clamp(self.vel_x + 0x10 * (self.target_x - self.x).signum(), -0x200, 0x200);
                self.vel_y = clamp(self.vel_y + 0x10 * (self.target_y - self.y).signum(), -0x200, 0x200);

                if self.action_counter2 < 120 {
                    self.action_counter2 += 1;
                } else if abs(self.x - player.x) < 8 * 0x200 && self.y < player.y && self.y + 96 * 0x200 > player.y {
                    self.vel_x /= 2;
                    self.vel_y = 0;
                    self.action_num = 3;
                    self.npc_flags.set_ignore_solidity(false);
                }
            }
            3 => {
                self.vel_y += 0x40;
                if self.vel_y > 0x5ff {
                    self.vel_y = 0x5ff;
                }

                if self.flags.hit_bottom_wall() {
                    self.vel_x *= 2;
                    self.vel_y = 0;
                    self.action_counter2 = 0;
                    self.action_num = 1;
                    self.npc_flags.set_ignore_solidity(true);
                }
            }
            _ => {}
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.action_num == 3 {
            self.anim_num = 3;
        } else {
            self.animate(1, 0, 2);
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };
        self.anim_rect = state.constants.npc.n026_bat_flying[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n028_flying_critter(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.y += 3 * 0x200;
                    self.action_num = 1;
                }

                if self.action_counter >= 8
                    && abs(self.x - player.x) < 96 * 0x200
                    && self.y - 128 * 0x200 < player.y
                    && self.y + 48 * 0x200 > player.y
                {
                    if self.x > player.x {
                        self.direction = Direction::Left;
                    } else {
                        self.direction = Direction::Right;
                    }

                    self.anim_num = 1;
                } else {
                    if self.action_counter < 8 {
                        self.action_counter += 1;
                    }

                    self.anim_num = 0;
                }

                if self.shock > 0 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }

                if self.action_counter >= 8
                    && abs(self.x - player.x) < 96 * 0x200
                    && self.y - 96 * 0x200 < player.y
                    && self.y + 48 * 0x200 > player.y
                {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    if self.x > player.x {
                        self.direction = Direction::Left;
                    } else {
                        self.direction = Direction::Right;
                    }

                    self.action_num = 3;
                    self.anim_num = 2;
                    self.vel_x = self.direction.vector_x() * 0x100;
                    self.vel_y = -0x4cc;
                    state.sound_manager.play_sfx(30);
                }
            }
            3 => {
                if self.vel_y > 0x100 {
                    self.target_y = self.y;
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.anim_num = 3;
                }
            }
            4 => {
                if self.x > player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }

                self.action_counter += 1;

                if (self.flags.hit_left_wall() || self.flags.hit_right_wall() || self.flags.hit_top_wall())
                    || self.action_counter > 100
                {
                    self.action_num = 5;
                    self.anim_num = 2;
                    self.vel_x /= 2;
                    self.damage = 3;
                } else {
                    if self.action_counter % 4 == 1 {
                        state.sound_manager.play_sfx(110);
                    }

                    if self.flags.hit_bottom_wall() {
                        self.vel_y = -0x200;
                    }

                    self.animate(0, 3, 5);
                }
            }
            5 => {
                if self.flags.hit_bottom_wall() {
                    self.vel_x = 0;
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.damage = 2;

                    state.sound_manager.play_sfx(23);
                }
            }
            _ => {}
        }

        if self.action_num != 4 {
            self.vel_y += 0x40;
            if self.vel_y > 0x5ff {
                self.vel_y = 0x5ff;
            }
        } else {
            self.vel_x = clamp(self.vel_x + if self.x < player.x { 0x20 } else { -0x20 }, -0x200, 0x200);
            self.vel_y = clamp(self.vel_y + if self.y > self.target_y { -0x10 } else { 0x10 }, -0x200, 0x200);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };
        self.anim_rect = state.constants.npc.n028_flying_critter[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n031_bat_hanging(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                if abs(self.x - player.x) < 8 * 0x200 && self.y - 8 * 0x200 < player.y && self.y + 96 * 0x200 > player.y
                {
                    self.action_num = 3;
                    self.anim_num = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            3 => {
                self.anim_num = 0;

                if self.shock > 0 || abs(self.x - player.x) > 20 * 0x200 {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }
            }
            4 => {
                self.vel_y += 0x20;
                if self.vel_y > 0x5ff {
                    self.vel_y = 0x5ff;
                }

                self.action_counter += 1;
                if self.action_counter >= 20 && (self.flags.hit_bottom_wall() || self.y > player.y - 16 * 0x200) {
                    self.action_num = 5;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                    self.target_y = self.y;

                    if self.flags.hit_bottom_wall() {
                        self.vel_y = -0x200;
                    }
                }
            }
            5 => {
                self.animate(1, 2, 4);
                self.direction = if player.x < self.x { Direction::Left } else { Direction::Right };

                self.vel_x += (player.x - self.x).signum() * 0x10;
                self.vel_y += (self.target_y - self.y).signum() * 0x10;

                self.vel_x = clamp(self.vel_x, -0x200, 0x200);
                self.vel_y = clamp(self.vel_y, -0x200, 0x200);

                if self.flags.hit_bottom_wall() {
                    self.vel_y = -0x200;
                }

                if self.flags.hit_top_wall() {
                    self.vel_y = 0x200;
                }
            }
            _ => {}
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 5 };

        self.anim_rect = state.constants.npc.n031_bat_hanging[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n035_mannan(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if self.action_num <= 2 && self.life < 90 {
            self.action_num = 3;
            self.action_counter = 0;
            self.anim_num = 2;
            self.damage = 0;
            self.npc_flags.set_shootable(false);

            npc_list.create_death_smoke(self.x, self.y, self.display_bounds.right, 8, state, &self.rng);
            self.create_xp_drop(state, npc_list);

            state.sound_manager.play_sfx(71);
        }

        if self.action_num == 2 {
            self.action_counter += 1;
            if self.action_counter > 20 {
                self.action_counter = 0;
                self.action_num = 1;
                self.anim_num = 0;
            }
        } else if self.action_num > 2 {
            if self.action_num == 3 {
                self.action_counter += 1;
                if self.action_counter == 50 || self.action_counter == 60 {
                    self.anim_num = 3;
                }
                if self.action_counter == 53 || self.action_counter == 63 {
                    self.anim_num = 2;
                }
                if self.action_counter > 100 {
                    self.action_num = 4;
                }
            }
        } else if self.shock > 0 {
            self.action_num = 2;
            self.action_counter = 0;
            self.anim_num = 1;

            let mut npc = NPC::create(103, &state.npc_table);
            npc.cond.set_alive(true);
            npc.x = self.x + self.direction.vector_x() * 8 * 0x200;
            npc.y = self.y + 8 * 0x200;
            npc.direction = self.direction;

            let _ = npc_list.spawn(0x100, npc);
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.npc.n035_mannan[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n094_kulala(&mut self, state: &SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 => {
                self.anim_num = 4;
                if self.shock > 0 {
                    self.anim_num = 0;
                    self.action_num = 10;
                    self.action_counter = 0;
                }
            }
            10 => {
                self.npc_flags.set_shootable(true);
                self.npc_flags.set_invulnerable(false);

                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_counter = 0;
                }
            }
            11 => {
                self.anim_counter += 1;
                if self.anim_counter > 5 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 2 {
                        self.action_num = 12;
                        self.anim_num = 3;
                    }
                }
            }
            12 => {
                self.vel_y = -0x155;

                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 10;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }
            }
            20 => {
                self.vel_x /= 2;
                self.vel_y += 0x10;

                if self.shock == 0 {
                    self.action_num = 10;
                    self.action_counter = 30;
                    self.anim_num = 0;
                }
            }
            _ => {}
        }

        if self.shock > 0 {
            self.action_counter2 += 1;
            if self.action_counter2 > 12 {
                self.action_num = 20;
                self.anim_num = 4;
                self.npc_flags.set_shootable(false);
                self.npc_flags.set_invulnerable(true);
            }
        } else {
            self.action_counter2 = 0;
        }

        if self.action_num >= 10 {
            if self.flags.hit_left_wall() {
                self.vel_x2 = 50;
                self.direction = Direction::Right;
            }

            if self.flags.hit_right_wall() {
                self.vel_x2 = 50;
                self.direction = Direction::Left;
            }

            if self.vel_x2 > 0 {
                self.vel_x2 -= 1;

                self.vel_x += self.direction.vector_x() * 0x80;
            } else {
                let player = self.get_closest_player_mut(players);

                self.vel_x2 = 50;
                self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
            }

            self.vel_y += 0x10;

            if self.flags.hit_bottom_wall() {
                self.vel_y = -2 * 0x200;
            }
        }

        self.vel_x = clamp(self.vel_x, -0x100, 0x100);
        self.vel_y = clamp(self.vel_y, -0x300, 0x300);

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n094_kulala[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n095_jelly(&mut self, state: &SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 | 10 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = self.rng.range(0..50) as u16;
                    self.target_x = self.x;
                    self.target_y = self.y;

                    self.vel_x = -self.direction.vector_x() * 0x200;
                }

                if self.action_num == 1 {
                    if self.action_counter > 0 {
                        self.action_counter -= 1;
                    } else {
                        self.action_num = 10;
                    }
                }

                if self.action_num == 10 {
                    self.action_counter += 1;
                    if self.action_counter > 10 {
                        self.action_num = 11;
                        self.action_counter = 0;
                        self.anim_counter = 0;
                    }
                }
            }
            11 => {
                self.anim_counter += 1;
                if self.anim_counter > 5 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }

                if self.anim_num == 2 {
                    self.vel_x += self.direction.vector_x() * 0x100;
                    self.vel_y -= 0x200;
                } else if self.anim_num > 2 {
                    self.action_num = 12;
                    self.anim_num = 3;
                }
            }
            12 => {
                self.action_counter += 1;
                if self.action_counter > 10 && self.y > self.target_y {
                    self.action_num = 10;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }
            }
            _ => {}
        }

        self.direction = if self.x <= self.target_x { Direction::Right } else { Direction::Left };

        if self.flags.hit_left_wall() {
            self.action_counter2 = 50;
            self.direction = Direction::Right;
        }

        if self.flags.hit_right_wall() {
            self.action_counter2 = 50;
            self.direction = Direction::Left;
        }

        self.vel_y += 0x20;
        if self.flags.hit_bottom_wall() {
            self.vel_y = -2 * 0x200;
        }

        self.vel_x = clamp(self.vel_x, -0x100, 0x100);
        self.vel_y = clamp(self.vel_y, -0x200, 0x200);

        if self.shock > 0 {
            self.x += self.vel_x / 2;
            self.y += self.vel_y / 2;
        } else {
            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.npc.n095_jelly[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n100_grate(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.y += 16 * 0x200;
            self.action_num = 1;

            self.anim_rect = if self.direction == Direction::Left {
                state.constants.npc.n100_grate[0]
            } else {
                state.constants.npc.n100_grate[1]
            };
        }

        Ok(())
    }

    pub(crate) fn tick_n101_malco_screen(&mut self, state: &mut SharedGameState) -> GameResult {
        self.animate(3, 0, 2);
        self.anim_rect = state.constants.npc.n101_malco_screen[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n102_malco_computer_wave(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y += 8 * 0x200;
        }

        self.animate(0, 0, 3);
        self.anim_rect = state.constants.npc.n102_malco_computer_wave[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n103_mannan_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
        }

        self.vel_x += self.direction.vector_x() * 0x20;

        self.animate(0, 0, 2);

        self.x += self.vel_x;

        self.action_counter2 += 1;
        if self.action_counter2 > 100 {
            self.cond.set_alive(false);
        }

        if self.action_counter2 % 4 == 1 {
            state.sound_manager.play_sfx(46);
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n103_mannan_projectile[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n104_frog(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 => {
                self.action_num = 1;
                self.action_counter = 0;
                self.vel_x = 0;
                self.vel_y = 0;

                if self.tsc_direction == 4 {
                    self.direction = if (self.rng.next_u16() & 1) == 0 { Direction::Left } else { Direction::Right };
                    self.tsc_direction = self.direction as u16;
                    self.action_num = 3;
                    self.anim_num = 2;
                    self.npc_flags.set_ignore_solidity(true);
                } else {
                    self.npc_flags.set_ignore_solidity(false);
                }

                self.action_counter += 1;

                if self.rng.range(0..50) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }
            }
            1 => {
                self.action_counter += 1;

                if self.rng.range(0..50) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                self.animate(2, 0, 1);

                if self.action_counter > 18 {
                    self.action_num = 1;
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.npc_flags.set_ignore_solidity(false);
                }

                if self.flags.hit_bottom_wall() {
                    self.anim_num = 0;
                    self.action_num = 0;
                    self.action_counter = 0;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                }

                if self.flags.hit_left_wall() && self.vel_x < 0 {
                    self.vel_x = -self.vel_x;
                    self.direction = Direction::Right;
                }

                if self.flags.hit_right_wall() && self.vel_x > 0 {
                    self.vel_x = -self.vel_x;
                    self.direction = Direction::Left;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 0;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }
            }
            _ => {}
        }

        if self.action_num <= 9
            && self.action_num != 3
            && self.action_counter > 10
            && ((self.shock > 0)
                || (abs(self.x - player.x) < 160 * 0x200 && abs(self.y - player.y) < 64 * 0x200)
                    && self.rng.range(0..50) == 2)
        {
            self.direction = if self.x >= player.x { Direction::Left } else { Direction::Right };
            self.action_num = 10;
            self.anim_num = 2;
            self.vel_x = self.direction.vector_x() * 0x200;
            self.vel_y = -0x5ff;

            if !player.cond.hidden() {
                state.sound_manager.play_sfx(30);
            }
        }

        self.vel_y = (self.vel_y + 0x80).min(0x5ff);

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n104_frog[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n107_malco_broken(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;

                if self.direction == Direction::Right {
                    self.anim_num = 5;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_counter = 0;

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    for _ in 0..4 {
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }

                self.anim_counter += 1;
                if self.anim_counter > 1 {
                    state.sound_manager.play_sfx(43);

                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 1 {
                        self.anim_num = 0;
                    }
                }

                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 12;
                }
            }
            12 | 13 => {
                if self.action_num == 12 {
                    self.action_num = 13;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 14;
                }
            }
            14 | 15 => {
                if self.action_num == 14 {
                    self.action_num = 15;
                    self.action_counter = 0;
                }

                if self.action_counter / 2 % 2 != 0 {
                    self.x += 0x200;
                    state.sound_manager.play_sfx(11);
                } else {
                    self.x -= 0x200;
                }

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 16;
                }
            }
            16 | 17 => {
                if self.action_num == 16 {
                    self.action_num = 17;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    state.sound_manager.play_sfx(12);

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    for _ in 0..8 {
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }

                self.action_counter += 1;
                if self.action_counter > 150 {
                    self.action_num = 18;
                }
            }
            18 | 19 => {
                if self.action_num == 18 {
                    self.action_num = 19;
                    self.action_counter = 0;
                    self.anim_num = 3;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 4 {
                        self.anim_num = 3;
                        state.sound_manager.play_sfx(11);
                    }
                }

                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 20;
                    state.sound_manager.play_sfx(12);

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    for _ in 0..4 {
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }
            }
            20 => {
                self.anim_num = 4;
            }
            21 => {
                self.action_num = 22;
                self.anim_num = 5;

                state.sound_manager.play_sfx(51);
            }
            100 | 101 => {
                if self.action_num == 100 {
                    self.action_num = 101;
                    self.anim_num = 6;
                    self.anim_counter = 0;
                }

                self.animate(4, 6, 9);
            }
            110 => {
                npc_list.create_death_smoke(self.x, self.y, 16 * 0x200, 16, state, &self.rng);
                self.cond.set_alive(false);
            }
            _ => {}
        }

        self.anim_rect = state.constants.npc.n107_malco_broken[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n109_malco_powered_on(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    if self.action_counter > 0 {
                        self.action_counter -= 1;
                    } else {
                        self.action_num = 1;
                    }

                    self.anim_num = 0;
                    self.action_counter = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                let player = self.get_closest_player_mut(players);

                if abs(self.x - player.x) < 32 * 0x200
                    && self.y - 32 * 0x200 < player.y
                    && self.y + 16 * 0x200 > player.y
                {
                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            10 => {
                self.action_num = 0;
                state.sound_manager.play_sfx(12);

                let mut npc = NPC::create(4, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;
                npc.y = self.y;
                for _ in 0..8 {
                    npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                    npc.vel_y = self.rng.range(-0x600..0) as i32;

                    let _ = npc_list.spawn(0x100, npc.clone());
                }
            }
            _ => {}
        }

        self.vel_y += 0x40;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n109_malco_powered_on[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n110_puchi(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 => {
                self.action_num = 1;
                self.action_counter = 0;
                self.vel_x = 0;
                self.vel_y = 0;

                if self.tsc_direction == 4 {
                    self.direction = if (self.rng.next_u16() & 1) == 0 { Direction::Left } else { Direction::Right };
                    self.tsc_direction = self.direction as u16;
                    self.action_num = 3;
                    self.anim_num = 2;
                    self.npc_flags.set_ignore_solidity(true);
                } else {
                    self.npc_flags.set_ignore_solidity(false);
                }

                self.action_counter += 1;

                if self.rng.range(0..50) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }
            }
            1 => {
                self.action_counter += 1;

                if self.rng.range(0..50) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }
            }
            2 => {
                self.action_counter += 1;

                self.animate(2, 0, 1);

                if self.action_counter > 18 {
                    self.action_num = 1;
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.npc_flags.set_ignore_solidity(false);
                }

                if self.flags.hit_bottom_wall() {
                    self.anim_num = 0;
                    self.action_num = 0;
                    self.action_counter = 0;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                }

                if self.flags.hit_left_wall() && self.vel_x < 0 {
                    self.vel_x = -self.vel_x;
                    self.direction = Direction::Right;
                }

                if self.flags.hit_right_wall() && self.vel_x > 0 {
                    self.vel_x = -self.vel_x;
                    self.direction = Direction::Left;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 0;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }
            }
            _ => {}
        }

        if self.action_num <= 9
            && self.action_num != 3
            && self.action_counter > 10
            && ((self.shock > 0)
                || (abs(self.x - player.x) < 160 * 0x200 && abs(self.y - player.y) < 64 * 0x200)
                    && self.rng.range(0..50) == 2)
        {
            self.direction = if self.x >= player.x { Direction::Left } else { Direction::Right };
            self.action_num = 10;
            self.anim_num = 2;
            self.vel_x = self.direction.vector_x() * 0x100;
            self.vel_y = -0x2ff;

            if !player.cond.hidden() {
                state.sound_manager.play_sfx(30);
            }
        }

        self.vel_y = (self.vel_y + 0x80).min(0x5ff);

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n110_puchi[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n115_ravil(
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
                    self.action_counter2 = 0;
                    self.vel_x = 0;
                }

                let player = self.get_closest_player_mut(players);
                if self.shock != 0
                    || (player.x < self.x + 0xc000
                        && player.x > self.x - 0xc000
                        && player.y < self.y + 0x4000
                        && player.y > self.y - 0xc000)
                {
                    self.action_num = 10;
                }
            }
            10 => {
                let player = self.get_closest_player_mut(players);

                self.anim_num = 1;
                self.direction = if self.x >= player.x { Direction::Left } else { Direction::Right };

                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 20;
                    self.action_counter = 0;
                }
            }
            20 => {
                self.damage = 0;
                self.vel_x = 0;

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_num += 1;
                    self.anim_counter = 0;
                    if self.anim_num > 2 {
                        let player = self.get_closest_player_mut(players);

                        self.direction = if self.x >= player.x { Direction::Left } else { Direction::Right };

                        self.vel_x = self.direction.vector_x() * 0x200;
                        self.action_counter2 += 1;
                        self.action_num = 21;
                        self.vel_y = -0x400;

                        if self.action_counter2 > 2 {
                            self.action_counter2 = 0;
                            self.anim_num = 4;
                            self.vel_x *= 2;
                            self.damage = 5;

                            state.sound_manager.play_sfx(102);
                        } else {
                            state.sound_manager.play_sfx(30);
                        }
                    }
                }
            }
            21 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 20;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                    self.damage = 0;

                    let player = self.get_closest_player_mut(players);
                    if player.x > self.x + 0x12000
                        && player.x < self.x - 0x12000
                        && player.y > self.y + 0x6000
                        && player.y < self.y - 0x12000
                    {
                        self.action_num = 0;
                    }
                }
            }
            30 => {
                let mut npc = NPC::create(4, &state.npc_table);
                for _ in 0..8 {
                    npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                    npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                    npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                    npc.vel_y = self.rng.range(-0x600..0) as i32;

                    let _ = npc_list.spawn(0x100, npc.clone());
                }

                self.anim_num = 0;
                self.action_num = 0;
            }
            50 | 51 => {
                if self.action_num == 50 {
                    self.action_num = 51;
                    self.anim_num = 4;
                    self.damage = 0;
                    self.vel_y = -0x200;

                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_solid_soft(false);
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 52;
                    self.anim_num = 5;
                    self.vel_x = 0;
                    state.sound_manager.play_sfx(23);
                }
            }
            _ => {}
        }

        self.vel_y += if self.action_num <= 50 { 0x40 } else { 0x20 };

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n115_ravil[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n192_scooter(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.display_bounds = Rect { left: 16 * 0x200, top: 8 * 0x200, right: 16 * 0x200, bottom: 8 * 0x200 };
            }
            10 => {
                self.action_num = 11;
                self.anim_num = 1;
                self.y -= 5 * 0x200;
                self.display_bounds.top = 16 * 0x200;
                self.display_bounds.bottom = 16 * 0x200
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 1;
                    self.target_x = self.x;
                    self.target_y = self.y;
                }

                self.x = self.target_x + self.rng.range(-1..1) as i32 * 0x200;
                self.y = self.target_y + self.rng.range(-1..1) as i32 * 0x200;

                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 30;
                }
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 1;
                    self.vel_x = -4 * 0x200;
                    self.x = self.target_x;
                    self.y = self.target_y;

                    state.sound_manager.play_sfx(44);
                }

                self.vel_x += 0x20;
                self.x += self.vel_x;
                self.y = self.target_y + self.rng.range(-1..1) as i32 * 0x200;
                self.action_counter += 1;

                if self.action_counter > 10 {
                    self.direction = Direction::Right;
                }

                if self.action_counter > 200 {
                    self.action_num = 40;
                }
            }
            40 | 41 => {
                if self.action_num == 40 {
                    self.action_num = 41;
                    self.action_counter = 2;
                    self.direction = Direction::Left;
                    self.y -= 48 * 0x200;
                    self.vel_x = -8 * 0x200;
                }

                self.x += self.vel_x;
                self.y += self.vel_y;

                self.action_counter += 2;

                if self.action_counter > 1200 {
                    self.cond.set_alive(false);
                }
            }
            _ => {}
        }

        if self.action_counter % 4 == 0 && self.action_num >= 20 {
            state.sound_manager.play_sfx(34);
            state.create_caret(
                self.x + self.direction.opposite().vector_x() * 10 * 0x200,
                self.y + 10 * 0x200,
                CaretType::Exhaust,
                self.direction.opposite(),
            );
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n192_scooter[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n193_broken_scooter(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.x += 24 * 0x200;
        }

        self.anim_rect = state.constants.npc.n193_broken_scooter;

        Ok(())
    }
}
