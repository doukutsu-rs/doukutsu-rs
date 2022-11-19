use num_traits::abs;

use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n040_santa(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..120) == 0 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                let player = self.get_closest_player_mut(players);
                if abs(self.x - player.x) < 0x4000 && self.y - 0x4000 < player.y && self.y + 0x2000 > player.y {
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
            3 | 4 => {
                if self.action_num == 3 {
                    self.action_num = 4;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num > 5 {
                        self.anim_num = 2;
                    }
                }

                self.x += self.direction.vector_x() * 0x200;
            }
            5 => {
                self.anim_num = 6;
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 7 };

        self.anim_rect = state.constants.npc.n040_santa[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n307_santa_caged(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.x += 0x200;
                    self.y -= 0x400;
                }

                if self.rng.range(0..160) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 12 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n307_santa_caged[self.anim_num as usize + dir_offset];

        Ok(())
    }
}
