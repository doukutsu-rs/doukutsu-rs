use crate::framework::context::Context;
use crate::framework::error::GameResult;

use crate::caret::CaretType;
use crate::common::Direction;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n093_chaco(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                let player = self.get_closest_player_mut(players);
                if (self.x - player.x).abs() < 32 * 0x200
                    && self.y - 32 * 0x200 < player.y
                    && self.y + 16 * 0x200 > player.y {
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

                self.animate(4, 2, 5);

                self.x += self.direction.vector_x() * 0x200;
            }
            10 => {
                self.anim_num = 6;

                self.action_counter += 1;
                if self.action_counter > 200 {
                    self.action_counter = 0;

                    state.create_caret(self.x, self.y, CaretType::Zzz, Direction::Left);
                }
            }
            _ => {}
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 7 };

        self.anim_rect = state.constants.npc.n093_chaco[self.anim_num as usize + dir_offset];

        Ok(())
    }
}
