use num_traits::{abs, clamp};

use crate::common::{Direction, CDEG_RAD};
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::{NPCContext, NPC};
use crate::game::shared_game_state::SharedGameState;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n044_polish(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { npc_list, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 | 2 => {
                if self.action_num <= 1 {
                    self.anim_num = 0;
                    self.action_num = match self.direction {
                        Direction::Left => 8,
                        Direction::Right => 2,
                        _ => 8,
                    };
                }
                self.vel_y += 0x20;
                if self.vel_y > 0 && self.flags.hit_bottom_wall() {
                    self.vel_y = -0x100;
                    self.vel_x += 0x100;
                }

                if self.flags.hit_right_wall() {
                    self.action_num = 3;
                }
            }
            3 => {
                self.vel_x += 0x20;
                if self.vel_x > 0 && self.flags.hit_right_wall() {
                    self.vel_x = -0x100;
                    self.vel_y -= 0x100;
                }

                if self.flags.hit_top_wall() {
                    self.action_num = 4;
                }
            }
            4 => {
                self.vel_y -= 0x20;
                if self.vel_y < 0 && self.flags.hit_top_wall() {
                    self.vel_y = 0x100;
                    self.vel_x -= 0x100;
                }

                if self.flags.hit_left_wall() {
                    self.action_num = 5;
                }
            }
            5 => {
                self.vel_x -= 0x20;
                if self.vel_x < 0 && self.flags.hit_left_wall() {
                    self.vel_x = 0x100;
                    self.vel_y += 0x100;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 2;
                }
            }
            6 => {
                self.vel_y += 0x20;
                if self.vel_y > 0 && self.flags.hit_bottom_wall() {
                    self.vel_y = -0x100;
                    self.vel_x -= 0x100;
                }

                if self.flags.hit_left_wall() {
                    self.action_num = 7;
                }
            }
            7 => {
                self.vel_x -= 0x20;
                if self.vel_x < 0 && self.flags.hit_left_wall() {
                    self.vel_x = 0x100;
                    self.vel_y -= 0x100;
                }

                if self.flags.hit_top_wall() {
                    self.action_num = 8;
                }
            }
            8 => {
                self.vel_y -= 0x20;
                if self.vel_y < 0 && self.flags.hit_top_wall() {
                    self.vel_y = 0x100;
                    self.vel_x += 0x100;
                }

                if self.flags.hit_right_wall() {
                    self.action_num = 9;
                }
            }
            9 => {
                self.vel_x += 0x20;
                if self.vel_x > 0 && self.flags.hit_right_wall() {
                    self.vel_x = -0x100;
                    self.vel_y += 0x100;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 6;
                }
            }
            _ => (),
        }

        if self.life <= 100 {
            npc_list.create_death_smoke(self.x, self.y, self.display_bounds.right as usize, 8, state, &self.rng);
            state.sound_manager.play_sfx(25);
            self.cond.set_alive(false);

            let mut npc = NPC::create(45, &state.npc_table);
            npc.cond.set_alive(true);
            npc.x = self.x;
            npc.y = self.y;
            for _ in 0..9 {
                let _ = npc_list.spawn(0x100, npc.clone());
            }
        }

        self.vel_x = clamp(self.vel_x, -0x200, 0x200);
        self.vel_y = clamp(self.vel_y, -0x200, 0x200);

        if self.shock > 0 {
            self.x += self.vel_x / 2;
            self.y += self.vel_y / 2;
        } else {
            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        if self.action_num > 1 && self.action_num <= 9 {
            self.anim_num += 1;
            if self.anim_num > 2 {
                self.anim_num = 1;
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n044_polish[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n045_baby(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 2;
            self.vel_x = if self.rng.next_u16() & 1 != 0 {
                self.rng.range(-0x200..-0x100) as i32
            } else {
                self.rng.range(0x100..0x200) as i32
            };
            self.vel_y = if self.rng.next_u16() & 1 != 0 {
                self.rng.range(-0x200..-0x100) as i32
            } else {
                self.rng.range(0x100..0x200) as i32
            };
            self.vel_x2 = self.vel_x;
            self.vel_y2 = self.vel_y;
        }

        match self.action_num {
            1 | 2 => {
                self.anim_num += 1;
                if self.anim_num > 2 {
                    self.anim_num = 1;
                }
            }
            _ => (),
        }

        if self.vel_x2 < 0 && self.flags.hit_left_wall() {
            self.vel_x2 = -self.vel_x2;
        }

        if self.vel_x2 > 0 && self.flags.hit_right_wall() {
            self.vel_x2 = -self.vel_x2;
        }

        if self.vel_y2 < 0 && self.flags.hit_top_wall() {
            self.vel_y2 = -self.vel_y2;
        }

        if self.vel_y2 > 0 && self.flags.hit_bottom_wall() {
            self.vel_y2 = -self.vel_y2;
        }

        self.vel_x2 = clamp(self.vel_x2, -0x200, 0x200);
        self.vel_y2 = clamp(self.vel_y2, -0x200, 0x200);

        if self.shock > 0 {
            self.x += self.vel_x2 / 2;
            self.y += self.vel_y2 / 2;
        } else {
            self.x += self.vel_x2;
            self.y += self.vel_y2;
        }

        self.anim_rect = state.constants.npc.n045_baby[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n047_sandcroc(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.target_y = self.y;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_ignore_solidity(false);
                    self.npc_flags.set_invulnerable(false);
                    self.npc_flags.set_solid_soft(false);
                }

                let player = self.get_closest_player_mut(players);
                if abs(self.x - player.x) < 0x1000 && player.y > self.y && player.y < self.y + 0x1000 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    state.sound_manager.play_sfx(102);
                }

                self.x += (player.x - self.x).signum() * 0x400;
            }
            2 => {
                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_num += 1;
                    self.anim_counter = 0;
                }

                match self.anim_num {
                    3 => self.damage = 10,
                    4 => {
                        self.action_num = 3;
                        self.action_counter = 0;
                        self.npc_flags.set_shootable(true);
                    }
                    _ => (),
                }
            }
            3 => {
                self.damage = 0;
                self.npc_flags.set_solid_soft(true);

                self.action_counter += 1;
                if self.shock > 0 {
                    self.action_num = 4;
                    self.action_counter = 0;
                }
            }
            4 => {
                self.npc_flags.set_ignore_solidity(true);
                self.y += 0x200;
                self.action_counter += 1;
                if self.action_counter == 32 {
                    self.action_num = 5;
                    self.action_counter = 0;
                    self.npc_flags.set_solid_soft(false);
                    self.npc_flags.set_shootable(false);
                }
            }
            5 => {
                if self.action_counter > 99 {
                    self.y = self.target_y;
                    self.action_num = 0;
                    self.anim_num = 0;
                } else {
                    self.action_counter += 1;
                }
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n047_sandcroc[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n049_skullhead(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        let parent = self.get_parent_ref_mut(npc_list);

        if self.action_num > 9 && parent.as_ref().map(|n| n.borrow().npc_type == 3).unwrap_or(false) {
            self.action_num = 3;
            self.vel_x = 0;
            self.vel_y = 0;
            self.action_counter2 = 1;
        }

        if self.flags.hit_left_wall() {
            self.direction = Direction::Right;
            self.vel_x = 0x100;
        }

        if self.flags.hit_right_wall() {
            self.direction = Direction::Left;
            self.vel_x = -0x100;
        }

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = if parent.is_some() { 10 } else { 1 };
                }

                self.action_counter += 1;
                if self.action_counter > 3 {
                    self.vel_y = -0x400;
                    self.action_num = 3;
                    self.anim_num = 2;

                    if self.action_counter2 > 0 {
                        self.vel_x = if self.direction == Direction::Left { -0x200 } else { 0x200 };
                    } else if self.direction != Direction::Left {
                        self.vel_x = 0x100;
                    } else {
                        self.vel_x = -0x100;
                    }
                }

                self.anim_num = 1;
            }
            3 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.vel_x = 0;
                }

                self.anim_num = if self.flags.hit_bottom_wall() || self.vel_y > 0 { 1 } else { 2 };
            }
            10 => {
                if self.vel_y2 >= 50 {
                    let player = self.get_closest_player_mut(players);

                    if abs(self.x - player.x) < 0x10000 && abs(self.y - player.y) < 0xc000 {
                        self.action_num = 11;
                        self.action_counter = 0;
                        self.anim_num = 2;
                    }
                } else {
                    self.vel_y2 += 1;
                }
            }
            11 => {
                self.action_counter += 1;
                if self.action_counter == 30 || self.action_counter == 35 {
                    let player = self.get_closest_player_mut(players);

                    let angle = f64::atan2((self.y + 0x800 - player.y) as f64, (self.x - player.x) as f64);

                    let mut npc = NPC::create(50, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.vel_x = (angle.cos() * -1024.0) as i32;
                    npc.vel_y = (angle.sin() * -1024.0) as i32;

                    let _ = npc_list.spawn(0x100, npc);
                    state.sound_manager.play_sfx(39);
                }

                if self.action_counter > 50 {
                    self.action_num = 10;
                    self.vel_y2 = 0;
                    self.anim_num = 1;
                }
            }
            _ => (),
        }

        if self.action_num > 9 {
            if let Some(parent) = parent {
                let mut parent = parent.borrow_mut();

                self.x = parent.x;
                self.y = parent.y + 0x2000;
                self.direction = parent.direction;
                parent.vel_y2 -= 1;
            }
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n049_skullhead[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n050_skeleton_projectile(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 && self.direction == Direction::Right {
                    self.action_num = 2;
                }

                self.x += self.vel_x;
                self.y += self.vel_y;

                if self.flags.hit_left_wall() {
                    self.action_num = 2;
                    self.vel_x = 0x200;
                    self.action_counter2 += 1;
                }

                if self.flags.hit_right_wall() {
                    self.action_num = 2;
                    self.vel_x = -0x200;
                    self.action_counter2 += 1;
                }

                if self.flags.hit_top_wall() {
                    self.action_num = 2;
                    self.vel_y = 0x200;
                    self.action_counter2 += 1;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 2;
                    self.vel_y = -0x200;
                    self.action_counter2 += 1;
                }
            }
            2 => {
                self.vel_y += 0x40;
                self.x += self.vel_x;
                self.y += self.vel_y;

                if self.flags.hit_bottom_wall() {
                    self.action_counter2 += 1;
                    if self.action_counter2 > 1 {
                        state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
                        self.cond.set_alive(false);
                    }
                }
            }
            _ => (),
        }

        self.vel_y = clamp(self.vel_y, -0x5ff, 0x5ff);

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;

            self.anim_num = if self.direction == Direction::Left {
                (self.anim_num + 1) % 4
            } else {
                self.anim_num.wrapping_sub(1) % 4
            }
        }

        self.anim_rect = state.constants.npc.n050_skeleton_projectile[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n051_crow_and_skullhead(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                let is_in_spawn_radius = (player.x - self.x).abs() < 0x28000 && (player.y - self.y).abs() < 0x28000;

                if self.action_num == 0 && is_in_spawn_radius {
                    self.action_num = 1;
                    self.target_x = self.x;
                    self.target_y = self.y;
                    self.vel_y = 0x400;

                    let mut npc = NPC::create(49, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.parent_id = self.id;
                    let _ = npc_list.spawn(0, npc);
                } else if self.action_num == 1 || (self.action_num == 0 && is_in_spawn_radius) {
                    self.direction = if player.x >= self.x { Direction::Right } else { Direction::Left };
                    self.vel_y += (self.target_y - self.y).signum() * 0x0a;
                    self.vel_y = clamp(self.vel_y, -0x200, 0x200);

                    if self.vel_y2 >= 10 {
                        self.action_num = 2;
                    } else {
                        self.vel_y2 += 1;
                    }
                }
            }
            2 => {
                self.direction = if player.x >= self.x { Direction::Right } else { Direction::Left };

                self.vel_x += if self.y <= player.y + 0x4000 {
                    (player.x - self.x).signum() * 0x10
                } else {
                    (self.x - player.x).signum() * 0x10
                };

                self.vel_y += (player.y - self.y).signum() * 0x10;

                if self.shock > 0 {
                    self.vel_x = 0;
                    self.vel_y += 0x20;
                }
            }
            _ => (),
        }

        if self.vel_x < 0 && self.flags.hit_left_wall() {
            self.vel_x = 0x100;
        }

        if self.vel_x > 0 && self.flags.hit_right_wall() {
            self.vel_x = -0x100;
        }

        if self.vel_y < 0 && self.flags.hit_top_wall() {
            self.vel_y = 0x100;
        }

        if self.vel_y > 0 && self.flags.hit_bottom_wall() {
            self.vel_y = -0x100;
        }

        self.vel_x = clamp(self.vel_x, -0x400, 0x400);
        self.vel_y = clamp(self.vel_y, -0x200, 0x200);

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.shock > 0 {
            self.anim_num = 4;
        } else if self.action_num == 2 && self.y < player.y - 0x4000 {
            self.anim_num = 0;
        } else if self.action_num != 0 {
            self.animate(1, 0, 1);
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 5 };

        self.anim_rect = state.constants.npc.n051_crow_and_skullhead[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n053_skullstep_leg(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { npc_list, .. }: NPCContext,
    ) -> GameResult {
        let parent = self.get_parent_ref_mut(npc_list);
        if parent.is_none() || parent.as_ref().unwrap().borrow().npc_type == 3 {
            self.vanish(state);
            npc_list.create_death_smoke(self.x, self.y, self.display_bounds.right as usize, 4, state, &self.rng);
            return Ok(());
        }

        let parent = parent.unwrap();

        let mut parent = parent.borrow_mut();

        let angle = (self.vel_x + parent.vel_y2) & 0xFF;

        if self.action_num < 2 {
            if self.action_num == 0 {
                self.action_num = 1;
                self.action_counter2 = 10;
            }

            if self.direction == Direction::Left && self.flags.hit_left_slope() {
                parent.y -= 0x400;
                parent.vel_y -= 0x100;
            }

            if self.direction == Direction::Right && self.flags.hit_right_slope() {
                parent.y -= 0x400;
                parent.vel_y -= 0x100;
            }

            if self.flags.hit_bottom_wall() {
                parent.y -= 0x400;
                parent.vel_y -= 0x100;
                parent.vel_x += parent.direction.vector_x() * 0x80;
            }

            self.x = parent.x + (self.action_counter2 as f64 * (angle as f64 * CDEG_RAD).cos() * 512.0) as i32;
            self.y = parent.y + (self.action_counter2 as f64 * (angle as f64 * CDEG_RAD).sin() * 512.0) as i32;
        }

        self.direction = parent.direction;
        self.anim_num = if !(0x14..=0x6c).contains(&angle) { 1 } else { 0 };

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n053_skullstep_leg[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n054_skullstep(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { npc_list, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 1;

                    let mut leg = NPC::create(53, &state.npc_table);
                    leg.cond.set_alive(true);
                    leg.direction = self.direction;
                    leg.parent_id = self.id;

                    let _ = npc_list.spawn(256, leg.clone());

                    leg.vel_x = 0x80;
                    let _ = npc_list.spawn(0, leg);
                }

                self.vel_y2 += self.direction.vector_x() * 6;

                if self.flags.hit_bottom_wall() {
                    self.vel_x = self.vel_x * 3 / 4;
                    self.action_counter += 1;
                    if self.action_counter > 60 {
                        self.action_num = 2;
                        self.action_counter = 0;
                    }
                } else {
                    self.action_counter = 0;
                }

                if self.direction == Direction::Left && self.flags.hit_left_wall() {
                    self.action_counter2 += 1;
                    if self.action_counter2 > 8 {
                        self.direction = Direction::Right;
                        self.vel_x = -self.vel_x;
                    }
                } else if self.direction == Direction::Right && self.flags.hit_right_wall() {
                    self.action_counter2 += 1;
                    if self.action_counter2 > 8 {
                        self.direction = Direction::Left;
                        self.vel_x = -self.vel_x;
                    }
                } else {
                    self.action_counter2 = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                self.shock += self.action_counter & 0xff;
                if self.action_counter > 50 {
                    state.sound_manager.play_sfx(25);
                    self.vanish(state);
                    npc_list.create_death_smoke(
                        self.x,
                        self.y,
                        self.display_bounds.right as usize,
                        8,
                        state,
                        &self.rng,
                    );
                }
            }
            _ => (),
        }

        self.vel_y += 0x80;
        self.vel_x = clamp(self.vel_x, -0x2ff, 0x2ff);
        self.vel_y = clamp(self.vel_y, -0x2ff, 0x2ff);

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n054_skullstep[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n056_tan_beetle(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = if self.direction == Direction::Left { 1 } else { 3 };
            }
            1 => {
                self.vel_x -= 0x10;
                if self.vel_x < -0x400 {
                    self.vel_x = -0x400;
                }

                self.x += if self.shock != 0 { self.vel_x / 2 } else { self.vel_x };

                self.animate(1, 1, 2);

                if self.flags.hit_left_wall() {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.vel_x = 0;
                    self.direction = Direction::Right;
                }
            }
            2 => {
                let player = self.get_closest_player_mut(players);
                if self.x < player.x && self.x > player.x - 0x20000 && (self.y - player.y).abs() < 0x1000 {
                    self.action_num = 3;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }
            }
            3 => {
                self.vel_x += 0x10;
                if self.vel_x > 0x400 {
                    self.vel_x = 0x400;
                }

                self.x += if self.shock != 0 { self.vel_x / 2 } else { self.vel_x };

                self.animate(1, 1, 2);

                if self.flags.hit_right_wall() {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.vel_x = 0;
                    self.direction = Direction::Left;
                }
            }
            4 => {
                let player = self.get_closest_player_mut(players);
                if self.x > player.x && self.x < player.x + 0x20000 && (self.y - player.y).abs() < 0x1000 {
                    self.action_num = 1;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n056_tan_beetle[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n057_crow(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = self.rng.range(0..1) as u16;
                    self.anim_counter = self.rng.range(0..4) as u16;
                    self.action_counter2 = 120;

                    let angle = self.rng.range(0..255);

                    self.vel_x = ((angle as f64 * CDEG_RAD).cos() * 512.0) as i32;
                    self.target_x = self.x + 8 * (((angle + 0x40) as f64 * CDEG_RAD).cos() * 512.0) as i32;

                    let angle = self.rng.range(0..255);
                    self.vel_y = ((angle as f64 * CDEG_RAD).sin() * 512.0) as i32;
                    self.target_y = self.y + 8 * (((angle + 0x40) as f64 * CDEG_RAD).sin() * 512.0) as i32;
                }

                let player = self.get_closest_player_mut(players);

                self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };

                self.vel_x += (self.target_x - self.x).signum() * 0x10;
                self.vel_y += (self.target_y - self.y).signum() * 0x10;

                self.vel_x = clamp(self.vel_x, -0x200, 0x200);
                self.vel_y = clamp(self.vel_y, -0x200, 0x200);

                if self.shock != 0 {
                    self.action_num = 2;
                    self.action_counter = 0;

                    self.vel_x = self.direction.opposite().vector_x() * 0x200;
                    self.vel_y = 0;
                }
            }
            2 => {
                let player = self.get_closest_player_mut(players);

                self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
                self.vel_x += if self.y <= player.y + 0x6000 {
                    (player.x - self.x).signum() * 0x10
                } else {
                    (self.x - player.x).signum() * 0x10
                };

                self.vel_y += (player.y - self.y).signum() * 0x10;

                if self.shock > 0 {
                    self.vel_x = 0;
                    self.vel_y += 0x20;
                }

                if self.vel_x < 0 && self.flags.hit_left_wall() {
                    self.vel_x = 0x200;
                }

                if self.vel_x > 0 && self.flags.hit_right_wall() {
                    self.vel_x = -0x200;
                }

                if self.vel_y < 0 && self.flags.hit_top_wall() {
                    self.vel_y = 0x200;
                }

                if self.vel_y > 0 && self.flags.hit_bottom_wall() {
                    self.vel_y = -0x200;
                }

                self.vel_x = clamp(self.vel_x, -0x5ff, 0x5ff);
                self.vel_y = clamp(self.vel_y, -0x5ff, 0x5ff);
            }
            _ => (),
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.shock > 0 {
            self.anim_num = 4;
        } else {
            self.animate(1, 0, 1);
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 5 };

        self.anim_rect = state.constants.npc.n057_crow[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n120_colon_a(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        let anim = if self.direction == Direction::Left { 0 } else { 1 };

        self.anim_rect = state.constants.npc.n120_colon_a[anim];

        Ok(())
    }

    pub(crate) fn tick_n121_colon_b(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        if self.direction != Direction::Left {
            self.anim_rect = state.constants.npc.n121_colon_b[2];

            self.action_counter += 1;
            if self.action_counter > 100 {
                self.action_counter = 0;
                state.create_caret(self.x, self.y, CaretType::Zzz, Direction::Left);
            }

            return Ok(());
        }

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n121_colon_b[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n122_colon_enraged(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                let player = self.get_closest_player_mut(players);

                if self.x - 0x4000 < player.x
                    && self.x + 0x4000 > player.x
                    && self.y - 0x4000 < player.y
                    && self.y + 0x2000 > player.y
                {
                    self.face_player(player);
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.life = 1000;
                    self.action_num = 11;
                    self.action_counter = self.rng.range(0..50) as u16;
                    self.anim_num = 0;
                    self.damage = 0;
                }

                if self.action_counter != 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 13;
                }
            }
            13 | 14 => {
                if self.action_num == 13 {
                    self.action_num = 14;
                    self.action_counter = self.rng.range(0..50) as u16;

                    let player = self.get_closest_player_mut(players);

                    self.face_player(player);
                }

                self.animate(2, 2, 5);

                self.vel_x = if self.direction != Direction::Left { self.vel_x + 64 } else { self.vel_x - 64 };

                if self.action_counter != 0 {
                    self.action_counter -= 1;
                } else {
                    self.npc_flags.set_shootable(true);
                    self.action_num = 15;
                    self.anim_num = 2;
                    self.vel_y = -512;
                    self.damage = 2;
                }
            }
            15 => {
                if self.flags.hit_bottom_wall() {
                    self.npc_flags.set_shootable(true);
                    self.vel_x = 0;
                    self.action_num = 10;
                    self.damage = 0;
                }
            }
            20 => {
                if self.flags.hit_bottom_wall() {
                    self.vel_x = 0;
                    self.action_num = 21;
                    self.damage = 0;
                    if self.anim_num == 6 {
                        self.anim_num = 8;
                    } else {
                        self.anim_num = 9;
                    }
                    self.action_counter = self.rng.range(300..400) as u16;
                }
            }
            21 => {
                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.npc_flags.set_shootable(true);
                    self.life = 1000;
                    self.action_num = 11;
                    self.action_counter = self.rng.range(0..50) as u16;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }
        if self.action_num > 10 && self.action_num < 20 && self.life != 1000 {
            self.action_num = 20;
            self.vel_y = -512;
            self.anim_num = self.rng.range(6..7) as u16;
            self.npc_flags.set_shootable(false);
        }
        self.vel_y += 32;

        self.vel_x = self.vel_x.clamp(-0x1FF, 0x1FF);

        self.clamp_fall_speed();
        self.y += self.vel_y;
        self.x += self.vel_x;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 10 };

        self.anim_rect = state.constants.npc.n122_colon_enraged[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n124_sunstone(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.x += 0x1000;
                    self.y += 0x1000;
                }

                self.npc_flags.set_ignore_solidity(false);
                self.anim_num = 0;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 1;

                    self.npc_flags.set_ignore_solidity(true);
                }

                match self.direction {
                    Direction::Left => self.x -= 0x80,
                    Direction::Up => self.y -= 0x80,
                    Direction::Right => self.x += 0x80,
                    Direction::Bottom => self.y += 0x80,
                    Direction::FacingPlayer => {}
                }

                self.action_counter += 1;

                state.quake_counter = 20;
                state.quake_rumble_counter = 20;
                if self.action_counter % 8 == 0 {
                    state.sound_manager.play_sfx(26);
                }
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n124_sunstone[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n126_puppy_running(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                let player = self.get_closest_player_ref(&players);

                if (self.x - player.x).abs() < 0xc000 && self.y - 0x4000 < player.y && self.y + 0x2000 > player.y {
                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
                }

                if (self.x - player.x).abs() < 0x4000 && self.y - 0x4000 < player.y && self.y + 0x2000 > player.y {
                    self.action_num = 10;
                    self.direction = if self.x > player.x { Direction::Right } else { Direction::Left };
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 4;
                    self.anim_counter = 0;
                }

                if self.flags.hit_bottom_wall() {
                    self.animate(2, 4, 5);
                } else {
                    self.anim_num = 5;
                    self.anim_counter = 0;
                }

                if self.vel_x < 0 && self.flags.hit_left_wall() {
                    self.vel_x /= -2;
                    self.direction = Direction::Right;
                }

                if self.vel_x > 0 && self.flags.hit_right_wall() {
                    self.vel_x /= -2;
                    self.direction = Direction::Left;
                }

                self.vel_x += self.direction.vector_x() * 0x40;

                // what the hell pixel?
                if self.vel_x > 0x5ff {
                    self.vel_x = 0x400;
                }

                if self.vel_x < -0x5ff {
                    self.vel_x = -0x400;
                }
            }
            _ => (),
        }

        // why
        self.npc_flags.set_interactable(false);
        for player in players {
            if player.controller.trigger_down() {
                self.npc_flags.set_interactable(true);
            }
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n126_puppy_running[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n130_puppy_sitting(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.npc_flags.set_interactable(true);
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                let player = self.get_closest_player_mut(players);
                if self.x - 0x8000 < player.x
                    && self.x + 0x8000 > player.x
                    && self.y - 0x4000 < player.y
                    && self.y + 0x2000 > player.y
                {
                    self.anim_counter += 1;
                    if self.anim_counter > 3 {
                        self.anim_counter = 0;
                        self.anim_num += 1;
                    }

                    if self.anim_num > 3 {
                        self.anim_num = 2;
                    }
                }

                if self.x - 0xC000 < player.x
                    && self.x + 0xC000 > player.x
                    && self.y - 0x4000 < player.y
                    && self.y + 0x2000 > player.y
                {
                    if self.x <= player.x {
                        self.direction = Direction::Right;
                    } else {
                        self.direction = Direction::Left;
                    }
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();
        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.npc.n130_puppy_sitting[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n131_puppy_sleeping(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        self.action_counter += 1;
        if self.action_counter > 100 {
            self.action_counter = 0;
            state.create_caret(self.x, self.y, CaretType::Zzz, Direction::Left);
        }

        let anim = if self.direction == Direction::Left { 0 } else { 1 };

        self.anim_rect = state.constants.npc.n131_puppy_sleeping[anim];

        Ok(())
    }

    pub(crate) fn tick_n132_puppy_barking(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, .. }: NPCContext,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                if (self.x - player.x).abs() < 0x8000 && (self.y - player.y).abs() < 0x2000 {
                    self.animate(4, 2, 4);

                    if self.anim_num == 4 && self.anim_counter == 0 {
                        state.sound_manager.play_sfx(105);
                    }
                } else {
                    if self.anim_num == 4 {
                        self.anim_num = 2;
                    }
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 12;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            12 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 11;
                    self.anim_num = 0;
                }
            }
            100 => {
                self.action_num = 101;
                self.action_counter2 = 0;
            }
            101 => {
                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 4 {
                        if self.action_counter2 > 2 {
                            self.anim_num = 0;
                            self.action_counter2 = 0;
                        } else {
                            self.anim_num = 2;
                            self.action_counter2 += 1;
                        }
                    }
                }

                if self.anim_num == 4 && self.anim_counter == 0 {
                    state.sound_manager.play_sfx(105);
                }
            }
            120 => {
                self.anim_num = 0;
            }
            _ => (),
        }

        if self.action_num < 100 {
            self.face_player(player);
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 5 };

        self.anim_rect = state.constants.npc.n132_puppy_barking[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n133_jenka(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n133_jenka[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n136_puppy_carried(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;

                    self.npc_flags.set_interactable(false);
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        // todo dog stacking?
        let player_index = 0;
        let player = &players[player_index];

        self.direction = player.direction;
        self.y = player.y - 0x1400;
        self.x = player.x + 0x800 * self.direction.opposite().vector_x();

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n136_puppy_carried[self.anim_num as usize + dir_offset];

        if (player.anim_num & 1) != 0 {
            self.anim_rect.top += 1;
        }

        Ok(())
    }

    pub(crate) fn tick_n134_armadillo(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, bullet_manager, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 2;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_invulnerable(true);
                }

                let player = self.get_closest_player_mut(players);
                if player.x > self.x - 0x28000
                    && player.x < self.x + 0x28000
                    && player.y > self.y - 0x14000
                    && player.y < self.y + 0x8000
                {
                    self.action_num = 10;
                    self.npc_flags.set_shootable(true);
                    self.npc_flags.set_invulnerable(false);
                }
            }
            10 => {
                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }
                if self.anim_num > 1 {
                    self.anim_num = 0;
                }
                if self.direction == Direction::Left && self.flags.hit_left_wall() {
                    self.direction = Direction::Right;
                }
                if self.direction == Direction::Right && self.flags.hit_right_wall() {
                    self.direction = Direction::Left;
                }
                self.x += self.direction.vector_x() * 0x100;

                if bullet_manager.count_bullets_type_idx_all(6) != 0 {
                    self.action_num = 20;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_invulnerable(true);
                }
            }
            20 => {
                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 10;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.npc_flags.set_shootable(true);
                    self.npc_flags.set_invulnerable(false);
                }
            }
            _ => (),
        }
        self.vel_y += 0x40;
        self.clamp_fall_speed();
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n134_armadillo[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n135_skeleton(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        if player.x < self.x - 0x2C000
            || player.x > self.x + 0x2C000
            || player.y < self.y - 0x14000
            || player.y > self.y + 0x8000
        {
            self.action_num = 0;
        }

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.vel_x = 0;
                }

                if player.x > self.x - 0x28000
                    && player.x < self.x + 0x28000
                    && player.y > self.y - 0x14000
                    && player.y < self.y + 0x8000
                {
                    self.action_num = 10;
                }

                if self.flags.hit_bottom_wall() {
                    self.anim_num = 0;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.vel_x = 0;
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 4 && self.flags.hit_bottom_wall() {
                    self.action_num = 20;
                    self.anim_num = 1;
                    self.action_counter2 = 0;
                    self.vel_y = -0x200 * self.rng.range(1..3);

                    if self.shock != 0 {
                        self.vel_x = if self.x >= player.x { self.vel_x + 0x100 } else { self.vel_x - 0x100 };
                    } else {
                        self.vel_x = if self.x >= player.x { self.vel_x - 0x100 } else { self.vel_x + 0x100 }
                    }
                }
            }
            20 => {
                if self.vel_y > 0 && self.action_counter2 == 0 {
                    self.action_counter2 += 1;

                    let angle = f64::atan2((self.y + 0x800 - player.y) as f64, (self.x - player.x) as f64);

                    let mut npc = NPC::create(50, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.vel_x = (angle.cos() * -1024.0) as i32;
                    npc.vel_y = (angle.sin() * -1024.0) as i32;

                    let _ = npc_list.spawn(0x180, npc);
                    state.sound_manager.play_sfx(39);
                }
                if self.flags.hit_bottom_wall() {
                    self.action_num = 10;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        if self.action_num > 9 {
            if self.x <= player.x {
                self.direction = Direction::Right;
            } else {
                self.direction = Direction::Left;
            }
        }
        self.vel_y += 0x33;
        self.clamp_fall_speed();

        self.vel_x = clamp(self.vel_x, -0x5ff, 0x5ff);
        self.y += self.vel_y;
        self.x += self.vel_x;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n135_skeleton[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n138_large_door(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        if self.action_num != 1 {
            if self.action_num > 1 {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 1;
                    self.action_counter = 0;
                    self.npc_flags.set_ignore_solidity(true);
                } else if self.action_num != 11 {
                    return Ok(());
                }

                self.action_counter += 1;
                if (self.action_counter & 7) == 0 {
                    state.sound_manager.play_sfx(26);
                }

                if self.direction != Direction::Left {
                    self.x = ((self.action_counter as i32 / 8) * 0x200) + self.target_x;
                    self.anim_rect.left = 112;
                    self.anim_rect.top = 112;
                    self.anim_rect.right = 128;
                    self.anim_rect.bottom = 136;
                    self.anim_rect.right -= self.action_counter / 8;
                } else {
                    self.anim_rect.left = 96;
                    self.anim_rect.top = 112;
                    self.anim_rect.right = 112;
                    self.anim_rect.bottom = 136;
                    self.anim_rect.left += self.action_counter / 8;
                }
                if self.action_counter == 104 {
                    self.cond.set_alive(false);
                }
            } else if self.action_num == 0 {
                self.action_num = 1;
                if self.direction != Direction::Left {
                    self.anim_rect.left = 112;
                    self.anim_rect.top = 112;
                    self.anim_rect.right = 128;
                    self.anim_rect.bottom = 136;
                    self.x -= 4096;
                } else {
                    self.anim_rect.left = 96;
                    self.anim_rect.top = 112;
                    self.anim_rect.right = 112;
                    self.anim_rect.bottom = 136;
                    self.x += 4096;
                }
                self.target_x = self.x;
            }
        }
        Ok(())
    }

    pub(crate) fn tick_n143_jenka_collapsed(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        let anim = if self.direction == Direction::Left { 0 } else { 1 };

        self.anim_rect = state.constants.npc.n143_jenka_collapsed[anim];

        Ok(())
    }
}
