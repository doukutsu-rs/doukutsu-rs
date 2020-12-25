use ggez::GameResult;
use num_traits::clamp;

use crate::caret::CaretType;
use crate::common::Direction;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n154_gaudi_dead(&mut self, state: &mut SharedGameState) -> GameResult {
        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n154_gaudi_dead[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n156_gaudi_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_counter > 300 || (self.flags.0 & 0xff) != 0 {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_num = (self.anim_num + 1) % 3;

        self.anim_rect = state.constants.npc.n156_gaudi_projectile[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n361_gaudi_dashing(&mut self, state: &mut SharedGameState, players: [&mut Player; 2], npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.vel_x2 = 0;
                    self.vel_y2 = 0;
                    self.action_num = 1;
                }

                let player = self.get_closest_player_mut(players);
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

                let player = self.get_closest_player_mut(players);
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
            npc_list.create_death_smoke(self.x, self.y, 0, 2, state, &self.rng);
            self.npc_type = 154;
            self.action_num = 0;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };
        self.anim_rect = state.constants.npc.n361_gaudi_dashing[self.anim_num as usize + dir_offset];

        Ok(())
    }
}
