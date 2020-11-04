use num_traits::{abs, clamp};

use crate::common::Direction;
use ggez::GameResult;
use crate::npc::NPC;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n154_gaudi_dead(&mut self, state: &mut SharedGameState) -> GameResult {
        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n154_gaudi_dead[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n361_gaudi_dashing(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.vel_x2 = 0;
                    self.vel_y2 = 0;
                    self.action_num = 1;
                }

                if (self.direction == Direction::Right && player.x > self.x + 272 * 0x200 && player.x < self.x + 288 * 0x200)
                    || (self.direction == Direction::Left && player.x < self.x - 272 * 0x200 && player.x > self.x - 288 * 0x200) {
                    self.action_num = 10;
                } else {
                    return Ok(());
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.npc_flags.set_shootable(true);
                    self.action_num = 11;
                    self.damage = 5;
                }

                if self.x > player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }

                self.anim_counter += 1;
                if self.anim_counter > 1 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num > 1 {
                        self.anim_num = 0;
                    }
                }

                self.vel_x2 += self.direction.vector_x() * 0x10;
                self.vel_y2 += (player.y - self.y).signum() * 0x10;

                if self.vel_x2 < 0 && self.flags.hit_left_wall() {
                    self.vel_x2 /= -2;
                }

                if self.vel_x2 > 0 && self.flags.hit_right_wall() {
                    self.vel_x2 /= -2;
                }

                if self.vel_y2 < 0 && self.flags.hit_top_wall() {
                    self.vel_y2 *= -1;
                }

                if self.vel_y2 > 0 && self.flags.hit_bottom_wall() {
                    self.vel_y2 /= -2;
                }

                self.vel_x2 = clamp(self.vel_x2, -0x5ff, 0x5ff);
                self.vel_y2 = clamp(self.vel_y2, -0x5ff, 0x5ff);

                self.x += self.vel_x2;
                self.y += self.vel_y2;
            }
            _ => {}
        }

        if self.life <= 985 {
            self.cond.set_drs_destroyed(true);
            self.npc_type = 154;
            self.action_num = 0;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };
        self.anim_rect = state.constants.npc.n361_gaudi_dashing[self.anim_num as usize + dir_offset];

        Ok(())
    }
}
