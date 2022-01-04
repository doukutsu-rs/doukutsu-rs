use crate::{GameResult, SharedGameState};
use crate::caret::CaretType;
use crate::common::Direction;
use crate::npc::NPC;
use crate::rng::RNG;
use crate::player::Player;

impl NPC {
    pub(crate) fn tick_n337_numahachi(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y -= 0x1000;
        }

        if self.action_num == 1 {
            self.action_num = 2;
            self.anim_num = 0;
            self.vel_x = 0;
        }

        if self.action_num == 2{
            self.animate(50, 0, 1);
        }

        self.vel_y += 0x40;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n337_numahachi[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n357_puppy_ghost(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.anim_rect = state.constants.npc.n357_puppy_ghost;
                self.action_counter += 1;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    state.sound_manager.play_sfx(29);
                }

                self.action_counter += 1;
                self.anim_rect = state.constants.npc.n357_puppy_ghost;

                if self.action_counter & 2 != 0 {
                    self.anim_rect.right = self.anim_rect.left;
                }

                if self.action_counter > 50 {
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        if self.action_counter & 8 == 1 {
            state.create_caret(
                self.x + self.rng.range(-8..8) * 0x200,
                self.y + 0x1000,
                CaretType::LittleParticles,
                Direction::Up,
            );
        }

        Ok(())
    }

    pub(crate) fn tick_n309_bute(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {

        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                self.action_num = 1;

                if (self.direction == Direction::Left && player.x > self.x - 0x24000 && player.x < self.x - 0x22000) || 
                   (player.x < self.x + 0x24000 && player.x > self.x + 0x22000) {
                    self.action_num = 10;
                    return Ok(())
                }

            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.npc_flags.set_shootable(true);
                    self.damage = 5;
                }

                if self.x > player.x {
                    self.direction = Direction::Left;
                    self.vel_x2 -= 0x10;
                }
                else {
                    self.direction = Direction::Right;
                    self.vel_x2 += 0x10;
                }

                if self.y > player.y {
                    self.vel_y2 -= 0x10;
                }
                else {
                    self.vel_y2 += 0x10;
                }

                if self.vel_x2 < 0 && self.flags.hit_left_wall() { self.vel_x2 *= -1 };
                if self.vel_x2 > 0 && self.flags.hit_right_wall() { self.vel_x2 *= -1 };
                if self.vel_y2 < 0 && self.flags.hit_top_wall() { self.vel_y2 *= -1 };
                if self.vel_y2 > 0 && self.flags.hit_bottom_wall() { self.vel_y2 *= -1 };

                self.vel_x2 = self.vel_x2.clamp(-0x5FF, 0x5FF);
                self.vel_y2 = self.vel_y2.clamp(-0x5FF, 0x5FF);

                self.x += self.vel_x2;
                self.y += self.vel_y2;

                self.animate(1, 0, 1);
                let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };
                self.anim_rect = state.constants.npc.n309_bute[self.anim_num as usize + dir_offset];

            }
            _ => (),
        }

        if self.life <= 996 {
            self.npc_type = 316;
            self.action_num = 0;
        }

        Ok(())
    }

    pub(crate) fn tick_n316_bute_dead(&mut self, state: &mut SharedGameState) -> GameResult {
        // (nearly) same as Gaudi death
        match self.action_num {
            0 => {
                self.npc_flags.set_shootable(false);
                self.npc_flags.set_ignore_solidity(false);
                self.damage = 0;
                self.action_num = 1;
                self.anim_num = 0;
                self.display_bounds.top = 0x1800;
                self.display_bounds.right = 0x1800;
                self.display_bounds.left = 0x1800;
                self.vel_y = -0x200;

                match self.direction {
                    Direction::Left => self.vel_x = 0x100,
                    Direction::Right => self.vel_x = -0x100,
                    _ => (),
                };
                state.sound_manager.play_sfx(50);
            }
            1 if self.flags.hit_bottom_wall() => {
                self.action_num = 2;
                self.action_counter = 0;
                self.anim_num = 1;
                self.anim_counter = 0;
            }
            2 => {
                self.vel_x = 8 * self.vel_x / 9;
                self.animate(3, 1, 2);

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.cond.set_explode_die(true);
                }
            }
            _ => (),
        }

        self.vel_y += 0x20;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n316_bute_dead[self.anim_num as usize + dir_offset];

        Ok(())
    }
}
