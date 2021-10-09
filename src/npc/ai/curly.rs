use num_traits::{abs, clamp};

use crate::caret::CaretType;
use crate::common::{Direction, Rect};
use crate::framework::error::GameResult;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::{Player, TargetPlayer};
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;
use crate::weapon::bullet::BulletManager;

impl NPC {
    pub(crate) fn tick_n117_curly(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    let player = self.get_closest_player_mut(players);

                    if self.direction == Direction::FacingPlayer {
                        self.direction = if self.x <= player.x { Direction::Right } else { Direction::Left };
                    }

                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                self.vel_x = 0;
                self.vel_y += 0x40;
            }
            3 | 4 => {
                if self.action_num == 3 {
                    self.action_num = 4;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }

                self.animate(4, 1, 4);

                self.vel_x = self.direction.vector_x() * 0x200;
                self.vel_y += 0x40;
            }
            5 => {
                self.action_num = 6;
                self.anim_num = 5;
                npc_list.create_death_smoke(self.x, self.y, self.display_bounds.right as usize, 8, state, &self.rng);
            }
            6 => {
                self.anim_num = 5;
            }
            10 | 11 => {
                let player = self.get_closest_player_mut(players);

                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                    self.direction = if self.x <= player.x { Direction::Right } else { Direction::Left };
                }

                self.animate(4, 1, 4);
                self.x += self.direction.vector_x() * 0x200;

                if abs(self.x - player.x) > 0x2800 {
                    self.action_num = 0;
                }
            }
            20 => {
                self.vel_x = 0;
                self.anim_num = 6;
            }
            21 => {
                self.vel_x = 0;
                self.anim_num = 9;
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 0;
                    self.vel_y = -0x400;
                }

                self.anim_num = 7;
                self.vel_x = self.direction.vector_x() * 0x200;
                self.vel_y += 0x40;

                self.action_counter += 1;
                if self.action_counter > 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 32;
                }
            }
            32 => {
                self.vel_x = 0;
                self.vel_y += 0x40;
                self.anim_num = 8;
            }
            70 | 71 => {
                if self.action_num == 70 {
                    self.action_num = 71;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }

                self.animate(8, 1, 4);
                self.x += self.direction.vector_x() * 0x100;
            }
            _ => (),
        }

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 10 };

        self.anim_rect = state.constants.npc.n117_curly[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n118_curly_boss(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        bullet_manager: &BulletManager,
    ) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 0;
                self.anim_counter = 0;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = self.rng.range(50..100) as u16;
                    self.anim_num = 0;

                    let player = self.get_closest_player_mut(players);
                    self.direction = if self.x <= player.x { Direction::Right } else { Direction::Left };
                    self.npc_flags.set_shootable(true);
                    self.npc_flags.set_invulnerable(false);
                }

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 13;
                }
            }
            13 | 14 => {
                if self.action_num == 13 {
                    self.action_num = 14;
                    self.action_counter = self.rng.range(50..100) as u16;
                    self.anim_num = 3;

                    let player = self.get_closest_player_mut(players);
                    self.direction = if self.x <= player.x { Direction::Right } else { Direction::Left };
                }

                self.animate(2, 3, 6);

                self.vel_x += self.direction.vector_x() * 0x40;

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.npc_flags.set_shootable(true);
                    self.action_num = 20;
                    self.action_counter = 0;
                    state.sound_manager.play_sfx(103);
                }
            }
            20 => {
                let player = self.get_closest_player_mut(players);

                self.direction = if self.x <= player.x { Direction::Right } else { Direction::Left };

                self.vel_x = 8 * self.vel_x / 9;

                self.anim_num += 1;
                if self.anim_num > 1 {
                    self.anim_num = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 21;
                    self.action_counter = 0;
                }
            }
            21 => {
                self.action_counter += 1;
                if self.action_counter % 4 == 1 {
                    let player = self.get_closest_player_mut(players);

                    let facing_up = (self.direction == Direction::Left && self.x < player.x)
                        || (self.direction == Direction::Right && self.x > player.x);

                    let mut npc = NPC::create(123, &state.npc_table);
                    npc.cond.set_alive(true);

                    if facing_up {
                        self.anim_num = 2;
                        npc.x = self.x;
                        npc.y = self.y - 0x1000;
                        npc.direction = Direction::Up;
                    } else {
                        self.anim_num = 0;
                        self.x += self.direction.opposite().vector_x() * 0x200;

                        npc.x = self.x + self.direction.vector_x() * 0x1000;
                        npc.y = self.y + 0x800;
                        npc.direction = self.direction;
                    }

                    let _ = npc_list.spawn(256, npc);
                }

                if self.action_counter > 30 {
                    self.action_num = 10;
                }
            }
            30 => {
                self.anim_num += 1;
                if self.anim_num > 8 {
                    self.anim_num = 7;
                }

                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 10;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        if self.action_num > 10 && self.action_num < 30 && bullet_manager.count_bullets_type_idx_all(6) > 0 {
            self.action_num = 30;
            self.action_counter = 0;
            self.anim_num = 7;

            self.vel_x = 0;
            self.npc_flags.set_shootable(false);
            self.npc_flags.set_invulnerable(true);
        }

        self.vel_x = clamp(self.vel_x, -0x1ff, 0x1ff);
        self.vel_y += 0x20;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 9 };

        self.anim_rect = state.constants.npc.n118_curly_boss[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n123_curly_boss_bullet(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
            state.sound_manager.play_sfx(32);

            match self.direction {
                Direction::Left => {
                    self.vel_x = -0x1000;
                    self.vel_y = self.rng.range(-0x80..0x80);
                }
                Direction::Up => {
                    self.vel_x = self.rng.range(-0x80..0x80);
                    self.vel_y = -0x1000;
                }
                Direction::Right => {
                    self.vel_x = 0x1000;
                    self.vel_y = self.rng.range(-0x80..0x80);
                }
                Direction::Bottom => {
                    self.vel_x = self.rng.range(-0x80..0x80);
                    self.vel_y = 0x1000;
                }
                Direction::FacingPlayer => unreachable!(),
            }

            self.anim_rect = state.constants.npc.n123_curly_boss_bullet[self.direction as usize];
        }

        if match self.direction {
            Direction::Left if self.flags.hit_left_wall() => true,
            Direction::Right if self.flags.hit_right_wall() => true,
            Direction::Up if self.flags.hit_top_wall() => true,
            Direction::Bottom if self.flags.hit_bottom_wall() => true,
            _ => false,
        } {
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Right);
            state.sound_manager.play_sfx(28);
            self.cond.set_alive(false);

            return Ok(());
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n165_curly_collapsed(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.y += 0x1400;
                }

                let player = self.get_closest_player_mut(players);
                self.anim_num = if self.direction == Direction::Right {
                    if player.x > self.x - 0x4000
                        && player.x < self.x + 0x4000
                        && player.y > self.y - 0x2000
                        && player.y < self.y + 0x2000
                    {
                        2
                    } else {
                        1
                    }
                } else {
                    0
                };
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n165_curly_collapsed[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n180_curly_ai(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        let player = self.get_closest_player_ref(&players);

        if self.y >= player.y - 0x14000 {
            if state.npc_curly_counter > 0 {
                self.target_x = state.npc_curly_target.0;
                self.target_y = state.npc_curly_target.1;
            } else {
                self.target_x = player.x;
                self.target_y = player.y;
            }
        } else {
            self.target_x = if self.y > 0x1FFFF { 0 } else { 0x280000 };
            self.target_y = self.y;
        }

        if (self.vel_x < 0 && self.flags.hit_left_wall()) || (self.vel_x > 0 && self.flags.hit_right_wall()) {
            self.vel_x = 0;
        }

        match self.action_num {
            20 => {
                self.action_num = 100;
                self.anim_num = 0;
                self.x = player.x;
                self.y = player.y;

                let mut npc = NPC::create(183, &state.npc_table);
                npc.cond.set_alive(true);
                npc.parent_id = self.id;
                let _ = npc_list.spawn(0x100, npc);

                if !state.get_flag(563) {
                    let mut npc = NPC::create(181, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.parent_id = self.id;
                    let _ = npc_list.spawn(0x100, npc);
                } else {
                    let mut npc = NPC::create(182, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.parent_id = self.id;
                    let _ = npc_list.spawn(0x100, npc);
                }
            }
            40 | 41 => {
                if self.action_num == 40 {
                    self.action_num = 41;
                    self.action_counter = 0;
                    self.anim_num = 10;
                }

                self.action_counter += 1;
                if self.action_counter == 750 {
                    self.npc_flags.set_interactable(false);
                    self.anim_num = 0;
                }
                if self.action_counter > 1000 {
                    self.action_num = 100;
                    self.anim_num = 0;

                    let mut npc = NPC::create(183, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.parent_id = self.id;
                    let _ = npc_list.spawn(0x100, npc);

                    if !state.get_flag(563) {
                        let mut npc = NPC::create(181, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.parent_id = self.id;
                        let _ = npc_list.spawn(0x100, npc);
                    } else {
                        let mut npc = NPC::create(182, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.parent_id = self.id;
                        let _ = npc_list.spawn(0x100, npc);
                    }
                }
            }
            100 => {
                self.anim_num = 0;
                self.vel_x = 7 * self.vel_x / 8;
                self.action_counter3 = 0;

                if self.x <= self.target_x + 0x2000 {
                    if self.x < self.target_x - 0x2000 {
                        self.action_num = 300;
                        self.anim_num = 1;
                        self.direction = Direction::Right;
                        self.action_counter = self.rng.range(20..60) as u16;
                    }
                } else {
                    self.action_num = 200;
                    self.anim_num = 1;
                    self.direction = Direction::Left;
                    self.action_counter = self.rng.range(20..60) as u16;
                }
            }
            200 => {
                self.vel_x -= 0x20;
                self.direction = Direction::Left;
                if self.flags.hit_left_wall() {
                    self.action_counter3 += 1;
                } else {
                    self.action_counter3 = 0;
                }
            }
            210 => {
                self.vel_x -= 0x20;
                self.direction = Direction::Left;
                if self.flags.hit_bottom_wall() {
                    self.action_num = 100;
                }
            }
            300 => {
                self.vel_x += 0x20;
                self.direction = Direction::Right;
                if self.flags.hit_right_wall() {
                    self.action_counter3 += 1;
                } else {
                    self.action_counter3 = 0;
                }
            }
            310 => {
                self.vel_x += 0x20;
                self.direction = Direction::Right;
                if self.flags.hit_bottom_wall() {
                    self.action_num = 100;
                }
            }
            _ => (),
        }

        if state.npc_curly_counter > 0 {
            state.npc_curly_counter -= 1;
        }

        if state.npc_curly_counter == 70 {
            self.action_counter2 = 10;
        } else if state.npc_curly_counter == 60 && self.flags.hit_bottom_wall() && self.rng.range(0..2) != 0 {
            self.action_num = if self.x <= self.target_x { 310 } else { 210 };
            self.anim_num = 1;
            self.action_counter3 = 0;
            self.vel_y = -0x600;
            state.sound_manager.play_sfx(15);
        }

        let mut delx = self.x - self.target_x;
        let dely = self.y - self.target_y;
        if delx < 0 {
            delx = -delx;
        }

        if self.action_num == 100 {
            self.anim_num = if delx + 0x400 >= dely { 0 } else { 5 }
        }

        if self.action_num == 210 || self.action_num == 310 {
            self.anim_num = if delx + 0x400 >= dely { 1 } else { 6 }
        }

        if self.action_num == 200 || self.action_num == 300 {
            self.anim_counter += 1;
            self.anim_num = (self.anim_counter / 4 % 4) + if delx + 0x400 >= dely { 1 } else { 6 };

            if self.action_counter > 0 {
                self.action_counter -= 1;
                if self.flags.any_flag() && self.action_counter3 > 10 {
                    self.action_num += 10;
                    self.anim_num = 1;
                    self.action_counter3 = 0;
                    self.vel_y = -0x600;
                    state.sound_manager.play_sfx(15);
                }
            } else {
                self.action_num = 100;
                self.anim_num = 0;
            }
        }

        if self.action_num >= 100 && self.action_num < 500 {
            if self.x >= player.x - 0xA000 && self.x <= player.x + 0xA000 {
                self.vel_y += 0x33;
            } else {
                self.vel_y += if self.flags.any_flag() { 0x10 } else { 0x33 };
            }
        }

        self.vel_x = self.vel_x.clamp(-0x300, 0x300);

        if self.vel_y > 0x5FF {
            self.vel_y = 0x5FF;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.action_num >= 100 && !self.flags.hit_bottom_wall() && self.anim_num != 1000 {
            if delx + 0x400 >= dely {
                self.anim_num = 1;
            } else {
                self.anim_num = 6;
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 11 };

        self.anim_rect = state.constants.npc.n180_curly_ai[self.anim_num as usize + dir_offset];
        Ok(())
    }

    pub(crate) fn tick_n181_curly_ai_machine_gun(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
        bullet_manager: &mut BulletManager,
    ) -> GameResult {
        if let Some(parent) = self.get_parent_ref_mut(npc_list) {
            if parent.anim_num > 4 {
                self.direction = parent.direction;
                self.x = parent.x;
                self.y = parent.y - 0x1400;
                self.anim_num = 1;
            } else {
                self.x = parent.x
                    + if parent.direction == Direction::Left {
                        self.direction = Direction::Left;
                        -0x1000
                    } else {
                        self.direction = Direction::Right;
                        0x1000
                    };
                self.y = parent.y;
                self.anim_num = 0;
            }

            if parent.anim_num == 1 || parent.anim_num == 3 || parent.anim_num == 6 || parent.anim_num == 8 {
                self.y -= 0x200;
            }

            if self.action_num == 0 {
                if parent.action_counter2 == 10 {
                    parent.action_counter2 = 0;
                    self.action_num = 10;
                    self.action_counter = 0;
                }
            } else if self.action_num == 10 {
                self.action_counter += 1;
                if self.action_counter % 6 == 1 {
                    if self.anim_num != 0 {
                        if self.direction != Direction::Left {
                            bullet_manager.create_bullet(
                                self.x + 0x400,
                                self.y - 0x800,
                                12,
                                TargetPlayer::Player1,
                                Direction::Up,
                                &state.constants,
                            );
                            state.create_caret(self.x + 0x400, self.y - 0x800, CaretType::Shoot, Direction::Left);
                        } else {
                            bullet_manager.create_bullet(
                                self.x - 0x400,
                                self.y - 0x800,
                                12,
                                TargetPlayer::Player1,
                                Direction::Up,
                                &state.constants,
                            );
                            state.create_caret(self.x - 0x400, self.y - 0x800, CaretType::Shoot, Direction::Left);
                        }
                    } else if self.direction != Direction::Left {
                        bullet_manager.create_bullet(
                            self.x + 0x800,
                            self.y + 0x600,
                            12,
                            TargetPlayer::Player1,
                            Direction::Right,
                            &state.constants,
                        );
                        state.create_caret(self.x + 0x800, self.y + 0x600, CaretType::Shoot, Direction::Left);
                    } else {
                        bullet_manager.create_bullet(
                            self.x - 0x800,
                            self.y + 0x600,
                            12,
                            TargetPlayer::Player1,
                            Direction::Left,
                            &state.constants,
                        );
                        state.create_caret(self.x - 0x800, self.y + 0x600, CaretType::Shoot, Direction::Left);
                    }
                }
                if self.action_counter == 60 {
                    self.action_num = 0;
                }
            }

            let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

            self.anim_rect = state.constants.npc.n181_curly_ai_machine_gun[self.anim_num as usize + dir_offset];
        }
        Ok(())
    }

    pub(crate) fn tick_n182_curly_ai_polar_star(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
        bullet_manager: &mut BulletManager,
    ) -> GameResult {
        if let Some(parent) = self.get_parent_ref_mut(npc_list) {
            if parent.anim_num > 4 {
                self.direction = parent.direction;
                self.x = parent.x;
                self.y = parent.y - 0x1400;
                self.anim_num = 1;
            } else {
                self.x = parent.x
                    + if parent.direction == Direction::Left {
                        self.direction = Direction::Left;
                        -0x1000
                    } else {
                        self.direction = Direction::Right;
                        0x1000
                    };
                self.y = parent.y;
                self.anim_num = 0;
            }
            if parent.anim_num == 1 || parent.anim_num == 3 || parent.anim_num == 6 || parent.anim_num == 8 {
                self.y -= 0x200;
            }

            if self.action_num == 0 {
                if parent.action_counter2 == 10 {
                    parent.action_counter2 = 0;
                    self.action_num = 10;
                    self.action_counter = 0;
                }
            } else if self.action_num == 10 {
                self.action_counter += 1;
                if self.action_counter % 6 == 1 {
                    if self.anim_num != 0 {
                        if self.direction != Direction::Left {
                            bullet_manager.create_bullet(
                                self.x + 0x400,
                                self.y - 0x800,
                                12,
                                TargetPlayer::Player1,
                                Direction::Up,
                                &state.constants,
                            );
                            state.create_caret(self.x + 0x400, self.y - 0x800, CaretType::Shoot, Direction::Left);
                        } else {
                            bullet_manager.create_bullet(
                                self.x - 0x400,
                                self.y - 0x800,
                                12,
                                TargetPlayer::Player1,
                                Direction::Up,
                                &state.constants,
                            );
                            state.create_caret(self.x - 0x400, self.y - 0x800, CaretType::Shoot, Direction::Left);
                        }
                    } else if self.direction != Direction::Left {
                        bullet_manager.create_bullet(
                            self.x + 0x800,
                            self.y + 0x600,
                            12,
                            TargetPlayer::Player1,
                            Direction::Right,
                            &state.constants,
                        );
                        state.create_caret(self.x + 0x800, self.y + 0x600, CaretType::Shoot, Direction::Left);
                    } else {
                        bullet_manager.create_bullet(
                            self.x - 0x800,
                            self.y + 0x600,
                            12,
                            TargetPlayer::Player1,
                            Direction::Left,
                            &state.constants,
                        );
                        state.create_caret(self.x - 0x800, self.y + 0x600, CaretType::Shoot, Direction::Left);
                    }
                }
                if self.action_counter == 60 {
                    self.action_num = 0;
                }
            }

            let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

            self.anim_rect = state.constants.npc.n182_curly_ai_polar_star[self.anim_num as usize + dir_offset];
        }
        Ok(())
    }

    pub(crate) fn tick_n183_curly_air_tank_bubble(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        if let Some(parent) = self.get_parent_ref_mut(npc_list) {
            if self.action_num == 0 {
                self.x = parent.x;
                self.y = parent.y;
                self.action_num = 1;
            }

            self.x += (parent.x - self.x) / 2;
            self.y += (parent.y - self.y) / 2;

            self.animate(1, 0, 1);

            self.anim_rect = if parent.flags.in_water() {
                state.constants.npc.n183_curly_air_tank_bubble[self.anim_num as usize]
            } else {
                Rect::new(0, 0, 0, 0)
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n259_curly_unconcious(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.npc_flags.set_interactable(false);
                    self.action_num = 1;
                }

                let player = &players[0];

                self.direction = player.direction;
                self.x = player.x + 0x600 * self.direction.opposite().vector_x();
                self.y = player.y - 0x800;

                let dir_offset = if self.direction == Direction::Left { 0 } else { 1 };

                self.anim_rect = state.constants.npc.n259_curly_unconscious[dir_offset];

                if (player.anim_num & 1) != 0 {
                    self.anim_rect.top += 1;
                }
            }
            10 => {
                self.action_num = 11;
                self.vel_x = 0x40;
                self.vel_y = -0x20;
                self.anim_rect = state.constants.npc.n259_curly_unconscious[0];
            }
            11 => {
                if self.y <= 0x7FFF {
                    self.vel_y = 0x20;
                }
                self.x += self.vel_x;
                self.y += self.vel_y;
            }
            20 => {
                self.vanish(state);
                npc_list.create_death_smoke_up(self.x, self.y, 0x2000, 64, state, &self.rng);
            }
            _ => (),
        }

        Ok(())
    }
}
