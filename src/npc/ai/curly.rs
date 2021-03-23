use num_traits::{abs, clamp};

use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;
use crate::weapon::bullet::BulletManager;
use crate::caret::CaretType;

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
                npc_list.create_death_smoke(self.x, self.y, self.display_bounds.right, 8, state, &self.rng);
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
            _ => {}
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
            _ => {}
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
}
