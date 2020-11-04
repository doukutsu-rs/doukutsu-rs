use ggez::GameResult;
use num_traits::{abs, clamp};

use crate::common::Direction;
use crate::npc::NPC;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n129_fireball_snake_trail(&mut self, state: &mut SharedGameState) -> GameResult {
        self.anim_counter += 1;

        if self.anim_counter > 1 {
            self.anim_counter = 0;

            self.anim_num += 1;
            if self.anim_num > 2 {
                self.cond.set_alive(false);
                return Ok(());
            }
        }

        self.y += self.vel_y;

        if self.anim_counter == 1 {
            let frame = (self.action_counter2 as usize % 6) * 3 + self.anim_num as usize;
            self.anim_rect = state.constants.npc.n129_fireball_snake_trail[frame];
        }

        Ok(())
    }
}
